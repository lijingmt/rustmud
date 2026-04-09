// gamenv/http_api/mod.rs - HTTP API module
// Corresponds to txpike9/gamenv/single/daemons/http_api/

pub mod auth;
pub mod virtual_conn;
pub mod command_queue;
pub mod config;
pub mod utils;
pub mod handlers;
pub mod thread_manager;
pub mod mud_output;
pub mod commands;

pub use auth::*;
pub use virtual_conn::*;
pub use command_queue::*;
pub use config::*;
pub use mud_output::*;
pub use commands::*;
pub use utils::{TextPart, parse_color_codes, parse_color_codes_to_parts};

use axum::{
    extract::{State, WebSocketUpgrade, ws::Message, Query},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};
use tower::ServiceBuilder;

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

    // CORS layer - allow requests from any origin
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // WebSocket connection
        .route("/ws", get(ws_handler))
        // REST API - Vue frontend compatibility
        .route("/api", get(api_get_handler))  // For Vue frontend (txd+cmd query params)
        .route("/api/json", get(api_get_handler))  // Alternative path for compatibility
        .route("/api/partitions", get(get_partitions))  // Get game partitions list
        // REST API - Internal endpoints
        .route("/api/command", post(execute_command))
        .route("/api/invite/seturl", post(save_invite_url))  // Invite URL tracking
        .route("/api/status", get(get_status))
        .route("/api/user/{userid}", get(get_user_info))
        // Static files
        .route("/static/{*path}", get(static_files))
        // Home page
        .route("/", get(index))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(cors))
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
            let userid = req.userid.clone().unwrap_or_default();
            let cmd = req.cmd.clone().unwrap_or_default();

            let command_result = execute_command_internal(
                userid.clone(),
                cmd.clone(),
                state.clone(),
            ).await?;

            // 构建增强的 JSON 响应
            let response = build_game_response(&command_result.output, &userid, &cmd).await;

            socket.send(Message::Text(response.to_string().into())).await?;
        }
        "auth" => {
            let auth_result = handle_auth(req.txd.unwrap_or_default(), &state).await?;
            let json_str = serde_json::to_string(&auth_result)?;
            socket.send(Message::Text(json_str.into())).await?;
        }
        "poll" => {
            // 轮询房间状态更新
            let userid = req.userid.unwrap_or_default();
            let response = build_room_update(&userid).await;
            socket.send(Message::Text(response.to_string().into())).await?;
        }
        _ => {}
    }

    Ok(())
}

/// 构建房间状态更新（用于实时刷新）
async fn build_room_update(userid: &str) -> serde_json::Value {
    use crate::gamenv::world::get_world;
    use crate::gamenv::http_api::mud_output::{MudOutputParser, RoomData, NpcInfo};

    let mut parser = MudOutputParser::new();

    // 获取玩家当前房间
    let player_mgr = crate::gamenv::player_state::get_player_manager();
    let mgr = player_mgr.read().await;

    let (current_room_id, player_stats) = if let Some(player) = mgr.get(userid).await {
        let state = player.read().await;
        (
            state.current_room.clone(),
            Some(serde_json::json!({
                "hp": state.hp,
                "hp_max": state.hp_max,
                "spirit": state.mp,
                "spirit_max": state.mp_max,
                "potential": 0,
                "potential_max": 100,
                "neili": 0,
                "neili_max": 100,
                "exp": state.exp,
                "level": state.level,
                "name_cn": state.name,
                "autofight": false
            }))
        )
    } else {
        ("xinshoucun/xinshoucunguangchang".to_string(), None)
    };

    let world_arc = get_world();
    let world_guard = world_arc.read().await;
    let room = world_guard.get_room(&current_room_id);

    // 构建房间数据
    let room_info = if let Some(room) = &room {
        let mut npcs = vec![];
        for npc_id in &room.npcs {
            if let Some(npc) = world_guard.get_npc(npc_id) {
                npcs.push(NpcInfo {
                    id: npc.id.clone(),
                    name: npc.name.clone(),
                    short: npc.short.clone(),
                });
            }
        }

        let exits: Vec<String> = room.exits.keys().cloned().collect();

        // 构建带目标房间名称的出口信息
        let mut exits_with_names = vec![];
        for (direction, target_room_id) in &room.exits {
            let target_room = world_guard.get_room(target_room_id);
            let target_room_name = target_room.map(|r| r.name.clone()).unwrap_or_else(|| "未知".to_string());

            let (direction_cn, arrow) = match direction.as_str() {
                "north" => ("北".to_string(), "↑".to_string()),
                "south" => ("南".to_string(), "↓".to_string()),
                "east" => ("东".to_string(), "→".to_string()),
                "west" => ("西".to_string(), "←".to_string()),
                "up" => ("上".to_string(), "↑".to_string()),
                "down" => ("下".to_string(), "↓".to_string()),
                "northeast" => ("东北".to_string(), "↗".to_string()),
                "northwest" => ("西北".to_string(), "↖".to_string()),
                "southeast" => ("东南".to_string(), "↘".to_string()),
                "southwest" => ("西南".to_string(), "↙".to_string()),
                _ => (direction.clone(), "".to_string()),
            };

            exits_with_names.push(ExitInfo {
                direction: direction.clone(),
                direction_cn,
                arrow,
                target_room: target_room_name,
            });
        }

        Some(RoomData {
            id: room.id.clone(),
            name: room.name.clone(),
            short: room.short.clone(),
            long: room.long.clone(),
            npcs,
            exits,
            exits_with_names,
        })
    } else {
        None
    };

    // 更新解析器并生成房间 JSON
    if let Some(ref room_data) = room_info {
        parser.update_room(room_data);
    }

    let mud_lines = parser.generate_room_json();

    serde_json::json!({
        "status": "success",
        "action": "room_update",
        "lines": mud_lines,
        "room_info": room_info,
        "player_stats": player_stats
    })
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
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mud_lines: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    room_info: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    player_stats: Option<serde_json::Value>,
}

