// gamenv/http_api/handlers.rs - HTTP 请求处理器
// 对应 txpike9 的各种 HTTP 处理函数

use axum::{
    extract::{Path, Query, State},
    response::{Html, Json},
};
use serde::{Deserialize, Serialize};
use crate::gamenv::http_api::{HttpApiState, ApiError};

/// 游戏页面渲染请求
#[derive(Debug, Deserialize)]
pub struct GamePageRequest {
    pub userid: Option<String>,
    pub txd: Option<String>,
    pub view: Option<String>,
}

/// 渲染游戏页面
pub async fn render_game_page(
    State(state): State<HttpApiState>,
    Query(params): Query<GamePageRequest>,
) -> Result<Html<String>, ApiError> {
    // 验证 TXD
    let auth_data = if let Some(txd) = params.txd {
        let auth_mgr = crate::gamenv::http_api::auth::get_auth_manager();
        let mut mgr = auth_mgr.lock().map_err(|e| {
            ApiError::Internal(format!("Auth lock error: {}", e))
        })?;
        mgr.verify_txd(&txd).await.ok_or(ApiError::AuthFailed)?
    } else {
        return Err(ApiError::AuthFailed);
    };

    // 获取用户数据
    let user_html = render_user_interface(&auth_data.userid, &params.view).await;

    Ok(Html(user_html))
}

/// 渲染用户界面
async fn render_user_interface(userid: &str, view: &Option<String>) -> String {
    use crate::gamenv::http_api::utils::parse_color_codes;

    let content = match view.as_deref() {
        Some("hp") => render_hp_view(userid),
        Some("inventory") => render_inventory_view(userid),
        Some("skills") => render_skills_view(userid),
        _ => render_default_view(userid),
    };

    parse_color_codes(&content)
}

/// 渲染 HP 视图
fn render_hp_view(userid: &str) -> String {
    // TODO: 从数据库获取实际数据
    format!(r#"
<div class="status-panel">
    <h3>身体情况</h3>
    <p>生命: 100/100</p>
    <p>精神: 50/50</p>
    <p>内力: 50/50</p>
    <p>潜能: 100</p>
</div>
"#)
}

/// 渲染背包视图
fn render_inventory_view(userid: &str) -> String {
    format!(r#"
<div class="inventory-panel">
    <h3>背包物品</h3>
    <p>暂无物品</p>
</div>
"#)
}

/// 渲染技能视图
fn render_skills_view(userid: &str) -> String {
    format!(r#"
<div class="skills-panel">
    <h3>技能列表</h3>
    <p>暂无技能</p>
</div>
"#)
}

/// 渲染默认视图
fn render_default_view(userid: &str) -> String {
    format!(r#"
<div class="main-panel">
    <h2>欢迎, {}!</h2>
    <p>RustMUD - 基于 Rust 的 MUD 引擎</p>
</div>
"#, userid)
}

/// 获取房间信息 API
#[derive(Debug, Serialize)]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub short: String,
    pub long: String,
    pub exits: Vec<String>,
    pub npcs: Vec<String>,
    pub items: Vec<String>,
}

pub async fn get_room_info(
    State(_state): State<HttpApiState>,
    Path(room_id): Path<String>,
) -> Result<Json<RoomInfo>, ApiError> {
    // TODO: 从数据库/内存加载实际房间数据
    Ok(Json(RoomInfo {
        id: room_id.clone(),
        name: "客栈".to_string(),
        short: "长安客栈".to_string(),
        long: "这是一家古老的客栈，来往的旅客络绎不绝。".to_string(),
        exits: vec!["east".to_string(), "west".to_string()],
        npcs: vec![],
        items: vec![],
    }))
}

/// 心跳检查
#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub status: String,
    pub timestamp: i64,
    pub online_users: usize,
}

pub async fn heartbeat(
    State(state): State<HttpApiState>,
) -> Json<HeartbeatResponse> {
    let online_count = state.virtual_conns.read().await.count();

    Json(HeartbeatResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        online_users: online_count,
    })
}
