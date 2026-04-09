// gamenv/single/daemons/malld.rs - 商城系统守护进程
// 对应 txpike9/gamenv/single/daemons/malld.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 商品类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MallItemType {
    /// 消耗品
    Consumable,
    /// 装备
    Equipment,
    /// 材料
    Material,
    /// 宠物
    Pet,
    /// 礼包
    GiftPack,
    /// 特殊物品
    Special,
}

/// 货币类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CurrencyType {
    /// 金币
    Gold,
    /// 钻石
    Diamond,
    /// 绑定钻石
    BoundDiamond,
    /// 代币
    Token,
    /// VIP积分
    VIPPoints,
}

/// 商品
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MallItem {
    /// 商品ID
    pub id: String,
    /// 商品名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 商品类型
    pub item_type: MallItemType,
    /// 原价
    pub original_price: u64,
    /// 当前价格
    pub current_price: u64,
    /// 货币类型
    pub currency: CurrencyType,
    /// 折扣 (0-100)
    pub discount: i32,
    /// 库存 (-1表示无限)
    pub stock: i32,
    /// 每日限购数量
    pub daily_limit: Option<i32>,
    /// VIP等级要求
    pub vip_requirement: i32,
    /// 等级要求
    pub level_requirement: i32,
    /// 是否可购买
    pub available: bool,
    /// 排序权重
    pub sort_order: i32,
    /// 标签
    pub tags: Vec<String>,
    /// 商品图标
    pub icon: String,
}

impl MallItem {
    /// 获取实际价格（折扣后）
    pub fn get_actual_price(&self) -> u64 {
        if self.discount > 0 {
            (self.current_price * (100 - self.discount) as u64) / 100
        } else {
            self.current_price
        }
    }

    /// 是否有折扣
    pub fn has_discount(&self) -> bool {
        self.discount > 0
    }

    /// 是否有库存
    pub fn has_stock(&self) -> bool {
        self.stock < 0 || self.stock > 0
    }

    /// 格式化商品信息
    pub fn format_info(&self) -> String {
        let currency_name = match self.currency {
            CurrencyType::Gold => "金币",
            CurrencyType::Diamond => "钻石",
            CurrencyType::BoundDiamond => "绑定钻石",
            CurrencyType::Token => "代币",
            CurrencyType::VIPPoints => "VIP积分",
        };

        let type_name = match self.item_type {
            MallItemType::Consumable => "消耗品",
            MallItemType::Equipment => "装备",
            MallItemType::Material => "材料",
            MallItemType::Pet => "宠物",
            MallItemType::GiftPack => "礼包",
            MallItemType::Special => "特殊",
        };

        let price_display = if self.has_discount() {
            format!("{} {} §R({}折)§N", self.get_actual_price(), currency_name, self.discount)
        } else {
            format!("{} {}", self.get_actual_price(), currency_name)
        };

        let stock_display = if self.stock < 0 {
            "无限".to_string()
        } else {
            format!("{}", self.stock)
        };

        format!(
            "§H[{}]§N {}\n\
             类型: {} | 价格: {}\n\
             库存: {} | 描述: {}",
            self.icon, self.name, type_name, price_display, stock_display, self.description
        )
    }
}

/// 购买记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PurchaseRecord {
    /// 玩家ID
    pub player_id: String,
    /// 商品ID
    pub item_id: String,
    /// 购买数量
    pub count: i32,
    /// 购买价格
    pub price: u64,
    /// 购买时间
    pub purchased_at: i64,
}

/// 玩家商城数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerMallData {
    /// 每日购买记录
    pub daily_purchases: HashMap<String, i32>, // item_id -> count
    /// 最后更新日期
    pub last_update_date: i64,
}

impl PlayerMallData {
    pub fn new() -> Self {
        Self {
            daily_purchases: HashMap::new(),
            last_update_date: 0,
        }
    }

    /// 检查是否需要重置
    pub fn needs_reset(&self) -> bool {
        let today = chrono::Utc::now().timestamp() / 86400;
        let last_day = self.last_update_date / 86400;
        today != last_day
    }

    /// 重置每日数据
    pub fn reset_daily(&mut self) {
        self.daily_purchases.clear();
        self.last_update_date = chrono::Utc::now().timestamp();
    }

    /// 获取今日已购买数量
    pub fn get_today_purchased(&self, item_id: &str) -> i32 {
        *self.daily_purchases.get(item_id).unwrap_or(&0)
    }

    /// 增加购买记录
    pub fn add_purchase(&mut self, item_id: &str, count: i32) {
        let entry = self.daily_purchases.entry(item_id.to_string()).or_insert(0);
        *entry += count;
    }
}

