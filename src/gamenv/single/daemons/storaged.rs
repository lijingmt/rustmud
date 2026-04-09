// gamenv/single/daemons/storaged.rs - 仓库系统守护进程
// 对应 txpike9/gamenv/single/daemons/storaged.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 仓库格子
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageSlot {
    /// 物品ID
    pub item_id: String,
    /// 物品名称
    pub item_name: String,
    /// 数量
    pub count: i32,
    /// 物品类型（用于分类）
    pub item_type: String,
    /// 品质
    pub quality: i32,
    /// 绑定状态
    pub bound: bool,
}

/// 仓库页面
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoragePage {
    /// 页码
    pub page_num: i32,
    /// 容量
    pub capacity: i32,
    /// 格子
    pub slots: HashMap<i32, StorageSlot>,
}

impl StoragePage {
    /// 创建新页面
    pub fn new(page_num: i32, capacity: i32) -> Self {
        Self {
            page_num,
            capacity,
            slots: HashMap::new(),
        }
    }

    /// 获取空位数
    pub fn free_slots(&self) -> i32 {
        self.capacity - self.slots.len() as i32
    }

    /// 是否已满
    pub fn is_full(&self) -> bool {
        self.slots.len() >= self.capacity as usize
    }

    /// 获取首个空位
    pub fn first_free_slot(&self) -> Option<i32> {
        for i in 1..=self.capacity {
            if !self.slots.contains_key(&i) {
                return Some(i);
            }
        }
        None
    }

    /// 添加物品
    pub fn add_item(&mut self, slot: i32, item: StorageSlot) -> Result<()> {
        if slot < 1 || slot > self.capacity {
            return Err(MudError::RuntimeError("格子位置无效".to_string()));
        }

        if self.slots.contains_key(&slot) {
            return Err(MudError::RuntimeError("格子已被占用".to_string()));
        }

        self.slots.insert(slot, item);
        Ok(())
    }

    /// 移除物品
    pub fn remove_item(&mut self, slot: i32) -> Result<StorageSlot> {
        self.slots.remove(&slot)
            .ok_or_else(|| MudError::NotFound("格子为空".to_string()))
    }

    /// 获取物品
    pub fn get_item(&self, slot: i32) -> Option<&StorageSlot> {
        self.slots.get(&slot)
    }
}

/// 玩家仓库
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStorage {
    /// 玩家ID
    pub player_id: String,
    /// 当前页面数
    pub page_count: i32,
    /// 最大页面数
    pub max_pages: i32,
    /// 每页容量
    pub page_capacity: i32,
    /// 页面
    pub pages: HashMap<i32, StoragePage>,
    /// 金币存储
    pub gold_stored: u64,
}

impl PlayerStorage {
    /// 创建新仓库
    pub fn new(player_id: String, initial_pages: i32, page_capacity: i32) -> Self {
        let mut pages = HashMap::new();
        for i in 1..=initial_pages {
            pages.insert(i, StoragePage::new(i, page_capacity));
        }

        Self {
            player_id,
            page_count: initial_pages,
            max_pages: 10,
            page_capacity,
            pages,
            gold_stored: 0,
        }
    }

    /// 获取当前页
    pub fn get_page(&self, page_num: i32) -> Option<&StoragePage> {
        self.pages.get(&page_num)
    }

    /// 获取可变页
    pub fn get_page_mut(&mut self, page_num: i32) -> Option<&mut StoragePage> {
        self.pages.get_mut(&page_num)
    }

    /// 添加新页面
    pub fn add_page(&mut self) -> Result<()> {
        if self.page_count >= self.max_pages {
            return Err(MudError::RuntimeError("已达到最大页数".to_string()));
        }

        self.page_count += 1;
        self.pages.insert(self.page_count, StoragePage::new(self.page_count, self.page_capacity));
        Ok(())
    }

