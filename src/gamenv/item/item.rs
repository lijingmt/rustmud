// gamenv/item/item.rs - 物品基类
// 对应 txpike9/gamenv/clone/item/ 中的基础物品

use crate::core::*;
use serde::{Deserialize, Serialize};

/// 物品类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    /// 普通物品
    Normal,
    /// 武器
    Weapon,
    /// 护甲
    Armor,
    /// 药品
    Medicine,
    /// 食物
    Food,
    /// 书籍
    Book,
    /// 任务物品
    Quest,
    /// 货币
    Money,
    /// 材料
    Material,
    /// 宝石
    Gem,
    /// 核心碎片
    CoreShard,
}

/// 物品品质
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ItemQuality {
    /// 普通 (白色)
    Common = 0,
    /// 优秀 (绿色)
    Uncommon = 1,
    /// 稀有 (蓝色)
    Rare = 2,
    /// 史诗 (紫色)
    Epic = 3,
    /// 传说 (橙色)
    Legendary = 4,
    /// 神话 (红色)
    Mythic = 5,
}

impl ItemQuality {
    /// 获取品质对应的颜色代码
    pub fn color_code(&self) -> &str {
        match self {
            ItemQuality::Common => "§w",      // 白色
            ItemQuality::Uncommon => "§g",    // 绿色
            ItemQuality::Rare => "§b",        // 蓝色
            ItemQuality::Epic => "§p",        // 紫色
            ItemQuality::Legendary => "§o",   // 橙色
            ItemQuality::Mythic => "§r",      // 红色
        }
    }

    /// 获取品质名称
    pub fn name(&self) -> &str {
        match self {
            ItemQuality::Common => "普通",
            ItemQuality::Uncommon => "优秀",
            ItemQuality::Rare => "稀有",
            ItemQuality::Epic => "史诗",
            ItemQuality::Legendary => "传说",
            ItemQuality::Mythic => "神话",
        }
    }

    /// 从等级获取品质
    pub fn from_level(level: u32) -> Self {
        match level {
            0..=10 => ItemQuality::Common,
            11..=30 => ItemQuality::Uncommon,
            31..=60 => ItemQuality::Rare,
            61..=100 => ItemQuality::Epic,
            101..=150 => ItemQuality::Legendary,
            _ => ItemQuality::Mythic,
        }
    }
}

/// 物品基类 (对应 /gamenv/clone/item/ 中的基础物品)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// 物品ID (唯一)
    pub id: ObjectId,
    /// 物品模板ID (用于同类型物品)
    pub template_id: String,
    /// 物品名称
    pub name: String,
    /// 物品中文名
    pub name_cn: String,
    /// 物品描述
    pub desc: String,
    /// 物品类型
    pub item_type: ItemType,
    /// 物品品质
    pub quality: ItemQuality,
    /// 物品等级
    pub level: u32,
    /// 堆叠数量
    pub quantity: u32,
    /// 最大堆叠数
    pub max_stack: u32,
    /// 是否可交易
    pub tradable: bool,
    /// 是否可丢弃
    pub droppable: bool,
    /// 是否绑定到玩家
    pub bound_to: Option<String>,
    /// 物品创建时间
    pub created_at: i64,
    /// 物品过期时间 (0表示永不过期)
    pub expire_at: i64,
    /// 扩展属性 (用于存储额外数据)
    pub extra_data: serde_json::Value,
}

impl Item {
    /// 创建新物品
    pub fn new(name: String, name_cn: String, item_type: ItemType) -> Self {
        Self {
            id: ObjectId::new(),
            template_id: name.clone(),
            name,
            name_cn,
            desc: String::new(),
            item_type,
            quality: ItemQuality::Common,
            level: 1,
            quantity: 1,
            max_stack: 1,
            tradable: true,
            droppable: true,
            bound_to: None,
            created_at: chrono::Utc::now().timestamp(),
            expire_at: 0,
            extra_data: serde_json::json!({}),
        }
    }

    /// 设置描述
    pub fn with_desc(mut self, desc: String) -> Self {
        self.desc = desc;
        self
    }

    /// 设置品质
    pub fn with_quality(mut self, quality: ItemQuality) -> Self {
        self.quality = quality;
        self
    }

