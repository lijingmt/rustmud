// gamenv/world/mod.rs - 游戏世界数据
// 对应 txpike9 的房间、NPC、物品等游戏内容

pub mod room;
pub mod npc;
pub mod item;
pub mod shop;
pub mod loader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

pub use room::*;
pub use npc::*;
pub use item::*;
pub use shop::*;
pub use loader::*;

/// 全局游戏世界实例
static WORLD: once_cell::sync::Lazy<Arc<TokioRwLock<GameWorld>>> =
    once_cell::sync::Lazy::new(|| Arc::new(TokioRwLock::new(GameWorld::new())));

/// 获取全局游戏世界
pub fn get_world() -> Arc<TokioRwLock<GameWorld>> {
    WORLD.clone()
}

/// 游戏世界
#[derive(Clone, Serialize, Deserialize)]
pub struct GameWorld {
    /// 所有房间
    pub rooms: HashMap<String, Room>,
    /// 所有NPC
    pub npcs: HashMap<String, Npc>,
    /// 所有物品模板
    pub items: HashMap<String, ItemTemplate>,
    /// 所有商店
    pub shops: HashMap<String, Shop>,
}

impl GameWorld {
    pub fn new() -> Self {
        let mut world = Self {
            rooms: HashMap::new(),
            npcs: HashMap::new(),
            items: HashMap::new(),
            shops: HashMap::new(),
        };

        // 从JSON加载游戏世界数据
        world.load_from_json();

        world
    }

    /// 从JSON文件加载游戏世界数据
    fn load_from_json(&mut self) {
        let loader = WorldLoader::new("./data/world");

        // 加载房间
        match loader.load_rooms_from_json() {
            Ok(rooms) => {
                tracing::info!("Loaded {} rooms from JSON", rooms.len());
                self.rooms = rooms;
            }
            Err(e) => {
                tracing::error!("Failed to load rooms: {}", e);
            }
        }

        // 加载NPC
        match loader.load_npcs_from_json() {
            Ok(npcs) => {
                tracing::info!("Loaded {} NPCs from JSON", npcs.len());
                self.npcs = npcs;
            }
            Err(e) => {
                tracing::error!("Failed to load NPCs: {}", e);
            }
        }

        // 加载物品
        match loader.load_items_from_json() {
            Ok(items) => {
                tracing::info!("Loaded {} items from JSON", items.len());
                self.items = items;
            }
            Err(e) => {
                tracing::error!("Failed to load items: {}", e);
            }
        }

        // 加载商店
        match loader.load_shops_from_json() {
            Ok(shops) => {
                tracing::info!("Loaded {} shops from JSON", shops.len());
                self.shops = shops;
            }
            Err(e) => {
                tracing::error!("Failed to load shops: {}", e);
            }
        }
    }

    /// 获取房间
    pub fn get_room(&self, id: &str) -> Option<&Room> {
        self.rooms.get(id)
    }

    /// 获取NPC
    pub fn get_npc(&self, id: &str) -> Option<&Npc> {
        self.npcs.get(id)
    }

    /// 获取物品模板
    pub fn get_item_template(&self, id: &str) -> Option<&ItemTemplate> {
        self.items.get(id)
    }

    /// 获取商店
    pub fn get_shop(&self, id: &str) -> Option<&Shop> {
        self.shops.get(id)
    }

    /// 添加房间
    pub fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.id.clone(), room);
    }

    /// 添加NPC
    pub fn add_npc(&mut self, npc: Npc) {
        self.npcs.insert(npc.id.clone(), npc);
    }

    /// 添加物品模板
    pub fn add_item_template(&mut self, item: ItemTemplate) {
        self.items.insert(item.id.clone(), item);
    }

    /// 添加商店
    pub fn add_shop(&mut self, shop: Shop) {
        self.shops.insert(shop.id.clone(), shop);
    }

    /// 获取房间数量
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// 获取NPC数量
    pub fn npc_count(&self) -> usize {
        self.npcs.len()
    }

    /// 获取物品数量
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// 获取商店数量
    pub fn shop_count(&self) -> usize {
        self.shops.len()
    }
}
