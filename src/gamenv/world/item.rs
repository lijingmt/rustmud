// gamenv/world/item.rs - 物品系统

use serde::{Deserialize, Serialize};

/// 物品特效
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemEffect {
    /// 特效类型
    pub effect_type: String,
    /// 效果值
    pub value: i32,
    /// 持续时间（秒，0表示永久）
    pub duration: i32,
}

/// 物品模板
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemTemplate {
    /// 物品ID
    pub id: String,
    /// 物品名称
    pub name: String,
    /// 物品类型
    #[serde(rename = "item_type")]
    pub item_type: ItemType,
    /// 子类型
    #[serde(default = "default_string")]
    pub subtype: String,
    /// 描述
    #[serde(default = "default_string")]
    pub description: String,
    /// 品质
    pub quality: ItemQuality,
    /// 等级要求
    pub level: i32,
    /// 属性加成
    #[serde(default)]
    pub stats: ItemStats,
    /// 价格/价值
    #[serde(default)]
    pub price: i32,
    /// 是否可堆叠
    #[serde(default)]
    pub stackable: bool,
    /// 物品特效列表
    #[serde(default)]
    pub effects: Vec<ItemEffect>,
}

fn default_string() -> String { String::new() }

impl Default for ItemTemplate {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            item_type: ItemType::Other,
            subtype: String::new(),
            description: String::new(),
            quality: ItemQuality::Common,
            level: 1,
            stats: ItemStats::default(),
            price: 0,
            stackable: false,
            effects: vec![],
        }
    }
}

/// 物品类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType {
    /// 消耗品
    Consumable,
    /// 武器
    Weapon,
    /// 防具
    Armor,
    /// 首饰
    Accessory,
    /// 任务物品
    Quest,
    /// 材料
    Material,
    /// 其他
    Other,
    /// 书籍（技能书）
    Book,
    /// 护身符（临时效果）
    Charm,
    /// 药水
    Potion,
    /// 工具
    Tool,
    /// 坐骑
    Mount,
    /// 宝物
    Treasure,
}

/// 物品品质
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemQuality {
    /// 普通
    Common,
    /// 优秀
    Uncommon,
    /// 稀有
    Rare,
    /// 史诗
    Epic,
    /// 传说
    Legendary,
}

impl ItemQuality {
    /// 获取颜色代码
    pub fn color_code(&self) -> &'static str {
        match self {
            ItemQuality::Common => "#888888",
            ItemQuality::Uncommon => "#1eff00",
            ItemQuality::Rare => "#0070dd",
            ItemQuality::Epic => "#a335ee",
            ItemQuality::Legendary => "#ff8000",
        }
    }

    /// 获取中文名称
    pub fn chinese_name(&self) -> &'static str {
        match self {
            ItemQuality::Common => "普通",
            ItemQuality::Uncommon => "优秀",
            ItemQuality::Rare => "稀有",
            ItemQuality::Epic => "史诗",
            ItemQuality::Legendary => "传说",
        }
    }
}

/// 物品属性
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ItemStats {
    /// 攻击力
    pub attack: i32,
    /// 防御力
    pub defense: i32,
    /// HP加成
    pub hp_bonus: i32,
    /// MP加成
    pub mp_bonus: i32,
    /// 暴击率
    #[serde(default)]
    pub crit_rate: i32,
    /// 暴击伤害
    #[serde(default)]
    pub crit_damage: i32,
}

/// 物品实例
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemInstance {
    /// 物品ID
    pub id: String,
    /// 模板ID
    pub template_id: String,
    /// 数量
    pub count: i32,
    /// 强化等级
    #[serde(default)]
    pub enhance_level: i32,
    /// 自定义属性
    #[serde(default)]
    pub custom_stats: Option<ItemStats>,
}

impl ItemTemplate {
    /// 创建实例
    pub fn create_instance(&self, count: i32) -> ItemInstance {
        ItemInstance {
            id: format!("{}_{}", self.id, uuid::Uuid::new_v4()),
            template_id: self.id.clone(),
            count,
            enhance_level: 0,
            custom_stats: None,
        }
    }

