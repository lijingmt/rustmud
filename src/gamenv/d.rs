// gamenv/d.rs - 房间/世界定义
// 对应 txpike9/gamenv/d/ 目录

use crate::core::*;
use crate::gamenv::inherit::room::Room;
use std::collections::HashMap;

/// 房间管理器
pub struct RoomManager {
    rooms: HashMap<String, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    /// 加载房间 (对应 load_object)
    pub fn load_room(&mut self, room_id: &str) -> Result<&Room> {
        if !self.rooms.contains_key(room_id) {
            // TODO: 从文件加载房间
            let room = Room::new(
                room_id.to_string(),
                "未命名房间".to_string(),
                "这里还没有描述".to_string(),
                "详细描述...".to_string(),
            );
            self.rooms.insert(room_id.to_string(), room);
        }
        Ok(self.rooms.get(room_id).unwrap())
    }
}

/// 初始房间 (对应 /gamenv/d/init)
pub fn init_room() -> Room {
    let mut room = Room::new(
        "init".to_string(),
        "客栈".to_string(),
        "长安客栈".to_string(),
        "这是一家古老的客栈，来往的旅客络绎不绝。\n".to_string(),
    );
    room.add_exit("east".to_string(), "/gamenv/d/changan/street".to_string());
    room.add_exit("west".to_string(), "/gamenv/d/weapon_shop".to_string());
    room
}
