// gamenv/single/daemons/itemd.rs - 物品管理守护进程
// 对应 txpike9/gamenv/single/daemons/itemd.pike

use crate::core::*;
use crate::gamenv::world::ItemTemplate;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 物品管理守护进程
///
/// 负责管理所有游戏物品的定义和数据
pub struct ItemDaemon {
    /// 所有物品模板
    items: HashMap<String, ItemTemplate>,
}

impl ItemDaemon {
    /// 创建新的物品守护进程
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// 添加物品模板
    pub fn add_item(&mut self, item: ItemTemplate) {
        self.items.insert(item.id.clone(), item);
    }

    /// 获取物品
    pub fn get_item(&self, item_id: &str) -> Option<&ItemTemplate> {
        self.items.get(item_id)
    }

    /// 获取所有物品
    pub fn get_all_items(&self) -> Vec<&ItemTemplate> {
        self.items.values().collect()
    }

    /// 根据类型获取物品
    pub fn get_items_by_type(&self, item_type: &crate::gamenv::world::ItemType) -> Vec<&ItemTemplate> {
        self.items.values()
            .filter(|item| item.item_type == *item_type)
            .collect()
    }

    /// 根据等级获取物品
    pub fn get_items_by_level(&self, min_level: i32, max_level: i32) -> Vec<&ItemTemplate> {
        self.items.values()
            .filter(|item| item.level >= min_level && item.level <= max_level)
            .collect()
    }

    /// 搜索物品
    pub fn search_items(&self, keyword: &str) -> Vec<&ItemTemplate> {
        self.items.values()
            .filter(|item| {
                item.name.contains(keyword) || item.id.contains(keyword)
            })
            .collect()
    }

    /// 初始化默认物品
    pub fn init_default_items(&mut self) {
        // TODO: 从配置文件或数据库加载物品
        tracing::info!("Initializing default items...");
    }
}

impl Default for ItemDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局物品守护进程
pub static ITEMD: std::sync::OnceLock<RwLock<ItemDaemon>> = std::sync::OnceLock::new();

/// 获取物品守护进程
pub fn get_itemd() -> &'static RwLock<ItemDaemon> {
    ITEMD.get_or_init(|| RwLock::new(ItemDaemon::new()))
}
