// gamenv/http_api/mod.rs - HTTP API module
// Corresponds to txpike9/gamenv/single/daemons/http_api/

pub mod auth;
pub mod virtual_conn;
pub mod command_queue;
pub mod config;
pub mod utils;
pub mod handlers;
pub mod thread_manager;

pub use auth::*;
pub use virtual_conn::*;
pub use command_queue::*;
pub use config::*;

use axum::{
    extract::{State, WebSocketUpgrade, ws::Message},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// HTTP API state
#[derive(Clone)]
pub struct HttpApiState {
    pub virtual_conns: Arc<RwLock<VirtualConnectionPool>>,
    pub cmd_queue: Arc<RwLock<CommandQueue>>,
    pub config: Arc<HttpApiConfig>,
}

/// HTTP API main router
pub fn create_router() -> Router {
    let state = HttpApiState {
        virtual_conns: Arc::new(RwLock::new(VirtualConnectionPool::new())),
        cmd_queue: Arc::new(RwLock::new(CommandQueue::new())),
        config: Arc::new(HttpApiConfig::default()),
    };

    Router::new()
        // WebSocket connection
        .route("/ws", get(ws_handler))
        // REST API
        .route("/api/command", post(execute_command))
        .route("/api/status", get(get_status))
        .route("/api/user/:userid", get(get_user_info))
        // Static files
        .route("/static/*path", get(static_files))
        // Home page
        .route("/", get(index))
        .with_state(state)
}

/// WebSocket handler
pub async fn ws_handler(
    State(state): State<HttpApiState>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// WebSocket connection handler
async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    state: HttpApiState,
) {
    tracing::info!("WebSocket connection established");

    while let Some(result) = socket.recv().await {
        match result {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    // Convert Utf8Bytes to String
                    let text_str = text.to_string();
                    if let Err(e) = handle_ws_message(text_str, state.clone(), &mut socket).await {
                        tracing::error!("WS message error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("WS error: {:?}", e);
                break;
            }
        }
    }
}

/// Handle WebSocket message
async fn handle_ws_message(
    msg: String,
    state: HttpApiState,
    socket: &mut axum::extract::ws::WebSocket,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse message: {"action":"command","userid":"xxx","cmd":"look"}
    let req: WsRequest = serde_json::from_str(&msg)?;

    match req.action.as_str() {
        "command" => {
            let result = execute_command_internal(
                req.userid.unwrap_or_default(),
                req.cmd.unwrap_or_default(),
                state.clone(),
            ).await?;

            // Convert String to Utf8Bytes for Message::Text
            let json_str = serde_json::to_string(&result)?;
            socket.send(Message::Text(json_str.into())).await?;
        }
        "auth" => {
            let auth_result = handle_auth(req.txd.unwrap_or_default(), &state).await?;
            let json_str = serde_json::to_string(&auth_result)?;
            socket.send(Message::Text(json_str.into())).await?;
        }
        _ => {}
    }

    Ok(())
}

/// WebSocket request format
#[derive(Debug, Deserialize)]
struct WsRequest {
    action: String,
    userid: Option<String>,
    cmd: Option<String>,
    txd: Option<String>,
}

/// WebSocket response format
#[derive(Debug, Serialize)]
struct WsResponse {
    status: String,
    data: Option<serde_json::Value>,
    output: Option<String>,
}

/// Execute command API
pub async fn execute_command(
    State(state): State<HttpApiState>,
    Json(req): Json<CommandRequest>,
) -> Result<Json<CommandResponse>, ApiError> {
    let result = execute_command_internal(req.userid, req.command, state).await?;
    Ok(Json(result))
}

/// Internal command execution
async fn execute_command_internal(
    userid: String,
    command: String,
    state: HttpApiState,
) -> Result<CommandResponse, ApiError> {
    // Get or create virtual connection
    let vconn = state.virtual_conns.write().await.get_or_create(&userid).await
        .map_err(|e| ApiError::Internal(e))?;

    // Execute command via command queue
    let output = state.cmd_queue.write().await
        .enqueue_and_wait(userid, command, vconn)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(CommandResponse {
        status: "success".to_string(),
        output,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Command request
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub userid: String,
    pub command: String,
}

/// Command response
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub status: String,
    pub output: String,
    pub timestamp: i64,
}

/// Get server status
pub async fn get_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "running",
        "version": "0.1.0",
        "engine": "RustMUD"
    }))
}

/// API error type
#[derive(Debug)]
pub enum ApiError {
    AuthFailed,
    UserNotFound,
    CommandError,
    Internal(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg): (u16, String) = match self {
            ApiError::AuthFailed => (401, "Authentication failed".to_string()),
            ApiError::UserNotFound => (404, "User not found".to_string()),
            ApiError::CommandError => (400, "Command error".to_string()),
            ApiError::Internal(e) => (500, e.to_string()),
        };
        (axum::http::StatusCode::from_u16(status).unwrap(), msg).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::AuthFailed => write!(f, "Authentication failed"),
            ApiError::UserNotFound => write!(f, "User not found"),
            ApiError::CommandError => write!(f, "Command error"),
            ApiError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<String> for ApiError {
    fn from(s: String) -> Self {
        ApiError::Internal(s)
    }
}

/// Home page
pub async fn index() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>RustMUD</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>RustMUD - Rust MUD Engine</h1>
    <p>1:1 Port of txpike9</p>
    <p>WebSocket: ws://localhost:8080/ws</p>
</body>
</html>
"#)
}

/// Static files handler
pub async fn static_files(
    axum::extract::Path(_path): axum::extract::Path<String>
) -> impl axum::response::IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "Not found")
}

/// Get user info
pub async fn get_user_info(
    axum::extract::Path(userid): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "userid": userid,
        "name": "Player"
    })))
}
