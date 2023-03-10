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
use capnp::message::{HeapAllocator, ReaderOptions};
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
    Stream,
};
use rand::Rng;
mod c2s_capnp;
mod s2c_capnp;
mod schema_server_generated;
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
    tx: UnboundedSender<Message>,
    rx: UnboundedReceiver<Vec<u8>>,
}

enum GameState {
    Lobby,
    Started,
}

type Join = (SocketAddr, Player);
type JoinsSender = UnboundedSender<Join>;
type JoinsReceiver = UnboundedReceiver<Join>;

struct Game {
    state: GameState,
    join: JoinsSender,
}

impl Game {
    fn new(join: JoinsSender) -> Self {
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
    #[error("expected binary")]
    ExpectedBinary,
    #[error("proto decode error: {0}")]
    ProtoError(#[from] capnp::Error),
    #[error("encountered value not in schema")]
    NotInSchema(#[from] capnp::NotInSchema),
    #[error("unexpected message type")]
    UnexpectedMessage,
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
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
    #[error("mpsc error: {0}")]
    Mpsc(#[from] futures::channel::mpsc::SendError),
}

async fn recv_next<S>(receiver: &mut S) -> Result<Vec<u8>, HandlerError>
where
    S: Stream<Item = Result<Message, axum::Error>> + Unpin,
{
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(_) => return Err(HandlerError::ExpectedBinary),
            Message::Binary(b) => return Ok(b),
            Message::Close(_) => return Err(HandlerError::ConnectionClosed),
            Message::Ping(_) | Message::Pong(_) => {} // ignore
        }
    }
    Err(HandlerError::NoRecv)
}

fn serialize(builder: &capnp::message::Builder<HeapAllocator>) -> Message {
    Message::Binary(capnp::serialize::write_message_to_words(builder))
}

fn error_msg(error: &str) -> Message {
    let mut builder = capnp::message::Builder::new_default();
    let mut err = builder.init_root::<s2c_capnp::msg::Builder>().init_error();
    err.set_msg(error);
    serialize(&builder)
}

async fn create_game(
    state: AppState,
    host_addr: SocketAddr,
    host: Player,
) -> Result<(), HandlerError> {
    let (join, receive_joins) = unbounded();
    let game = Mutex::new(Game::new(join));
    for _ in 0..1000 {
        let game_id: GameID = rand::thread_rng().gen_range(0..100000);
        match state.games.write().await.entry(game_id) {
            Entry::Occupied(_) => {}
            Entry::Vacant(v) => {
                tokio::spawn(game_handler(
                    state.clone(),
                    game_id,
                    host_addr,
                    host,
                    receive_joins,
                ));
                v.insert(game);
                return Ok(());
            }
        }
    }
    Err(HandlerError::NoOpenSlots)
}

async fn join_game(
    state: &AppState,
    game_id: &GameID,
    player_addr: SocketAddr,
    player: Player,
) -> Result<(), HandlerError> {
    let games = state.games.read().await;
    let mut game = games
        .get(game_id)
        .ok_or(HandlerError::GameNotFound)?
        .lock()
        .await;
    match game.state {
        GameState::Lobby => {
            game.join
                .send((player_addr, player))
                .await
                .map_err(|_| HandlerError::AlreadyStarted)?;
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
            match sender.send(error_msg(format!("{}", err).as_ref())).await {
                Ok(_) => {}
                Err(e) => tracing::debug!("error sending error msg: {}", e),
            };
        }
    }
}

enum GameRef {
    Create,
    Join(GameID),
}

struct Identify {
    name: String,
    game: GameRef,
}

fn parse_identify(dat: &[u8]) -> Result<Identify, HandlerError> {
    let msg =
        capnp::serialize::read_message_from_flat_slice(&mut dat.as_ref(), ReaderOptions::new())?;
    let identify = match msg.get_root::<c2s_capnp::msg::Reader>()?.which()? {
        c2s_capnp::msg::Which::Identify(i) => i?,
        _ => Err(HandlerError::UnexpectedMessage)?,
    };
    let name: String = identify.get_name()?.to_owned();
    let game = match identify.get_game()?.which()? {
        c2s_capnp::game_ref::Which::Create(()) => GameRef::Create,
        c2s_capnp::game_ref::Which::Join(j) => GameRef::Join(j),
    };
    Ok(Identify { name, game })
}

/// Basically ? except returns ownership of $sender to caller
macro_rules! map_err {
    ($sender:expr, $e:expr) => {
        match $e {
            Err(e) => return Err(($sender, e.into())),
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
    let dat = map_err!(sender, recv_next(&mut receiver).await);
    let identify = map_err!(sender, parse_identify(dat.as_ref()));

    let (send_tx, mut send_rx) = unbounded();
    let (mut recv_tx, recv_rx) = unbounded();
    let player = Player {
        name: identify.name,
        tx: send_tx,
        rx: recv_rx,
    };

    match identify.game {
        GameRef::Create => map_err!(sender, create_game(state.clone(), who, player).await),
        GameRef::Join(id) => map_err!(sender, join_game(&state, &id, who, player).await),
    };

    let mut send_task: JoinHandle<Result<(), HandlerError>> = tokio::spawn(async move {
        while let Some(msg) = send_rx.next().await {
            sender.send(msg).await?;
        }
        Ok(())
    });

    let mut recv_task: JoinHandle<Result<(), HandlerError>> = tokio::spawn(async move {
        loop {
            recv_tx.send(recv_next(&mut receiver).await?).await?;
        }
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

type Connections = HashMap<SocketAddr, Player>;

async fn broadcast(connections: &mut Connections, msg: Message) {
    for c in connections.values_mut() {
        if let Err(e) = c.tx.send(msg.clone()).await {
            tracing::debug!("Failed to send broadcast: {e}");
        }
    }
}

async fn handle_lobby(
    state: &AppState,
    game_id: &GameID,
    connections: &mut Connections,
    host_addr: &SocketAddr,
    mut receive_joins: UnboundedReceiver<(SocketAddr, Player)>,
) -> Option<()> {
    loop {
        tokio::select! {
            msg = connections.get_mut(host_addr).unwrap().rx.next() => {
                let Some(msg) = msg else {
                    tracing::debug!("Host disconnected from lobby");
                    return None;
                };
                // TODO
            }
            join = receive_joins.next() => {
                let Some((new_addr, new_player)) = join else { continue };
                // TODO
                connections.insert(new_addr, new_player);
            }
        };
    }
}

fn game_created(game_id: GameID) -> Message {
    let mut builder = capnp::message::Builder::new_default();
    let mut gc = builder
        .init_root::<s2c_capnp::msg::Builder>()
        .init_game_created();
    gc.set_id(game_id);
    serialize(&builder)
}

async fn game_handler(
    state: AppState,
    game_id: GameID,
    host_addr: SocketAddr,
    mut host: Player,
    receive_joins: JoinsReceiver,
) {
    tracing::debug!("Starting game with ID {}", game_id);
    let Ok(_) = host.tx.send(game_created(game_id)).await else {
        tracing::debug!("Failed to ack game creation");
        return;
    };

    let mut connections: Connections = HashMap::new();
    connections.insert(host_addr, host);
    let Some(()) = handle_lobby(&state, &game_id, &mut connections, &host_addr, receive_joins).await else {
        tracing::debug!("Failed to handle lobby");
        return;
    };
    tracing::debug!("Game starts with {} players", connections.len());
}