/// API GET handler for Vue frontend compatibility
/// Handles requests like: /api?txd=xxx&cmd=look or /api?userid=xxx&password=xxx&cmd=look
pub async fn api_get_handler(
    State(state): State<HttpApiState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    // Extract parameters manually
    let txd = params.get("txd").map(|s| s.as_str());
    let userid = params.get("userid").map(|s| s.as_str());
    let _password = params.get("password").map(|s| s.as_str());
    let cmd = params.get("cmd").map(|s| s.as_str()).unwrap_or("look");

    // Try multiple authentication methods
    let userid = if let Some(txd_val) = txd {
        // Method 1: Decode TXD to get userid
        let auth_mgr = crate::gamenv::http_api::auth::get_auth_manager();
        match auth_mgr.lock() {
            Ok(mgr) => mgr.decode_txd(txd_val).map(|d| d.userid),
            Err(_) => None,
        }
    } else if let Some(uid) = userid {
        // Method 2: Direct userid+password authentication
        // TODO: Validate password against database
        Some(uid.to_string())
    } else {
        None
    };

    let userid = match userid {
        Some(u) => u,
        None => return Json(serde_json::json!({"error": "Authentication failed", "status": "error"})),
    };

    // Execute command
    let result = match execute_command_internal(userid.clone(), cmd.to_string(), state).await {
        Ok(r) => r,
        Err(_) => return Json(serde_json::json!({"error": "Command failed"})),
    };

    // Parse the output to extract game state
    let response = build_game_response(&result.output, &userid, cmd).await;

    Json(response)
}