    /// 格式化描述
    pub fn format(&self) -> String {
        let quality_name = self.quality.chinese_name();
        format!(
            "[{}]{} - {}",
            quality_name,
            self.name,
            self.description
        )
    }

    /// 格式化属性
    pub fn format_stats(&self) -> String {
        let mut parts = vec![];
        if self.stats.attack > 0 {
            parts.push(format!("攻击+{}", self.stats.attack));
        }
        if self.stats.defense > 0 {
            parts.push(format!("防御+{}", self.stats.defense));
        }
        if self.stats.hp_bonus > 0 {
            parts.push(format!("HP+{}", self.stats.hp_bonus));
        }
        if self.stats.mp_bonus > 0 {
            parts.push(format!("MP+{}", self.stats.mp_bonus));
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join(" ")
        }
    }
}

impl ItemInstance {
    /// 获取物品名称（含数量）
    pub fn display_name(&self, template: &ItemTemplate) -> String {
        if template.stackable && self.count > 1 {
            format!("{} x{}", template.name, self.count)
        } else {
            template.name.clone()
        }
    }

    /// 是否可堆叠
    pub fn can_stack(&self, other: &ItemInstance) -> bool {
        self.template_id == other.template_id
            && self.enhance_level == other.enhance_level
    }

    /// 增加数量
    pub fn add_count(&mut self, amount: i32) {
        self.count += amount;
    }

    /// 减少数量
    pub fn reduce_count(&mut self, amount: i32) -> bool {
        if self.count >= amount {
            self.count -= amount;
            true
        } else {
            false
        }
    }

    /// 获取总属性（包含强化加成）
    pub fn get_total_stats(&self, base: &ItemTemplate) -> ItemStats {
        let mut stats = base.stats.clone();

        // 强化加成：每级+5%
        let bonus_percent = 1.0 + (self.enhance_level as f32 * 0.05);
        stats.attack = (stats.attack as f32 * bonus_percent) as i32;
        stats.defense = (stats.defense as f32 * bonus_percent) as i32;

        // 自定义属性覆盖
        if let Some(custom) = &self.custom_stats {
            if custom.attack > 0 {
                stats.attack = custom.attack;
            }
            if custom.defense > 0 {
                stats.defense = custom.defense;
            }
        }

        stats
    }
}

/// 背包物品
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item: ItemInstance,
    pub slot: Option<i32>,
}

/// 背包
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
    pub capacity: i32,
}

impl Inventory {
    pub fn new(capacity: i32) -> Self {
        Self {
            items: vec![],
            capacity,
        }
    }

    /// 添加物品
    pub fn add_item(&mut self, template: &ItemTemplate, count: i32) -> Result<(), String> {
        // 检查容量
        if self.items.len() >= self.capacity as usize {
            return Err("背包已满".to_string());
        }

        // 如果可堆叠，先尝试合并
        if template.stackable {
            for inv_item in &mut self.items {
                if inv_item.item.template_id == template.id {
                    inv_item.item.add_count(count);
                    return Ok(());
                }
            }
        }

        // 创建新物品
        let instance = template.create_instance(count);
        self.items.push(InventoryItem {
            item: instance,
            slot: None,
        });

        Ok(())
    }

    /// 移除物品
    pub fn remove_item(&mut self, template_id: &str, count: i32) -> Result<(), String> {
        let mut remaining = count;

        for inv_item in &mut self.items {
            if inv_item.item.template_id == template_id {
                if inv_item.item.count >= remaining {
                    inv_item.item.reduce_count(remaining);
                    remaining = 0;
                    break;
                } else {
                    remaining -= inv_item.item.count;
                    inv_item.item.count = 0;
                }
            }
        }

        // 清空数量为0的物品
        self.items.retain(|x| x.item.count > 0);

        if remaining == 0 {
            Ok(())
        } else {
            Err(format!("物品数量不足，还需要{}", remaining))
        }
    }

    /// 获取物品数量
    pub fn get_item_count(&self, template_id: &str) -> i32 {
        self.items
            .iter()
            .filter(|x| x.item.template_id == template_id)
            .map(|x| x.item.count)
            .sum()
    }

    /// 是否有物品
    pub fn has_item(&self, template_id: &str, count: i32) -> bool {
        self.get_item_count(template_id) >= count
    }
}