impl Default for PlayerMallData {
    fn default() -> Self {
        Self::new()
    }
}

/// 商城守护进程
pub struct MallDaemon {
    /// 所有商品
    items: HashMap<String, MallItem>,
    /// 玩家商城数据
    player_data: HashMap<String, PlayerMallData>,
    /// 购买记录
    purchase_records: Vec<PurchaseRecord>,
}

impl MallDaemon {
    /// 创建新的商城守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            items: HashMap::new(),
            player_data: HashMap::new(),
            purchase_records: Vec::new(),
        };

        daemon.init_default_items();
        daemon
    }

    /// 初始化默认商品
    fn init_default_items(&mut self) {
        // 药水类
        let hp_potion = MallItem {
            id: "mall_hp_potion_small".to_string(),
            name: "小生命药水".to_string(),
            description: "恢复100点HP".to_string(),
            item_type: MallItemType::Consumable,
            original_price: 100,
            current_price: 100,
            currency: CurrencyType::Gold,
            discount: 0,
            stock: -1,
            daily_limit: Some(100),
            vip_requirement: 0,
            level_requirement: 1,
            available: true,
            sort_order: 1,
            tags: vec!["消耗品".to_string(), "药水".to_string()],
            icon: "🧪".to_string(),
        };

        let mp_potion = MallItem {
            id: "mall_mp_potion_small".to_string(),
            name: "小魔法药水".to_string(),
            description: "恢复100点MP".to_string(),
            item_type: MallItemType::Consumable,
            original_price: 100,
            current_price: 100,
            currency: CurrencyType::Gold,
            discount: 0,
            stock: -1,
            daily_limit: Some(100),
            vip_requirement: 0,
            level_requirement: 1,
            available: true,
            sort_order: 2,
            tags: vec!["消耗品".to_string(), "药水".to_string()],
            icon: "🧪".to_string(),
        };

        // 强化材料
        let reinforce_stone = MallItem {
            id: "mall_reinforce_stone".to_string(),
            name: "强化石".to_string(),
            description: "用于装备强化".to_string(),
            item_type: MallItemType::Material,
            original_price: 1000,
            current_price: 1000,
            currency: CurrencyType::Gold,
            discount: 0,
            stock: -1,
            daily_limit: Some(50),
            vip_requirement: 0,
            level_requirement: 10,
            available: true,
            sort_order: 10,
            tags: vec!["材料".to_string(), "强化".to_string()],
            icon: "💎".to_string(),
        };

        // 礼包类
        let starter_pack = MallItem {
            id: "mall_starter_pack".to_string(),
            name: "新手礼包".to_string(),
            description: "包含大量新手必备物品".to_string(),
            item_type: MallItemType::GiftPack,
            original_price: 1000,
            current_price: 100,
            currency: CurrencyType::Diamond,
            discount: 90,
            stock: 1,
            daily_limit: Some(1),
            vip_requirement: 0,
            level_requirement: 1,
            available: true,
            sort_order: 100,
            tags: vec!["礼包".to_string(), "新手".to_string()],
            icon: "🎁".to_string(),
        };

        self.items.insert(hp_potion.id.clone(), hp_potion);
        self.items.insert(mp_potion.id.clone(), mp_potion);
        self.items.insert(reinforce_stone.id.clone(), reinforce_stone);
        self.items.insert(starter_pack.id.clone(), starter_pack);
    }

    /// 获取商品
    pub fn get_item(&self, item_id: &str) -> Option<&MallItem> {
        self.items.get(item_id)
    }

    /// 获取所有商品
    pub fn get_all_items(&self) -> Vec<&MallItem> {
        self.items.values().collect()
    }

    /// 按类型获取商品
    pub fn get_items_by_type(&self, item_type: MallItemType) -> Vec<&MallItem> {
        self.items.values()
            .filter(|item| item.item_type == item_type && item.available)
            .collect()
    }

    /// 获取可用商品
    pub fn get_available_items(&self, player_level: i32, player_vip: i32) -> Vec<&MallItem> {
        self.items.values()
            .filter(|item| {
                item.available
                    && item.level_requirement <= player_level
                    && item.vip_requirement <= player_vip
                    && item.has_stock()
            })
            .collect()
    }

    /// 获取玩家商城数据
    fn get_player_data_mut(&mut self, player_id: &str) -> &mut PlayerMallData {
        self.player_data
            .entry(player_id.to_string())
            .or_insert_with(PlayerMallData::new)
    }

    /// 购买商品
    pub fn purchase(
        &mut self,
        player_id: &str,
        player_level: i32,
        player_vip: i32,
        item_id: &str,
        count: i32,
    ) -> Result<PurchaseResult> {
        // 获取商品
        let item = self.items.get(item_id)
            .ok_or_else(|| MudError::NotFound("商品不存在".to_string()))?;

        // 验证可用性
        if !item.available {
            return Err(MudError::RuntimeError("商品已下架".to_string()));
        }

        // 验证等级
        if player_level < item.level_requirement {
            return Err(MudError::RuntimeError("等级不足".to_string()));
        }

        // 验证VIP
        if player_vip < item.vip_requirement {
            return Err(MudError::RuntimeError("VIP等级不足".to_string()));
        }

        // 验证库存
        if item.stock >= 0 && item.stock < count {
            return Err(MudError::RuntimeError("库存不足".to_string()));
        }

        // 提取每日限购数据
        let daily_limit = item.daily_limit;

        // 计算价格和准备返回数据（在可变借用之前）
        let actual_price = item.get_actual_price();
        let total_cost = actual_price * count as u64;
        let item_id_clone = item_id.to_string();
        let item_name = item.name.clone();
        let item_currency = item.currency.clone();

        // 验证并更新每日限购
        {
            let data = self.get_player_data_mut(player_id);
            if data.needs_reset() {
                data.reset_daily();
            }

            if let Some(limit) = daily_limit {
                let purchased = data.get_today_purchased(item_id);
                if purchased + count > limit {
                    return Err(MudError::RuntimeError(
                        format!("超过每日限购数量({})", limit)
                    ));
                }
                data.add_purchase(item_id, count);
            }
        }

        // 记录购买
        let record = PurchaseRecord {
            player_id: player_id.to_string(),
            item_id: item_id_clone.clone(),
            count,
            price: total_cost,
            purchased_at: chrono::Utc::now().timestamp(),
        };
        self.purchase_records.push(record);

        // 扣除库存
        if let Some(item) = self.items.get_mut(&item_id_clone) {
            if item.stock >= 0 {
                item.stock -= count;
            }
        }

        Ok(PurchaseResult {
            item_id: item_id_clone,
            item_name,
            count,
            cost: total_cost,
            currency: item_currency,
        })
    }

    /// 格式化商品列表
    pub fn format_item_list(&self, items: &[&MallItem]) -> String {
        let mut output = format!("§H=== 商城商品 ({}件) ===§N\n", items.len());

        if items.is_empty() {
            output.push_str("暂无可用商品。\n");
        } else {
            for item in items {
                let discount_tag = if item.has_discount() {
                    format!(" §R{}折§N", item.discount)
                } else {
                    String::new()
                };

                let currency_name = match item.currency {
                    CurrencyType::Gold => "金币",
                    CurrencyType::Diamond => "钻石",
                    CurrencyType::BoundDiamond => "绑定钻石",
                    CurrencyType::Token => "代币",
                    CurrencyType::VIPPoints => "VIP积分",
                };

                output.push_str(&format!(
                    "  {} {} {} {}{}",
                    item.icon,
                    item.name,
                    item.get_actual_price(),
                    currency_name,
                    discount_tag
                ));

                if let Some(limit) = item.daily_limit {
                    output.push_str(&format!(" [限购{}]", limit));
                }

                output.push('\n');
            }
        }

        output
    }

    /// 获取玩家购买记录
    pub fn get_purchase_records(&self, player_id: &str) -> Vec<&PurchaseRecord> {
        self.purchase_records.iter()
            .filter(|r| r.player_id == player_id)
            .collect()
    }

    /// 清理过期记录
    pub fn cleanup_old_records(&mut self, days: i64) -> usize {
        let cutoff = chrono::Utc::now().timestamp() - (days * 86400);
        let original_len = self.purchase_records.len();
        self.purchase_records.retain(|r| r.purchased_at > cutoff);
        original_len - self.purchase_records.len()
    }

    /// 刷新商品折扣
    pub fn refresh_discounts(&mut self) {
        for item in self.items.values_mut() {
            // 10%概率出现折扣
            if rand::random::<f32>() < 0.1 {
                item.discount = [10, 20, 30, 50][rand::random::<usize>() % 4];
            } else {
                item.discount = 0;
            }
        }
    }
}

impl Default for MallDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 购买结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PurchaseResult {
    pub item_id: String,
    pub item_name: String,
    pub count: i32,
    pub cost: u64,
    pub currency: CurrencyType,
}

/// 全局商城守护进程
pub static MALLD: std::sync::OnceLock<RwLock<MallDaemon>> = std::sync::OnceLock::new();

/// 获取商城守护进程
pub fn get_malld() -> &'static RwLock<MallDaemon> {
    MALLD.get_or_init(|| RwLock::new(MallDaemon::default()))
}