/// Build game response in format expected by Vue frontend
async fn build_game_response(output: &str, userid: &str, command: &str) -> serde_json::Value {
    use crate::gamenv::world::get_world;
    use crate::gamenv::http_api::mud_output::{MudOutputParser, RoomData, NpcInfo};

    let mut parser = MudOutputParser::new();

    // 获取当前玩家房间
    let player_mgr = crate::gamenv::player_state::get_player_manager();
    let mgr = player_mgr.read().await;

    let (current_room_id, player_stats) = if let Some(player) = mgr.get(userid).await {
        let state = player.read().await;
        (
            state.current_room.clone(),
            Some(serde_json::json!({
                "hp": state.hp,
                "hp_max": state.hp_max,
                "spirit": state.mp,
                "spirit_max": state.mp_max,
                "potential": 0,
                "potential_max": 100,
                "neili": 0,
                "neili_max": 100,
                "exp": state.exp,
                "level": state.level,
                "name_cn": state.name,
                "autofight": false
            }))
        )
    } else {
        ("xinshoucun/xinshoucunguangchang".to_string(), None)
    };

    let world_arc = get_world();
    let world_guard = world_arc.read().await;
    let room = world_guard.get_room(&current_room_id);

    // 构建房间数据
    let room_info = if let Some(room) = &room {
        let mut npcs = vec![];
        for npc_id in &room.npcs {
            if let Some(npc) = world_guard.get_npc(npc_id) {
                npcs.push(NpcInfo {
                    id: npc.id.clone(),
                    name: npc.name.clone(),
                    short: npc.short.clone(),
                });
            }
        }

        let exits: Vec<String> = room.exits.keys().cloned().collect();

        // 构建带目标房间名称的出口信息
        let mut exits_with_names = vec![];
        for (direction, target_room_id) in &room.exits {
            let target_room = world_guard.get_room(target_room_id);
            let target_room_name = target_room.map(|r| r.name.clone()).unwrap_or_else(|| "未知".to_string());

            let (direction_cn, arrow) = match direction.as_str() {
                "north" => ("北".to_string(), "↑".to_string()),
                "south" => ("南".to_string(), "↓".to_string()),
                "east" => ("东".to_string(), "→".to_string()),
                "west" => ("西".to_string(), "←".to_string()),
                "up" => ("上".to_string(), "↑".to_string()),
                "down" => ("下".to_string(), "↓".to_string()),
                "northeast" => ("东北".to_string(), "↗".to_string()),
                "northwest" => ("西北".to_string(), "↖".to_string()),
                "southeast" => ("东南".to_string(), "↘".to_string()),
                "southwest" => ("西南".to_string(), "↙".to_string()),
                _ => (direction.clone(), "".to_string()),
            };

            exits_with_names.push(ExitInfo {
                direction: direction.clone(),
                direction_cn,
                arrow,
                target_room: target_room_name,
            });
        }

        Some(RoomData {
            id: room.id.clone(),
            name: room.name.clone(),
            short: room.short.clone(),
            long: room.long.clone(),
            npcs,
            exits,
            exits_with_names,
        })
    } else {
        None
    };

    // 更新解析器的房间信息
    if let Some(ref room_data) = room_info {
        parser.update_room(room_data);
    }

    // 解析输出为 mud_lines
    // 方向命令和 look 命令都使用完整的房间渲染
    let is_direction_command = matches!(command, "north" | "south" | "east" | "west" | "up" | "down" | "n" | "s" | "e" | "w" | "u" | "d");
    let mud_lines = if command == "look" || command == "l" || is_direction_command {
        // look 命令和方向命令使用完整的房间渲染
        parser.generate_room_json()
    } else {
        // 其他命令解析输出
        parser.parse_output(output)
    };

    // 构建导航按钮数据
    let navigation = if let Some(ref room_data) = room_info {
        // 使用 exits_with_names 构建导航按钮
        let mut exits = vec![];
        tracing::info!("Building navigation, exits_with_names count: {}", room_data.exits_with_names.len());
        for exit in &room_data.exits_with_names {
            exits.push(serde_json::json!({
                "direction": exit.direction,
                "label": format!("{}：{}", exit.direction_cn, exit.target_room),
                "command": format!("go {}", exit.direction)
            }));
        }
        tracing::info!("Navigation built: {} buttons", exits.len());
        // 返回符合 dist 前端期望的格式: {exits: [...]}
        serde_json::json!({"exits": exits})
    } else {
        tracing::info!("No room_info, navigation empty");
        serde_json::json!({"exits": []})
    };

    // 构建消息类型（基于命令）
    let msg_type = match command {
        "kill" | "attack" => "combat",
        "talk" => "system",
        _ => "info"
    };

    let response = serde_json::json!({
        "status": "success",
        "lines": mud_lines,
        "room_info": room_info,
        "player_stats": player_stats,
        "state": {
            "player": {
                "name": userid,
                "level": 1,
                "hp": 100,
                "hpMax": 100,
                "hpPercent": 100,
                "mp": 50,
                "mpMax": 50,
                "mpPercent": 100,
                "exp": 0,
                "expPercent": 0
            },
            "messages": [{
                "id": chrono::Utc::now().timestamp_millis(),
                "type": msg_type,
                "text": output
            }],
            "actions": [
                {"id": "look", "label": "查看", "command": "look", "style": "primary"},
                {"id": "inventory", "label": "背包", "command": "inventory", "style": "default"},
                {"id": "score", "label": "状态", "command": "score", "style": "default"},
                {"id": "skills", "label": "技能", "command": "skills", "style": "default"}
            ],
            "navigation": navigation
        }
    });

    // 打印完整响应到控制台
    println!("=== API Response for command '{}' ===", command);
    println!("Navigation: {}", serde_json::to_string_pretty(&navigation).unwrap_or_default());
    println!("=== End Response ===");

    response
}

/// 获取方向中文标签
fn get_direction_label(dir: &str) -> &str {
    match dir {
        "north" => "北方",
        "south" => "南方",
        "east" => "东方",
        "west" => "西方",
        "up" => "上方",
        "down" => "下方",
        "northeast" => "东北",
        "northwest" => "西北",
        "southeast" => "东南",
        "southwest" => "西南",
        _ => dir,
    }
}

/// API GET parameters
#[derive(Debug, Deserialize)]
#[serde(default)]
struct ApiGetParams {
    txd: Option<String>,
    userid: Option<String>,
    password: Option<String>,
    cmd: String,
}

impl Default for ApiGetParams {
    fn default() -> Self {
        Self {
            txd: None,
            userid: None,
            password: None,
            cmd: "look".to_string(),
        }
    }
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
    tracing::info!("Executing command '{}' for user '{}'", command, userid);

    // Get or create virtual connection
    let mut vconn = state.virtual_conns.write().await.get_or_create(&userid).await
        .map_err(|e| ApiError::Internal(e))?;

    // Directly execute command (simplified for now)
    // TODO: Integrate with full game command processor
    let output = execute_game_command(&userid, &command, &vconn).await;

    // Update connection time
    state.virtual_conns.write().await.update_time(&userid).await;

