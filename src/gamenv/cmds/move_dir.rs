// gamenv/cmds/move_dir.rs - 移动命令
// 对应 txpike9/wapmud2/cmds/ 目录中的移动相关命令

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock as TokioRwLock;

use crate::gamenv::world::GameWorld;
use crate::gamenv::core::command::*;
use crate::gamenv::player_state;
use crate::gamenv::world;

/// 移动命令结果
pub struct MoveResult {
    pub success: bool,
    pub new_room: String,
    pub message: String,
}

/// 移动到指定方向
pub async fn move_command(world: &GameWorld, room_id: &str, direction: &str) -> MoveResult {
    if let Some(room) = world.get_room(room_id) {
        // 检查方向是否有效
        if let Some(target_room_id) = room.exits.get(direction) {
            // 检查目标房间是否存在
            if let Some(_target_room) = world.get_room(target_room_id) {
                MoveResult {
                    success: true,
                    new_room: target_room_id.clone(),
                    message: format!("你向{}走去。", direction_name(direction)),
                }
            } else {
                MoveResult {
                    success: false,
                    new_room: room_id.to_string(),
                    message: "那个方向好像不通。".to_string(),
                }
            }
        } else {
            MoveResult {
                success: false,
                new_room: room_id.to_string(),
                message: "这个方向没有出口。".to_string(),
            }
        }
    } else {
        MoveResult {
            success: false,
            new_room: room_id.to_string(),
            message: "无法找到当前房间。".to_string(),
        }
    }
}

/// 获取方向的中文名称
fn direction_name(dir: &str) -> &str {
    match dir {
        "north" | "n" => "北",
        "south" | "s" => "南",
        "east" | "e" => "东",
        "west" | "w" => "西",
        "northeast" | "ne" => "东北",
        "northwest" | "nw" => "西北",
        "southeast" | "se" => "东南",
        "southwest" | "sw" => "西南",
        "up" | "u" => "上",
        "down" | "d" => "下",
        _ => dir,
    }
}

/// 通用移动命令（go <direction> 或直接输入方向）
pub struct MoveDirCommand;

/// 辅助函数：使用 RwLock 执行移动
async fn move_with_guard(
    world_guard: &TokioRwLock<GameWorld>,
    room_id: &str,
    direction: &str,
    player_id: &str,
) -> (String, bool) {
    let world = world_guard.read().await;
    let result = move_command(&world, room_id, direction).await;

    if result.success {
        let player_mgr = player_state::get_player_manager();
        let mgr = player_mgr.read().await;
        if let Some(player) = mgr.get(player_id).await {
            let mut state = player.write().await;
            state.move_to(result.new_room.clone());
            drop(state);
            drop(mgr);
            crate::gamenv::world::try_reset_room(&result.new_room).await;
        }
    }

    (result.message, result.success)
}

#[async_trait]
impl CommandHandler for MoveDirCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        if ctx.args.is_empty() {
            return CommandResult::from("请指定方向。");
        }

        let direction = &ctx.args[0];
        let world_ref = world::get_world();
        let (output, success) = move_with_guard(&world_ref, &ctx.room_id, direction, &ctx.player_id).await;

        CommandResult {
            output,
            should_update_room: success,
            events: vec![],
        }
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "go",
                    "向指定方向移动",
                    CommandCategory::Movement
                )
                .with_args(1, Some(1))
            });
        &META
    }
}

/// 获取移动命令实例
pub fn get_command() -> Arc<dyn CommandHandler> {
    Arc::new(MoveDirCommand) as Arc<dyn CommandHandler>
}