    /// 设置等级
    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    /// 设置数量
    pub fn with_quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    /// 设置最大堆叠数
    pub fn with_max_stack(mut self, max_stack: u32) -> Self {
        self.max_stack = max_stack;
        self
    }

    /// 设置是否可交易
    pub fn with_tradable(mut self, tradable: bool) -> Self {
        self.tradable = tradable;
        self
    }

    /// 设置是否可丢弃
    pub fn with_droppable(mut self, droppable: bool) -> Self {
        self.droppable = droppable;
        self
    }

    /// 绑定到玩家
    pub fn bind_to(&mut self, player_id: String) {
        self.bound_to = Some(player_id);
    }

    /// 检查物品是否已绑定
    pub fn is_bound(&self) -> bool {
        self.bound_to.is_some()
    }

    /// 检查物品是否已过期
    pub fn is_expired(&self) -> bool {
        if self.expire_at == 0 {
            return false;
        }
        chrono::Utc::now().timestamp() > self.expire_at
    }

    /// 增加数量
    pub fn add_quantity(&mut self, amount: u32) -> Result<()> {
        if self.quantity + amount > self.max_stack {
            return Err(MudError::InvalidOperation("超过最大堆叠数".to_string()));
        }
        self.quantity += amount;
        Ok(())
    }

    /// 减少数量
    pub fn reduce_quantity(&mut self, amount: u32) -> Result<()> {
        if self.quantity < amount {
            return Err(MudError::InvalidOperation("数量不足".to_string()));
        }
        self.quantity -= amount;
        Ok(())
    }

    /// 渲染物品名称 (带品质颜色)
    pub fn render_name(&self) -> String {
        format!("{}{}§r", self.quality.color_code(), self.name_cn)
    }

    /// 渲染物品详细信息
    pub fn render_info(&self) -> String {
        let mut info = String::new();
        info.push_str(&format!("{}【{}】{}§r\n",
            self.quality.color_code(),
            self.quality.name(),
            self.name_cn
        ));

        if !self.desc.is_empty() {
            info.push_str(&format!("{}\n", self.desc));
        }

        info.push_str(&format!("等级: {}\n", self.level));

        if self.quantity > 1 {
            info.push_str(&format!("数量: {}/{}\n", self.quantity, self.max_stack));
        }

        if self.is_bound() {
            info.push_str("§c已绑定§r\n");
        }

        if !self.tradable {
            info.push_str("§c不可交易§r\n");
        }

        if !self.droppable {
            info.push_str("§c不可丢弃§r\n");
        }

        info
    }

    /// 检查是否可以堆叠
    pub fn can_stack_with(&self, other: &Item) -> bool {
        self.template_id == other.template_id
            && self.max_stack > 1
            && self.quality == other.quality
            && self.level == other.level
    }
}

/// 物品类别
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemCategory {
    /// 武器
    Weapon,
    /// 头盔
    Helmet,
    /// 衣服
    Armor,
    /// 手套
    Gloves,
    /// 鞋子
    Boots,
    /// 腰带
    Belt,
    /// 护身符
    Amulet,
    /// 戒指
    Ring,
    /// 药品
    Medicine,
    /// 食物
    Food,
    /// 材料
    Material,
    /// 任务物品
    Quest,
    /// 其他
    Other,
}

/// 物品管理器
pub struct ItemManager {
    /// 物品模板缓存
    templates: std::collections::HashMap<String, Item>,
}

impl ItemManager {
    pub fn new() -> Self {
        Self {
            templates: std::collections::HashMap::new(),
        }
    }

    /// 创建物品实例
    pub fn create_item(&self, template_id: &str) -> Result<Item> {
        if let Some(template) = self.templates.get(template_id) {
            let mut item = template.clone();
            item.id = ObjectId::new();
            item.created_at = chrono::Utc::now().timestamp();
            Ok(item)
        } else {
            // 返回一个默认物品
            Ok(Item::new(
                template_id.to_string(),
                template_id.to_string(),
                ItemType::Normal,
            ))
        }
    }

    /// 注册物品模板
    pub fn register_template(&mut self, item: Item) {
        self.templates.insert(item.template_id.clone(), item);
    }
}

impl Default for ItemManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局物品管理器
pub static ITEMD: once_cell::sync::Lazy<std::sync::Mutex<ItemManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(ItemManager::default()));