    /// 存入物品
    pub fn store_item(&mut self, item: StorageSlot, preferred_page: Option<i32>) -> Result<i32> {
        // 尝试指定页面
        if let Some(page_num) = preferred_page {
            if let Some(page) = self.get_page_mut(page_num) {
                if !page.is_full() {
                    let slot = page.first_free_slot()
                        .ok_or_else(|| MudError::RuntimeError("页面已满".to_string()))?;
                    page.add_item(slot, item.clone())?;
                    return Ok((page_num * 100) + slot);
                }
            }
        }

        // 尝试所有页面
        for page_num in 1..=self.page_count {
            if let Some(page) = self.get_page_mut(page_num) {
                if !page.is_full() {
                    let slot = page.first_free_slot()
                        .ok_or_else(|| MudError::RuntimeError("页面已满".to_string()))?;
                    page.add_item(slot, item.clone())?;
                    return Ok((page_num * 100) + slot);
                }
            }
        }

        Err(MudError::RuntimeError("仓库已满".to_string()))
    }

    /// 取出物品
    pub fn retrieve_item(&mut self, position: i32) -> Result<StorageSlot> {
        let page_num = position / 100;
        let slot = position % 100;

        let page = self.pages.get_mut(&page_num)
            .ok_or_else(|| MudError::NotFound("页面不存在".to_string()))?;

        page.remove_item(slot)
    }

    /// 转移物品
    pub fn move_item(&mut self, from_pos: i32, to_pos: i32) -> Result<()> {
        let from_page = from_pos / 100;
        let from_slot = from_pos % 100;
        let to_page = to_pos / 100;
        let to_slot = to_pos % 100;

        if from_page == to_page && from_slot == to_slot {
            return Ok(());
        }

        // 取出源物品
        let item = {
            let page = self.pages.get_mut(&from_page)
                .ok_or_else(|| MudError::NotFound("源页面不存在".to_string()))?;
            page.remove_item(from_slot)?
        };

        // 放入目标位置
        {
            let page = self.pages.get_mut(&to_page)
                .ok_or_else(|| MudError::NotFound("目标页面不存在".to_string()))?;

            if page.slots.contains_key(&to_slot) {
                // 目标位置有物品，交换
                let target_item = page.remove_item(to_slot)?;
                page.add_item(to_slot, item.clone())?;

                let from_page = self.pages.get_mut(&from_page).unwrap();
                from_page.add_item(from_slot, target_item)?;
            } else {
                page.add_item(to_slot, item)?;
            }
        }

        Ok(())
    }

    /// 存入金币
    pub fn store_gold(&mut self, amount: u64) -> Result<()> {
        self.gold_stored += amount;
        Ok(())
    }

    /// 取出金币
    pub fn retrieve_gold(&mut self, amount: u64) -> Result<u64> {
        if amount > self.gold_stored {
            return Err(MudError::RuntimeError("金币不足".to_string()));
        }
        self.gold_stored -= amount;
        Ok(amount)
    }

    /// 获取总物品数
    pub fn total_items(&self) -> usize {
        self.pages.values()
            .map(|p| p.slots.len())
            .sum()
    }

    /// 获取总空位数
    pub fn total_free_slots(&self) -> usize {
        self.pages.values()
            .map(|p| p.free_slots() as usize)
            .sum()
    }

    /// 格式化页面内容
    pub fn format_page(&self, page_num: i32) -> String {
        if let Some(page) = self.get_page(page_num) {
            let mut output = format!(
                "§H=== 仓库 (第{}/{}页) ===§N\n\
                 金币: {} | 空位: {}/{}\n\n",
                page_num, self.page_count,
                self.gold_stored,
                page.free_slots(),
                page.capacity
            );

            let mut slot_list: Vec<_> = page.slots.iter().collect();
            slot_list.sort_by_key(|(k, _)| *k);

            if slot_list.is_empty() {
                output.push_str("此页面为空。\n");
            } else {
                for (slot, item) in slot_list {
                    let quality = match item.quality {
                        5 => "§Y",
                        4 => "§M",
                        3 => "§B",
                        2 => "§C",
                        1 => "§G",
                        _ => "",
                    };
                    output.push_str(&format!(
                        "  [{}] {}{} x{}{}\n",
                        slot, quality, item.item_name, item.count,
                        if item.bound { " §R(绑定)§N" } else { "" }
                    ));
                }
            }

            output
        } else {
            "§R页面不存在§N".to_string()
        }
    }
}

