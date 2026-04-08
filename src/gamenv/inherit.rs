// gamenv/inherit.rs - 继承基类
// 对应 txpike9/gamenv/inherit/ 目录

/// NPC 基类 (对应 CHINAQUEST_NPC)
pub mod npc {
    use crate::core::*;

    pub struct Npc {
        pub id: ObjectId,
        pub name: String,
        pub name_cn: String,
        pub level: u32,
        pub hp: i32,
        pub hp_max: i32,
    }

    impl Npc {
        pub fn new(name: String, name_cn: String) -> Self {
            Self {
                id: ObjectId::new(),
                name,
                name_cn,
                level: 1,
                hp: 100,
                hp_max: 100,
            }
        }
    }
}

/// 房间基类 (继承自 ROOM)
pub mod room {
    use crate::core::*;

    pub struct Room {
        pub id: ObjectId,
        pub name: String,
        pub short: String,
        pub long: String,
        pub exits: std::collections::HashMap<String, String>, // direction -> room_id
        pub npcs: Vec<ObjectId>,
        pub items: Vec<ObjectId>,
    }

    impl Room {
        pub fn new(id: String, name: String, short: String, long: String) -> Self {
            Self {
                id: ObjectId::new(),
                name,
                short,
                long,
                exits: std::collections::HashMap::new(),
                npcs: Vec::new(),
                items: Vec::new(),
            }
        }

        pub fn add_exit(&mut self, direction: String, room_id: String) {
            self.exits.insert(direction, room_id);
        }
    }
}

/// 物品基类
pub mod item {
    use crate::core::*;

    pub struct Item {
        pub id: ObjectId,
        pub name: String,
        pub name_cn: String,
        pub value: u32,
    }

    impl Item {
        pub fn new(name: String, name_cn: String) -> Self {
            Self {
                id: ObjectId::new(),
                name,
                name_cn,
                value: 0,
            }
        }
    }
}

pub use npc::Npc;
pub use room::Room;
pub use item::Item;
