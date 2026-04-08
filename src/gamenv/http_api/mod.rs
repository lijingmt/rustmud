// gamenv/http_api/mod.rs - HTTP API 模块
// 对应 txpike9/gamenv/single/daemons/http_api/

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
pub use utils::*;
pub use handlers::*;
pub use thread_manager::*;

use axum::{
    extract::{State, WebSocketUpgrade, ws::Message},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// HTTP API 状态
#[derive(Clone)]
pub struct HttpApiState {
    pub virtual_conns: Arc<RwLock<VirtualConnectionPool>>,
    pub cmd_queue: Arc<RwLock<CommandQueue>>,
    pub config: Arc<HttpApiConfig>,
}

/// HTTP API 主路由
pub fn create_router() -> Router {
    let state = HttpApiState {
        virtual_conns: Arc::new(RwLock::new(VirtualConnectionPool::new())),
        cmd_queue: Arc::new(RwLock::new(CommandQueue::new())),
        config: Arc::new(HttpApiConfig::default()),
    };

    Router::new()
        // WebSocket 连接
        .route("/ws", get(ws_handler))
        // REST API
        .route("/api/command", post(execute_command))
        .route("/api/status", get(get_status))
        .route("/api/user/:userid", get(get_user_info))
        // 静态文件
        .route("/static/*path", get(static_files))
        // 主页
        .route("/", get(index))
        .with_state(state)
}

/// WebSocket 处理器 (对应 Pike 的 WebSocket 处理)
pub async fn ws_handler(
    State(state): State<HttpApiState>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// WebSocket 连接处理
async fn handle_socket(
    mut socket: axum::extract::ws::WebSocket,
    state: HttpApiState,
) {
    tracing::info!("WebSocket connection established");

    while let Some(result) = socket.recv().await {
        match result {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    if let Err(e) = handle_ws_message(text, &state, &mut socket).await {
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

/// 处理 WebSocket 消息
async fn handle_ws_message(
    msg: String,
    state: &HttpApiState,
    socket: &mut axum::extract::ws::WebSocket,
) -> Result<(), Box<dyn std::error::Error>> {
    // 解析消息格式: {"action":"command","userid":"xxx","cmd":"look"}
    let req: WsRequest = serde_json::from_str(&msg)?;

    match req.action.as_str() {
        "command" => {
            // 执行命令
            let result = execute_command_internal(
                req.userid.unwrap_or_default(),
                req.cmd.unwrap_or_default(),
                state,
            ).await?;

            socket.send(Message::Text(serde_json::to_string(&result)?)).await?;
        }
        "auth" => {
            // 认证
            let auth_result = handle_auth(req.txd.unwrap_or_default(), state).await?;
            socket.send(Message::Text(serde_json::to_string(&auth_result)?)).await?;
        }
        _ => {}
    }

    Ok(())
}

/// WebSocket 请求格式
#[derive(Debug, Deserialize)]
struct WsRequest {
    action: String,
    userid: Option<String>,
    cmd: Option<String>,
    txd: Option<String>,
}

/// WebSocket 响应格式
#[derive(Debug, Serialize)]
struct WsResponse {
    status: String,
    data: Option<serde_json::Value>,
    output: Option<String>,
}

/// Execute command API
pub async fn execute_command(
    State(state): State<HttpApiState>,
    Json(req): CommandRequest,
) -> Result<Json<CommandResponse>, ApiError> {
    let result = execute_command_internal(req.userid, req.command, state).await?;
    Ok(Json(result))
}

/// 内部命令执行
async fn execute_command_internal(
    userid: String,
    command: String,
    state: HttpApiState,
) -> Result<CommandResponse, ApiError> {
    // 获取或创建虚拟连接
    let vconn = state.virtual_conns.write().await.get_or_create(&userid).await?;

    // 执行命令 (通过命令队列)
    let output = state.cmd_queue.write().await
        .enqueue_and_wait(userid, command, vconn)
        .await?;

    Ok(CommandResponse {
        status: "success".to_string(),
        output,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// 命令请求
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    userid: String,
    command: String,
}

/// 命令响应
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    status: String,
    output: String,
    timestamp: i64,
}

/// 获取状态
pub async fn get_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "running",
        "version": "0.1.0",
        "engine": "RustMUD"
    }))
}

/// API 错误类型
#[derive(Debug)]
pub enum ApiError {
    AuthFailed,
    UserNotFound,
    CommandError,
    Internal(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ApiError::AuthFailed => (401, "Authentication failed"),
            ApiError::UserNotFound => (404, "User not found"),
            ApiError::CommandError => (400, "Command error"),
            ApiError::Internal(e) => (500, e.as_str()),
        };
        (status, msg).into_response()
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

/// Home page
pub async fn index() -> Html<&'static str> {
    Html(include_str!("../web/templates/index.html"))
}

/// 静态文件处理
pub async fn static_files(axum::extract::Path(path): axum::extract::Path<String>) -> impl axum::response::IntoResponse {
    // TODO: 实现静态文件服务
    (axum::http::StatusCode::NOT_FOUND, "Not found")
}

/// 获取用户信息
pub async fn get_user_info(
    axum::extract::Path(userid): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: 实现用户信息查询
    Ok(Json(serde_json::json!({
        "userid": userid,
        "name": "Player"
    })))
}
