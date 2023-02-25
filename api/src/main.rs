use std::{
    collections::{hash_map::Entry, HashMap},
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    headers::Origin,
    response::IntoResponse,
    routing::get,
    Router, TypedHeader,
};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use rand::Rng;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type GameID = u32;

#[derive(Clone, Default)]
struct AppState {
    games: Arc<RwLock<HashMap<GameID, Mutex<Game>>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

struct Player {
    name: String,
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

enum GameState {
    Lobby,
    Started,
}

struct Game {
    state: GameState,
    join: Sender<Player>,
}

impl Game {
    fn new(join: Sender<Player>) -> Self {
        Self {
            state: GameState::Lobby,
            join,
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                concat!(env!("CARGO_PKG_NAME"), "=debug,tower_http=debug").into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(AppState::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    TypedHeader(origin): TypedHeader<Origin>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // TODO: check origin for security
    println!("{addr} connected from origin {origin}");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

#[derive(thiserror::Error, Debug)]
enum HandlerError {
    #[error("JSON decode error")]
    JSONDecode(#[from] serde_json::Error),
    #[error("connection closed")]
    ConnectionClosed,
    #[error("didn't receive message")]
    NoRecv,
    #[error("no open game slots, try again later")]
    NoOpenSlots,
    #[error("game not found")]
    GameNotFound,
    #[error("game already started")]
    AlreadyStarted,
    #[error("axum error")]
    Axum(#[from] axum::Error),
}

enum MsgData {
    Text(String),
    Binary(Vec<u8>),
}

async fn recv_next(receiver: &mut SplitStream<WebSocket>) -> Result<MsgData, HandlerError> {
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(t) => return Ok(MsgData::Text(t)),
            Message::Binary(b) => return Ok(MsgData::Binary(b)),
            Message::Close(_) => return Err(HandlerError::ConnectionClosed),
            Message::Ping(_) | Message::Pong(_) => {}
        }
    }
    Err(HandlerError::NoRecv)
}

async fn recv_json<T>(receiver: &mut SplitStream<WebSocket>) -> Result<T, HandlerError>
where
    T: DeserializeOwned,
{
    Ok(match recv_next(receiver).await? {
        MsgData::Text(t) => serde_json::from_str(t.as_ref())?,
        MsgData::Binary(b) => serde_json::from_slice(b.as_ref())?,
    })
}

async fn send_json<T>(
    sender: &mut SplitSink<WebSocket, Message>,
    value: &T,
) -> Result<(), axum::Error>
where
    T: Sized + Serialize,
{
    let msg_str = serde_json::to_string(value).unwrap();
    sender.send(Message::Text(msg_str)).await
}

#[derive(Deserialize)]
enum Room {
    Create,
    Join(GameID),
}

#[derive(Deserialize)]
struct IdentifyMessage {
    name: String,
    room: Room,
}

async fn create_game(state: &AppState, host: Player) -> Result<(), HandlerError> {
    let (join, receive_joins) = channel(0);
    let game = Mutex::new(Game::new(join));
    for _ in 0..1000 {
        let game_id: GameID = rand::thread_rng().gen_range(0..100000);
        match state.games.write().await.entry(game_id) {
            Entry::Occupied(_) => {}
            Entry::Vacant(v) => {
                tokio::spawn(game_handler(host, receive_joins));
                v.insert(game);
                return Ok(());
            }
        }
    }
    Err(HandlerError::NoOpenSlots)
}

async fn join_game(state: &AppState, game_id: &GameID, player: Player) -> Result<(), HandlerError> {
    let mut games = state.games.write().await;
    let mut game = games
        .get_mut(game_id)
        .ok_or(HandlerError::GameNotFound)?
        .lock()
        .await;
    match game.state {
        GameState::Lobby => {
            game.join.send(player);
            Ok(())
        }
        GameState::Started => Err(HandlerError::AlreadyStarted),
    }
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, state: AppState) {
    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (sender, receiver) = socket.split();

    match handle_socket_inner(sender, receiver, who, state).await {
        Ok(()) => {}
        Err((mut sender, err)) => {
            tracing::debug!("{:#?}", err);
            send_json::<Value>(&mut sender, &json!({ "error": format!("{}", err) })).await;
        }
    }
}

macro_rules! map_err {
    ($sender:expr, $e:expr) => {
        match $e {
            Err(e) => return Err(($sender, e)),
            Ok(v) => v,
        }
    };
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket_inner(
    mut sender: SplitSink<WebSocket, Message>,
    mut receiver: SplitStream<WebSocket>,
    who: SocketAddr,
    state: AppState,
) -> Result<(), (SplitSink<WebSocket, Message>, HandlerError)> {
    let identify = map_err!(sender, recv_json::<IdentifyMessage>(&mut receiver).await);
    let (send_tx, mut send_rx) = channel(0);
    let (mut recv_tx, recv_rx) = channel(0);
    let player = Player {
        name: identify.name,
        tx: send_tx,
        rx: recv_rx,
    };

    match identify.room {
        Room::Create => map_err!(sender, create_game(&state, player).await),
        Room::Join(game_id) => map_err!(sender, join_game(&state, &game_id, player).await),
    };

    let mut send_task: JoinHandle<Result<(), HandlerError>> = tokio::spawn(async move {
        while let Some(msg) = send_rx.next().await {
            sender.send(msg).await?;
        }
        Ok(())
    });

    let mut recv_task: JoinHandle<Result<(), HandlerError>> = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            recv_tx.send(msg?).await;
        }
        Ok(())
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(Ok(())) => {},
                Err(b) => tracing::debug!("Error sending messages {:?}", b),
                Ok(Err(b)) => tracing::debug!("Error sending messages {:?}", b)
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(Ok(())) => {},
                Err(b) => tracing::debug!("Error receiving messages {:?}", b),
                Ok(Err(b)) => tracing::debug!("Error receiving messages {:?}", b)
            }
            send_task.abort();
        }
    }

    Ok(())
}

async fn game_handler(host: Player, receive_joins: Receiver<Player>) {}
