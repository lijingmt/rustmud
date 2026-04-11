// gamenv/cmds/move_dir.rs - 移动命令
// 对应 txpike9/wapmud2/cmds/ 目录中的移动相关命令

use crate::gamenv::world::GameWorld;

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
