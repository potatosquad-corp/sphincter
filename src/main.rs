use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use dashmap::DashMap;
use futures::{sink::SinkExt};
use rand::{Rng, distr::Alphanumeric};
use std::sync::Arc;
use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream}};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

struct AppState {
    rooms: DashMap<String, broadcast::Sender<Vec<u8>>>,
}

#[derive(serde::Deserialize)]
struct WsParams {
    room: String,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, env = "SPHINCTER_TCP_ADDR", default_value = "0.0.0.0:9000")]
    tcp_addr: String,
    #[arg(short, long, env = "SPHINCTER_WS_ADDR", default_value = "127.0.0.1:8080")]
    ws_addr: String,
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    info!("Starting Sphincter v1.0");

    let state = Arc::new(AppState {
        rooms: DashMap::new(),
    });

    let tcp_state = state.clone();
    tokio::spawn(async move {
        start_tcp_server(tcp_state,args.tcp_addr).await;
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    info!("[WEB] Sphincter listening on {}", args.ws_addr);
    
    let listener = tokio::net::TcpListener::bind(args.ws_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn start_tcp_server(state: Arc<AppState>,tcp_addr: String) {
    let listener = TcpListener::bind(&tcp_addr).await.expect("Failed to bind TCP port 9000");
    info!("[TCP] Sphincter ready on {}", tcp_addr);

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("[TCP] New connection from {}", addr);
                let state = state.clone();
                tokio::spawn(async move {
                    handle_tcp_connection(socket, state).await;
                });
            }
            Err(e) => error!("[TCP] Accept error: {}", e),
        }
    }
}

async fn handle_tcp_connection(mut socket: TcpStream, state: Arc<AppState>) {
    let room_id: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();

    info!("[TCP] Room created: {} ", room_id);

    let (tx, _rx) = broadcast::channel(100);
    
    state.rooms.insert(room_id.clone(), tx.clone());

    let mut buffer = [0u8; 4096];
    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                info!("[TCP] Connection closed (Room {})", room_id);
                break;
            }
            Ok(n) => {
                let data = buffer[0..n].to_vec();
                let _ = tx.send(data);
            }
            Err(e) => {
                error!("[TCP] Read error Room {}: {}", room_id, e);
                break;
            }
        }
    }

    state.rooms.remove(&room_id);
    info!("[TCP] Room {} cleaned up.", room_id);
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if state.rooms.contains_key(&params.room) {
        ws.on_upgrade(move |socket| handle_ws_socket(socket, params.room, state))
    } else {
        warn!("[WEB] Access denied to missing room: {}", params.room);
        "Room not found".into_response()
    }
}

async fn handle_ws_socket(mut socket: WebSocket, room_id: String, state: Arc<AppState>) {
    info!("[WEB] Client joined Room {}", room_id);

    let mut rx = match state.rooms.get(&room_id) {
        Some(r) => r.subscribe(),
        None => {
            let _ = socket.close().await;
            return;
        }
    };

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                if let Err(e) = socket.send(Message::Binary(msg.into())).await {
                    warn!("[WEB] Client send error: {}", e);
                    break;
                }
            }

            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("[WEB] Client left Room {}", room_id);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}