/// 仓库守护进程
pub struct StorageDaemon {
    /// 玩家仓库
    storages: HashMap<String, PlayerStorage>,
    /// 每页容量
    page_capacity: i32,
    /// 初始页数
    initial_pages: i32,
    /// 最大页数
    max_pages: i32,
    /// 扩页费用
    page_expand_cost: u64,
}

impl StorageDaemon {
    /// 创建新的仓库守护进程
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
            page_capacity: 20, // 每页20格
            initial_pages: 2,   // 初始2页
            max_pages: 10,
            page_expand_cost: 10000, // 扩页费用10000金币
        }
    }

    /// 获取或创建玩家仓库
    pub fn get_or_create_storage(&mut self, player_id: &str) -> &mut PlayerStorage {
        if !self.storages.contains_key(player_id) {
            let storage = PlayerStorage::new(
                player_id.to_string(),
                self.initial_pages,
                self.page_capacity
            );
            self.storages.insert(player_id.to_string(), storage);
        }
        self.storages.get_mut(player_id).unwrap()
    }

    /// 获取仓库
    pub fn get_storage(&self, player_id: &str) -> Option<&PlayerStorage> {
        self.storages.get(player_id)
    }

    /// 获取可变仓库
    pub fn get_storage_mut(&mut self, player_id: &str) -> Option<&mut PlayerStorage> {
        self.storages.get_mut(player_id)
    }

    /// 存入物品
    pub fn store_item(
        &mut self,
        player_id: &str,
        item: StorageSlot,
        preferred_page: Option<i32>,
    ) -> Result<i32> {
        let storage = self.get_or_create_storage(player_id);
        storage.store_item(item, preferred_page)
    }

    /// 取出物品
    pub fn retrieve_item(&mut self, player_id: &str, position: i32) -> Result<StorageSlot> {
        if let Some(storage) = self.get_storage_mut(player_id) {
            storage.retrieve_item(position)
        } else {
            Err(MudError::NotFound("仓库不存在".to_string()))
        }
    }

    /// 转移物品
    pub fn move_item(&mut self, player_id: &str, from_pos: i32, to_pos: i32) -> Result<()> {
        if let Some(storage) = self.get_storage_mut(player_id) {
            storage.move_item(from_pos, to_pos)
        } else {
            Err(MudError::NotFound("仓库不存在".to_string()))
        }
    }

    /// 存入金币
    pub fn store_gold(&mut self, player_id: &str, amount: u64) -> Result<()> {
        let storage = self.get_or_create_storage(player_id);
        storage.store_gold(amount)
    }

    /// 取出金币
    pub fn retrieve_gold(&mut self, player_id: &str, amount: u64) -> Result<u64> {
        if let Some(storage) = self.get_storage_mut(player_id) {
            storage.retrieve_gold(amount)
        } else {
            Err(MudError::NotFound("仓库不存在".to_string()))
        }
    }

    /// 扩展页面
    pub fn expand_page(&mut self, player_id: &str, player_gold: u64) -> Result<()> {
        if player_gold < self.page_expand_cost {
            return Err(MudError::RuntimeError(
                format!("金币不足，需要{}金币", self.page_expand_cost)
            ));
        }

        let storage = self.get_or_create_storage(player_id);
        storage.add_page()?;
        Ok(())
    }

    /// 获取仓库信息
    pub fn get_storage_info(&self, player_id: &str) -> String {
        if let Some(storage) = self.get_storage(player_id) {
            format!(
                "§H=== 仓库信息 ===§N\n\
                 页数: {}/{}\n\
                 每页容量: {}\n\
                 总物品: {}\n\
                 总空位: {}\n\
                 存储金币: {}",
                storage.page_count,
                storage.max_pages,
                storage.page_capacity,
                storage.total_items(),
                storage.total_free_slots(),
                storage.gold_stored
            )
        } else {
            "§R你还没有开通仓库§N".to_string()
        }
    }
}

impl Default for StorageDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局仓库守护进程
pub static STORAGED: std::sync::OnceLock<RwLock<StorageDaemon>> = std::sync::OnceLock::new();

/// 获取仓库守护进程
pub fn get_storaged() -> &'static RwLock<StorageDaemon> {
    STORAGED.get_or_init(|| RwLock::new(StorageDaemon::default()))
}
