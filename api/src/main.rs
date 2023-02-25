use std::{
    collections::{hash_map::Entry, HashMap},
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
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
use rand::Rng;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// allows splitting the websocket stream into separate TX and RX branches
use futures::{
    channel::mpsc::{Receiver, Sender},
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

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

struct Game {}

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
enum IdentifyMessage {
    Create,
    Join(GameID),
}

#[derive(Serialize)]
enum IdentifyResponse {
    NotFound,
    OK,
}

fn create_game(state: &AppState) -> Result<&Mutex<Game>, HandlerError> {
    let mut rng = rand::thread_rng();
    let game = Mutex::new(Game {});
    for _ in 0..1000 {
        let game_id: GameID = rng.gen_range(0..100000);
        match state.games.write().unwrap().entry(game_id) {
            Entry::Occupied(_) => {}
            Entry::Vacant(v) => {
                return Ok(v.insert(game));
            }
        }
    }
    Err(HandlerError::NoOpenSlots)
}

fn join_game<'a>(state: &'a AppState, game_id: &GameID) -> Result<&'a Mutex<Game>, HandlerError> {
    match state.games.read().unwrap().get(game_id) {
        Some(game) => Ok(game),
        None => Err(HandlerError::GameNotFound),
    }
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, state: AppState) {
    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();

    match handle_socket_inner(&mut sender, &mut receiver, who, state).await {
        Ok(()) => {}
        Err(err) => {
            tracing::debug!("{:#?}", err);
            send_json::<Value>(&mut sender, &json!({ "error": format!("{}", err) })).await;
        }
    }
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket_inner(
    sender: &mut SplitSink<WebSocket, Message>,
    receiver: &mut SplitStream<WebSocket>,
    who: SocketAddr,
    state: AppState,
) -> Result<(), HandlerError> {
    let game: &Mutex<Game> = match recv_json::<IdentifyMessage>(receiver).await? {
        IdentifyMessage::Create => create_game(&state)?,
        IdentifyMessage::Join(game_id) => join_game(&state, &game_id)?,
    };

    Ok(())
}