    Ok(CommandResponse {
        status: "success".to_string(),
        output,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Execute game command (integrated with world system and player state)
async fn execute_game_command(userid: &str, command: &str, _vconn: &VirtualConnection) -> String {
    use crate::gamenv::world::get_world;
    use crate::gamenv::player_state::get_player_manager;
    use crate::gamenv::single::daemons::pkd::PKD;

    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    let cmd = parts.get(0).unwrap_or(&"").to_lowercase();
    let args = &parts[1..];

    // 获取或创建玩家状态
    let player_mgr = get_player_manager();
    let mut player_mgr_write = player_mgr.write().await;
    let player_state = player_mgr_write.get_or_create(userid.to_string()).await;
    drop(player_mgr_write); // 释放锁

    // 检查是否在战斗中 - 战斗锁定机制
    let in_battle = PKD.get_player_battle(userid).await.is_some();
    if in_battle {
        // 战斗中只允许以下命令
        let allowed_commands = ["pk", "escape", "surrender", "look"];
        if !allowed_commands.contains(&cmd.as_str()) {
            return "§R战斗中无法执行此操作！§N\n\n\
                   §Y【当前战斗】§N\n\
                   输入「pk continue」继续战斗\n\
                   输入「escape」逃跑\n\
                   输入「surrender」投降\n\
                   输入「look」查看战斗状态\n\
                   ────────────────────────────\n\
                   战斗结束后才能进行其他操作。".to_string();
        }

        // 如果是 pk 命令，必须是 "pk continue"
        if cmd == "pk" && args.len() > 0 && args[0] != "continue" {
            return "§R战斗中只能执行「pk continue」继续战斗！§N\n\
                   输入「escape」逃跑\n\
                   输入「surrender」投降".to_string();
        }
    }

    // 对于需要 world 的命令，先获取当前房间
    let player_room = {
        let state = player_state.read().await;
        state.current_room.clone()
    };

    // 先获取 world 的 Arc，避免临时值问题
    let world_arc = get_world();

    match cmd.as_str() {
        "look" | "l" => {
            // 战斗中查看战斗状态
            if in_battle {
                if let Some(battle) = PKD.get_player_battle(userid).await {
                    if battle.status == crate::gamenv::single::daemons::pkd::CombatStatus::Fighting {
                        return battle.generate_status();
                    } else {
                        // 战斗已结束，清理并显示结果
                        let result = battle.generate_result();
                        PKD.end_battle(&battle.battle_id).await;
                        return result;
                    }
                }
            }

            let world_guard = world_arc.read().await;
            // 检查是否有参数（查看NPC）
            if !args.is_empty() {
                let target = args.join(" ");
                // 尝试在当前房间查找NPC
                look_npc_command(&world_guard, &player_room, &target).await
            } else {
                // 查看房间
                look_command(&world_guard, &player_room).await
            }
        }
        "north" | "n" => {
            let world_guard = world_arc.read().await;
            let result = move_command(&world_guard, &player_room, "north").await;
            if result.success {
                let mut state = player_state.write().await;
                state.move_to(result.new_room.clone());
            }
            result.output
        }
        "south" | "s" => {
            let world_guard = world_arc.read().await;
            let result = move_command(&world_guard, &player_room, "south").await;
            if result.success {
                let mut state = player_state.write().await;
                state.move_to(result.new_room.clone());
            }
            result.output
        }
        "east" | "e" => {
            let world_guard = world_arc.read().await;
            let result = move_command(&world_guard, &player_room, "east").await;
            if result.success {
                let mut state = player_state.write().await;
                state.move_to(result.new_room.clone());
            }
            result.output
        }
        "west" | "w" => {
            let world_guard = world_arc.read().await;
            let result = move_command(&world_guard, &player_room, "west").await;
            if result.success {
                let mut state = player_state.write().await;
                state.move_to(result.new_room.clone());
            }
            result.output
        }
        "up" | "u" => {
            let world_guard = world_arc.read().await;
            let result = move_command(&world_guard, &player_room, "up").await;
            if result.success {
                let mut state = player_state.write().await;
                state.move_to(result.new_room.clone());
            }
            result.output
        }
        "down" | "d" => {
            let world_guard = world_arc.read().await;
            let result = move_command(&world_guard, &player_room, "down").await;
            if result.success {
                let mut state = player_state.write().await;
                state.move_to(result.new_room.clone());
            }
            result.output
        }
        "inventory" | "i" | "inv" => {
            let state = player_state.read().await;
            inventory_command(&state).await
        }
        "score" => {
            let state = player_state.read().await;
            state.format_score()
        }
        "who" => {
            who_command(userid).await
        }
        "skills" => {
            "你的技能:\n  [基础] 基础剑术 - Lv.1\n  [基础] 基础防御 - Lv.1".to_string()
        }
        "talk" | "ask" => {
            if args.is_empty() {
                "你要和谁说话？".to_string()
            } else {
                let target = args.join(" ");
                let world_guard = world_arc.read().await;
                talk_command(&world_guard, &player_room, &target).await
            }
        }
        "kill" | "attack" => {
            if args.is_empty() {
                "你要攻击谁？".to_string()
            } else {
                let target = args.join(" ");
                let world_guard = world_arc.read().await;
                kill_command(&world_guard, &player_room, &target).await
            }
        }
        "pk" => {
            if args.is_empty() {
                "你要和谁PK？".to_string()
            } else {
                let target = args.join(" ");
                // 检查是否是 "pk continue" 命令
                if target == "continue" {
                    pk_continue_command(userid).await
                } else {
                    let world_guard = world_arc.read().await;
                    pk_command(&world_guard, &player_room, userid, &target).await
                }
            }
        }
        "escape" => {
            escape_command(userid).await
        }
        "surrender" => {
            surrender_command(userid).await
        }
        cmd if cmd.starts_with("say") => {
            let msg = args.join(" ");
            format!("{}说: {}", userid, msg)
        }
        "help" => {
            // 使用命令注册表生成帮助文本
            COMMAND_REGISTRY.lock().unwrap().help_text()
        }
        "equipment" | "eq" => {
            "§Y当前装备:§N\n\n§H武器:§N 新手木剑 (攻击+5)\n§H衣服:§N 新手布衣 (防御+3)\n§H饰品:§N 无".to_string()
        }
        "use" => {
            if args.is_empty() {
                "使用什么物品？".to_string()
            } else {
                let item = args.join(" ");
                use_item_command(&item, userid)
            }
        }
        "equip" => {
            if args.is_empty() {
                "装备什么物品？".to_string()
            } else {
                let item = args.join(" ");
                equip_item_command(&item, userid)
            }
        }
        "save" => {
            // TODO: 实现保存
            format!("§G游戏进度已保存！§N\n")
        }
        "tell" => {
            if args.len() < 2 {
                "格式: tell <玩家名> <消息>".to_string()
            } else {
                let target = args[0];
                let msg = args[1..].join(" ");
                format!("你悄悄对{}说: {}", target, msg)
            }
        }
        "rest" => {
            let mut state = player_state.write().await;
            let hp_max = state.hp_max;
            let mp_max = state.mp_max;
            state.heal(hp_max);
            state.restore_mp(mp_max);
            format!("你休息了一会儿，体力恢复了！\n§HHP: {}/{}  MP: {}/{}§N\n",
                state.hp, state.hp_max, state.mp, state.mp_max)
        }
        _ => {
            format!("§R未知命令: {}§N\n输入「help」查看可用命令。", cmd)
        }
    }
}

/// 查看命令
async fn look_command(world: &crate::gamenv::world::GameWorld, room_id: &str) -> String {
    tracing::info!("look_command called with room_id: '{}'", room_id);
    tracing::info!("World has {} rooms loaded", world.room_count());

    if let Some(room) = world.get_room(room_id) {
        let mut output = format!("§Y{}§N\n", room.name);
        output.push_str(&format!("{}\n", room.long.trim()));

        // 显示NPC
        if !room.npcs.is_empty() {
            output.push_str("\n这里有以下人物：\n");
            for npc_id in &room.npcs {
                if let Some(npc) = world.get_npc(npc_id) {
                    output.push_str(&format!("  {}\n", npc.format_short()));
                }
            }
        }

        // 显示怪物
        if !room.monsters.is_empty() {
            output.push_str("\n§R这里有危险的生物：§N\n");
            for monster_id in &room.monsters {
                if let Some(monster) = world.get_npc(monster_id) {
                    output.push_str(&format!("  §R[怪物]{}§N\n", monster.short));
                }
            }
        }

        // 显示出口
        output.push_str(&format!("\n§H明显的出口: {}§N\n", room.format_exits()));

        output
    } else {
        "无法找到当前房间。".to_string()
    }
}

/// 查看NPC详情命令
async fn look_npc_command(world: &crate::gamenv::world::GameWorld, room_id: &str, target: &str) -> String {
    tracing::info!("look_npc_command called with target: '{}'", target);

    let mut output = String::new();
    let mut found_npc = false;

    if let Some(room) = world.get_room(room_id) {
        // 先检查房间内的NPC
        for npc_id in &room.npcs {
            if let Some(npc) = world.get_npc(npc_id) {
                if npc.id.contains(target) || npc.name.contains(target) || npc.short.contains(target) {
                    found_npc = true;
                    output.push_str(&format!("§Y========== {} =========§N\n", npc.name));
                    output.push_str(&format!("§C【等级】§N {}\n", npc.level));
                    output.push_str(&format!("§C【描述】§N {}\n", npc.long));

                    // 根据NPC类型显示不同的操作按钮
                    output.push_str(&format!("\n§H【操作】§N\n"));

                    // 对话按钮
                    output.push_str(&format!("[对话:talk {}]\n", npc.id));

                    // 商店按钮（如果是商人）
                    if npc.shop.is_some() {
                        output.push_str(&format!("[商店:shop {}]\n", npc.id));
                    }

                    // 攻击按钮（如果是怪物或敌对NPC）
                    if room.monsters.contains(&npc.id) {
                        output.push_str(&format!("[§R攻击§N:kill {}]\n", npc.id));
                    }

                    // PK按钮 - 对玩家或NPC进行PK
                    output.push_str(&format!("[§RPK§N:pk {}]\n", npc.id));

                    // 任务按钮
                    output.push_str(&format!("[任务:quest {}]\n", npc.id));

                    output.push_str(&format!("\n§H========== 返回 =========§N\n"));
                    output.push_str("[返回房间:look]\n");
                    break;
                }
            }
        }

        // 如果NPC列表中没找到，检查怪物列表
        if !found_npc {
            for monster_id in &room.monsters {
                if let Some(monster) = world.get_npc(monster_id) {
                    if monster.id.contains(target) || monster.name.contains(target) || monster.short.contains(target) {
                        found_npc = true;
                        output.push_str(&format!("§Y========== {} =========§N\n", monster.name));
                        output.push_str(&format!("§C【等级】§N {}\n", monster.level));
                        output.push_str(&format!("§C【生命值】§N {}/{}\n", monster.hp, monster.hp_max));
                        output.push_str(&format!("§C【描述】§N {}\n", monster.long));

                        // 怪物只有攻击选项
                        output.push_str(&format!("\n§H【操作】§N\n"));
                        output.push_str(&format!("[§R攻击§N:kill {}]\n", monster.id));

                        output.push_str(&format!("\n§H========== 返回 =========§N\n"));
                        output.push_str("[返回房间:look]\n");
                        break;
                    }
                }
            }
        }
    }

    if found_npc {
        output
    } else {
        format!("§R这里没有叫做「{}」的生物。§N\n[返回:look]", target)
    }
}

/// 移动命令结果
struct MoveResult {
    success: bool,
    new_room: String,
    output: String,
}

/// 移动命令
async fn move_command(
    world: &crate::gamenv::world::GameWorld,
    current_room: &str,
    direction: &str,
) -> MoveResult {
    if let Some(room) = world.get_room(current_room) {
        if let Some(exit_room_id) = room.get_exit(direction) {
            if let Some(exit_room) = world.get_room(exit_room_id) {
                let dir_name = get_direction_name(direction);
                let output = format!(
                    "你向{}走...\n\n§Y{}§N\n{}\n\n§H明显的出口: {}§N",
                    dir_name,
                    exit_room.name,
                    exit_room.long.trim(),
                    exit_room.format_exits()
                );
                MoveResult {
                    success: true,
                    new_room: exit_room_id.clone(),
                    output,
                }
            } else {
                MoveResult {
                    success: false,
                    new_room: current_room.to_string(),
                    output: "那个方向出错了！请联系管理员。".to_string(),
                }
            }
        } else {
            MoveResult {
                success: false,
                new_room: current_room.to_string(),
                output: "这个方向没有出口。".to_string(),
            }
        }
    } else {
        MoveResult {
            success: false,
            new_room: current_room.to_string(),
            output: "你迷失在了空间裂缝中...".to_string(),
        }
    }
}

/// 背包命令
async fn inventory_command(state: &crate::gamenv::player_state::PlayerState) -> String {
    if state.inventory.is_empty() {
        "你身上带着:\n  没有任何东西。".to_string()
    } else {
        let mut output = "你身上带着:\n".to_string();
        for (item_id, (_, count, _)) in &state.inventory {
            output.push_str(&format!("  {} x{}\n", item_id, count));
        }
        output
    }
}

/// Who命令
async fn who_command(userid: &str) -> String {
    use crate::gamenv::player_state::get_player_manager;
    let player_mgr = get_player_manager().read().await;
    let players = player_mgr.get_online_players().await;
    drop(player_mgr);

    if players.is_empty() {
        format!("在线玩家:\n  {}\n\n共1人在线", userid)
    } else {
        format!("在线玩家:\n  {}\n\n共{}人在线", userid, players.len())
    }
}

/// 对话命令
async fn talk_command(
    world: &crate::gamenv::world::GameWorld,
    room_id: &str,
    target: &str,
) -> String {
    if let Some(room) = world.get_room(room_id) {
        for npc_id in &room.npcs {
            if let Some(npc) = world.get_npc(npc_id) {
                if npc.name.contains(target) || npc.short.contains(target) {
                    return format_npc_dialog(&npc, "");
                }
            }
        }
    }
    format!("这里没有叫做「{}」的人。", target)
}

/// 攻击命令
async fn kill_command(
    world: &crate::gamenv::world::GameWorld,
    room_id: &str,
    target: &str,
) -> String {
    if let Some(room) = world.get_room(room_id) {
        for monster_id in &room.monsters {
            if let Some(monster) = world.get_npc(monster_id) {
                if monster.name.contains(target) || monster.short.contains(target) {
                    return format!("§R你开始攻击{}！§N\n{}", monster.short, combat_round(target, monster));
                }
            }
        }
    }
    format!("这里没有叫做「{}」的生物。", target)
}

/// PK命令 - 玩家对战或强制攻击NPC
async fn pk_command(
    world: &crate::gamenv::world::GameWorld,
    room_id: &str,
    userid: &str,
    target: &str,
) -> String {
    use crate::gamenv::single::daemons::pkd::{PKD, PkMode, CombatStats};

    // 获取玩家数据（用于战斗）
    let player_level = 1;
    let player_hp = 100;
    let player_hp_max = 100;
    let player_attack = 10;
    let player_defense = 5;

    if let Some(room) = world.get_room(room_id) {
        // 先检查NPC - 使用PK战斗系统
        for npc_id in &room.npcs {
            if let Some(npc) = world.get_npc(npc_id) {
                if npc.id.contains(target) || npc.name.contains(target) || npc.short.contains(target) {
                    // 构建挑战者数据
                    let challenger_stats = CombatStats {
                        id: userid.to_string(),
                        name: userid.to_string(),
                        name_cn: format!("玩家{}", &userid[..userid.len().min(3)]),
                        level: player_level,
                        hp: player_hp,
                        hp_max: player_hp_max,
                        mp: 50,
                        mp_max: 50,
                        jing: 100,
                        jing_max: 100,
                        qi: 50,
                        qi_max: 50,
                        attack: player_attack,
                        defense: player_defense,
                        dodge: 8,
                        parry: 6,
                        pk_mode: PkMode::Free,
                        pk_value: 0,
                        kill_streak: 0,
                        is_killing: false,
                    };

                    // 构建防守者（NPC）数据
                    let defender_stats = CombatStats {
                        id: npc.id.clone(),
                        name: npc.name.clone(),
                        name_cn: npc.short.clone(),
                        level: npc.level,
                        hp: npc.hp,
                        hp_max: npc.hp_max,
                        mp: 0,
                        mp_max: 0,
                        jing: 100,
                        jing_max: 100,
                        qi: 0,
                        qi_max: 0,
                        attack: 5,
                        defense: 3,
                        dodge: 5,
                        parry: 4,
                        pk_mode: PkMode::Free,  // NPC可以被攻击
                        pk_value: 0,
                        kill_streak: 0,
                        is_killing: false,
                    };

                    // 发起战斗
                    return match PKD.challenge(challenger_stats, defender_stats).await {
                        Ok(battle) => {
                            battle.generate_status()
                        }
                        Err(e) => {
                            format!("§R无法发起战斗: {}§N\n[返回:look]", e)
                        }
                    };
                }
            }
        }

        // 检查怪物 - 使用战斗系统
        for monster_id in &room.monsters {
            if let Some(monster) = world.get_npc(monster_id) {
                if monster.id.contains(target) || monster.name.contains(target) || monster.short.contains(target) {
                    // 构建挑战者数据
                    let challenger_stats = CombatStats {
                        id: userid.to_string(),
                        name: userid.to_string(),
                        name_cn: format!("玩家{}", &userid[..userid.len().min(3)]),
                        level: player_level,
                        hp: player_hp,
                        hp_max: player_hp_max,
                        mp: 50,
                        mp_max: 50,
                        jing: 100,
                        jing_max: 100,
                        qi: 50,
                        qi_max: 50,
                        attack: player_attack,
                        defense: player_defense,
                        dodge: 8,
                        parry: 6,
                        pk_mode: PkMode::Free,
                        pk_value: 0,
                        kill_streak: 0,
                        is_killing: false,
                    };

                    // 构建防守者（怪物）数据
                    let defender_stats = CombatStats {
                        id: monster.id.clone(),
                        name: monster.name.clone(),
                        name_cn: monster.short.clone(),
                        level: monster.level,
                        hp: monster.hp,
                        hp_max: monster.hp_max,
                        mp: 0,
                        mp_max: 0,
                        jing: 100,
                        jing_max: 100,
                        qi: 0,
                        qi_max: 0,
                        attack: 8,
                        defense: 4,
                        dodge: 6,
                        parry: 3,
                        pk_mode: PkMode::Free,
                        pk_value: 0,
                        kill_streak: 0,
                        is_killing: true,
                    };

                    // 发起战斗
                    return match PKD.challenge(challenger_stats, defender_stats).await {
                        Ok(battle) => {
                            battle.generate_status()
                        }
                        Err(e) => {
                            format!("§R无法发起战斗: {}§N\n[返回:look]", e)
                        }
                    };
                }
            }
        }
    }

    // 尝试玩家PK（使用PK守护进程）
    // 构建挑战者数据
    let challenger_stats = CombatStats {
        id: userid.to_string(),
        name: userid.to_string(),
        name_cn: userid.to_string(),
        level: 1,
        hp: 100,
        hp_max: 100,
        mp: 50,
        mp_max: 50,
        jing: 100,
        jing_max: 100,
        qi: 50,
        qi_max: 50,
        attack: 10,
        defense: 5,
        dodge: 8,
        parry: 6,
        pk_mode: PkMode::Free,
        pk_value: 0,
        kill_streak: 0,
        is_killing: false,
    };

    // 构建防守者数据（从当前房间查找玩家）
    // TODO: 从房间获取其他玩家数据
    let defender_stats = CombatStats {
        id: target.to_string(),
        name: target.to_string(),
        name_cn: target.to_string(),
        level: 1,
        hp: 100,
        hp_max: 100,
        mp: 50,
        mp_max: 50,
        jing: 100,
        jing_max: 100,
        qi: 50,
        qi_max: 50,
        attack: 10,
        defense: 5,
        dodge: 8,
        parry: 6,
        pk_mode: PkMode::Free,
        pk_value: 0,
        kill_streak: 0,
        is_killing: false,
    };

    // 发起PK挑战
    match PKD.challenge(challenger_stats, defender_stats).await {
        Ok(battle) => battle.generate_status(),
        Err(e) => format!("§R{}§N", e),
    }
}

/// 继续PK战斗
async fn pk_continue_command(userid: &str) -> String {
    use crate::gamenv::single::daemons::pkd::PKD;

    match PKD.get_player_battle(userid).await {
        Some(battle) => {
            if battle.status == crate::gamenv::single::daemons::pkd::CombatStatus::Fighting {
                // 执行下一回合
                if let Some(round) = PKD.next_round(&battle.battle_id).await {
                    if round.ended {
                        // 战斗结束，显示结果
                        let result = battle.generate_result();
                        // 清理战斗
                        PKD.end_battle(&battle.battle_id).await;
                        result
                    } else {
                        // 战斗继续：先显示战斗状态（含按钮），再显示战斗日志
                        let mut output = battle.generate_status();

                        // 添加战斗日志到最下面
                        output.push_str("\n────────────────────────────\n");
                        output.push_str("§H【本回合】§N\n");
                        for log in &round.log {
                            output.push_str(log);
                            output.push_str("\n");
                        }

                        output
                    }
                } else {
                    "战斗已结束！\n[返回:look]".to_string()
                }
            } else {
                // 战斗已结束，清理并显示结果
                let result = battle.generate_result();
                PKD.end_battle(&battle.battle_id).await;
                result
            }
        }
        None => {
            "你不在战斗中！\n[返回:look]".to_string()
        }
    }
}

/// 逃跑命令
async fn escape_command(userid: &str) -> String {
    use crate::gamenv::single::daemons::pkd::PKD;

    match PKD.escape(userid).await {
        Ok(msg) => format!("{}\n[返回房间:look]", msg),
        Err(e) => format!("{}\n[继续战斗:pk continue]", e),
    }
}

/// 投降命令
async fn surrender_command(userid: &str) -> String {
    use crate::gamenv::single::daemons::pkd::PKD;

    match PKD.surrender(userid).await {
        Ok(msg) => format!("{}\n[返回房间:look]", msg),
        Err(e) => e,
    }
}

/// 获取方向名称
fn get_direction_name(dir: &str) -> &str {
    match dir {
        "north" => "北",
        "south" => "南",
        "east" => "东",
        "west" => "西",
        "up" => "上",
        "down" => "下",
        "northeast" => "东北",
        "northwest" => "西北",
        "southeast" => "东南",
        "southwest" => "西南",
        _ => dir,
    }
}

/// 格式化NPC对话
fn format_npc_dialog(npc: &crate::gamenv::world::Npc, _node_id: &str) -> String {
    if let Some(dialog) = npc.get_dialog("greeting") {
        let mut result = format!("{}说: {}\n\n", npc.name, dialog.text);
        for (idx, option) in dialog.options.iter().enumerate() {
            result.push_str(&format!("  {}. {}\n", idx + 1, option.text));
        }
        result
    } else {
        format!("{}没有回应你。", npc.name)
    }
}

/// 战斗回合（简化版）
fn combat_round(userid: &str, monster: &crate::gamenv::world::Npc) -> String {
    let player_damage = 15;
    let monster_damage = monster.attack / 2;

    format!(
        "你攻击{}，造成了{}点伤害！\n{}反击，你受到了{}点伤害！\n\n(战斗系统开发中...)",
        monster.short,
        player_damage,
        monster.short,
        monster_damage
    )
}

/// 使用物品命令
fn use_item_command(item: &str, _userid: &str) -> String {
    match item {
        "生命药水" | "小生命药水" => {
            format!("§G你使用了一瓶小生命药水，恢复了30点HP！§N\n")
        }
        "魔法药水" | "小魔法药水" => {
            format!("§G你使用了一瓶小魔法药水，恢复了20点MP！§N\n")
        }
        _ => format!("§R你无法使用这个物品。§N\n")
    }
}

/// 装备物品命令
fn equip_item_command(item: &str, userid: &str) -> String {
    format!("§Y{}§N 装备了 {}。\n", userid, item)
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

/// Get game partitions list
pub async fn get_partitions() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "partitions": [
            {"id": "tx01", "name": "天下01", "online": 10}
        ]
    }))
}

/// Save invite URL (for invite tracking)
pub async fn save_invite_url(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Extract txd and url from query parameters
    let _txd = params.get("txd");
    let url = params.get("url");

    // TODO: Store the invite URL in database for the user
    tracing::info!("Invite URL saved: {:?}", url);

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "URL saved"
    })))
}
