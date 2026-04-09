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

pub use auth::*;
pub use virtual_conn::*;
pub use command_queue::*;
pub use config::*;
pub use mud_output::*;

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
        // REST API - Internal endpoints
        .route("/api/command", post(execute_command))
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
        ("newbie_square".to_string(), None)
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

        Some(RoomData {
            id: room.id.clone(),
            name: room.name.clone(),
            short: room.short.clone(),
            long: room.long.clone(),
            npcs,
            exits,
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
        "mud_lines": mud_lines,
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
/// Handles requests like: /api?txd=xxx&cmd=look
pub async fn api_get_handler(
    State(state): State<HttpApiState>,
    Query(params): Query<ApiGetParams>,
) -> Json<serde_json::Value> {
    // Decode TXD to get userid and password
    let auth_mgr = crate::gamenv::http_api::auth::get_auth_manager();
    let decoded = match auth_mgr.lock() {
        Ok(mgr) => mgr.decode_txd(&params.txd),
        Err(_) => None,
    };

    let userid = match decoded {
        Some(d) => d.userid,
        None => return Json(serde_json::json!({"error": "Authentication failed"})),
    };

    // Execute command
    let result = match execute_command_internal(userid.clone(), params.cmd.clone(), state).await {
        Ok(r) => r,
        Err(_) => return Json(serde_json::json!({"error": "Command failed"})),
    };

    // Parse the output to extract game state
    let response = build_game_response(&result.output, &userid, &params.cmd).await;

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
        ("newbie_square".to_string(), None)
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

        Some(RoomData {
            id: room.id.clone(),
            name: room.name.clone(),
            short: room.short.clone(),
            long: room.long.clone(),
            npcs,
            exits,
        })
    } else {
        None
    };

    // 更新解析器的房间信息
    if let Some(ref room_data) = room_info {
        parser.update_room(room_data);
    }

    // 解析输出为 mud_lines
    let mud_lines = if command == "look" || command == "l" {
        // look 命令使用完整的房间渲染
        parser.generate_room_json()
    } else {
        // 其他命令解析输出
        parser.parse_output(output)
    };

    // 构建消息类型（基于命令）
    let msg_type = match command {
        "kill" | "attack" => "combat",
        "talk" => "system",
        _ => "info"
    };

    serde_json::json!({
        "status": "success",
        "mud_lines": mud_lines,
        "room_info": room_info,
        "player_stats": player_stats,
        "messages": [{
            "type": msg_type,
            "text": output
        }],
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
        "actions": [
            {"id": "look", "label": "查看", "command": "look", "style": "primary"},
            {"id": "inventory", "label": "背包", "command": "inventory", "style": "default"},
            {"id": "score", "label": "状态", "command": "score", "style": "default"},
            {"id": "skills", "label": "技能", "command": "skills", "style": "default"}
        ]
    })
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
struct ApiGetParams {
    txd: String,
    cmd: String,
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

    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    let cmd = parts.get(0).unwrap_or(&"").to_lowercase();
    let args = &parts[1..];

    // 获取或创建玩家状态
    let player_mgr = get_player_manager();
    let mut player_mgr_write = player_mgr.write().await;
    let player_state = player_mgr_write.get_or_create(userid.to_string()).await;
    drop(player_mgr_write); // 释放锁

    // 对于需要 world 的命令，先获取当前房间
    let player_room = {
        let state = player_state.read().await;
        state.current_room.clone()
    };

    // 先获取 world 的 Arc，避免临时值问题
    let world_arc = get_world();

    match cmd.as_str() {
        "look" | "l" => {
            let world_guard = world_arc.read().await;
            look_command(&world_guard, &player_room).await
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
        cmd if cmd.starts_with("say") => {
            let msg = args.join(" ");
            format!("{}说: {}", userid, msg)
        }
        "help" => {
            r#"
§H可用命令:§N
§C【基础】§N
  look/l - 查看周围环境
  north/n, south/s, east/e, west/w, up/u, down/d - 移动

§C【互动】§N
  talk <npc> - 与NPC对话
  ask <npc> <option> - 选择对话选项
  kill <monster> - 攻击怪物

§C【角色】§N
  inventory/i - 查看背包
  equipment/eq - 查看装备
  score - 查看状态
  skills - 查看技能

§C【社交】§N
  who - 查看在线玩家
  say <message> - 说话
  tell <player> <message> - 悄悄话

§C【系统】§N
  help - 显示帮助
  save - 保存进度
"#.to_string()
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
