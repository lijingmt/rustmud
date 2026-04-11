// gamenv/clone/item.rs - 物品模板系统
// 对应 txpike9/gamenv/clone/item/ 目录
//
// 物品模板用于创建可克隆的物品实例

use crate::gamenv::item::{Item, ItemType, ItemQuality};
use crate::gamenv::item::weapon::WeaponType;
use crate::core::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 物品模板 - 定义物品的基础属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTemplate {
    /// 模板ID (唯一标识)
    pub id: String,
    /// 物品名称
    pub name: String,
    /// 中文名称
    pub name_cn: String,
    /// 物品描述
    pub description: String,
    /// 单位
    pub unit: String,
    /// 物品类型
    pub item_type: ItemType,
    /// 等级要求
    pub level: u32,
    /// 价值
    pub value: u64,
    /// 重量
    pub weight: u32,
    /// 图片文件名
    pub picture: Option<String>,
    /// 扩展属性 (attack, defense, etc.)
    pub extra_data: HashMap<String, serde_json::Value>,
    /// 是否自动保存
    pub autoload: bool,
}

impl ItemTemplate {
    /// 创建新物品模板
    pub fn new(
        id: String,
        name: String,
        name_cn: String,
        item_type: ItemType,
    ) -> Self {
        Self {
            id,
            name,
            name_cn,
            description: String::new(),
            unit: "个".to_string(),
            item_type,
            level: 1,
            value: 0,
            weight: 100,
            picture: None,
            extra_data: HashMap::new(),
            autoload: false,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// 设置等级
    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    /// 设置价值
    pub fn with_value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }

    /// 设置重量
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// 设置攻击力 (武器)
    pub fn with_attack(mut self, attack: i32) -> Self {
        self.extra_data.insert("attack".to_string(), serde_json::json!(attack));
        self
    }

    /// 设置防御力 (防具)
    pub fn with_defense(mut self, defense: i32) -> Self {
        self.extra_data.insert("defense".to_string(), serde_json::json!(defense));
        self
    }

    /// 设置武器类型
    pub fn with_weapon_type(mut self, weapon_type: WeaponType) -> Self {
        self.extra_data.insert("weapon_type".to_string(), serde_json::json!(format!("{:?}", weapon_type)));
        self
    }

    /// 设置耐久度
    pub fn with_durability(mut self, current: i32, max: i32) -> Self {
        self.extra_data.insert("eff_dura".to_string(), serde_json::json!(current));
        self.extra_data.insert("max_dura".to_string(), serde_json::json!(max));
        self
    }

    /// 设置单位
    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    /// 设置自动加载
    pub fn with_autoload(mut self, autoload: bool) -> Self {
        self.autoload = autoload;
        self
    }

    /// 从模板创建物品实例
    pub fn instantiate(&self) -> Item {
        // 将 HashMap 转换为 serde_json::Value::Object
        let extra_data_value = serde_json::json!(self.extra_data);

        Item {
            id: ObjectId::new(),
            template_id: self.id.clone(),
            name: self.name.clone(),
            name_cn: self.name_cn.clone(),
            desc: self.description.clone(),
            item_type: self.item_type.clone(),
            quality: ItemQuality::from_level(self.level),
            level: self.level,
            quantity: 1,
            max_stack: 1,
            tradable: true,
            droppable: true,
            bound_to: None,
            created_at: chrono::Utc::now().timestamp(),
            expire_at: 0,
            extra_data: extra_data_value,
        }
    }
}

/// 物品模板注册表 - 全局单例
pub struct ItemTemplateRegistry {
    templates: HashMap<String, Arc<ItemTemplate>>,
}

impl ItemTemplateRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// 注册模板
    pub fn register(&mut self, template: ItemTemplate) {
        let id = template.id.clone();
        self.templates.insert(id, Arc::new(template));
    }

    /// 获取模板
    pub fn get(&self, id: &str) -> Option<Arc<ItemTemplate>> {
        self.templates.get(id).cloned()
    }

    /// 通过模板创建物品
    pub fn create_item(&self, template_id: &str) -> Option<Item> {
        self.get(template_id).map(|t| t.instantiate())
    }

    /// 列出所有模板ID
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
}

impl Default for ItemTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局物品模板注册表
pub static ITEM_TEMPLATE_REGISTRY: tokio::sync::OnceCell<Arc<RwLock<ItemTemplateRegistry>>> = tokio::sync::OnceCell::const_new();

/// 初始化物品模板注册表
pub async fn init_item_templates() {
    let registry = Arc::new(RwLock::new(ItemTemplateRegistry::new()));
    let mut reg = registry.write().await;

    // 注册基础武器
    reg.register(ItemTemplate::new(
        "weapon/basic_sword".to_string(),
        "basic_sword".to_string(),
        "基础铁剑".to_string(),
        ItemType::Weapon,
    )
    .with_description("一把普通的铁剑，新手冒险者的标配。\\n")
    .with_level(1)
    .with_value(100)
    .with_weight(1500)
    .with_attack(10)
    .with_durability(100, 100)
    .with_weapon_type(WeaponType::Sword)
    .with_unit("把"));

    // 注册精良武器
    reg.register(ItemTemplate::new(
        "weapon/steel_sword".to_string(),
        "steel_sword".to_string(),
        "精钢剑".to_string(),
        ItemType::Weapon,
    )
    .with_description("一把用精钢打造的利剑，寒光闪闪。\\n")
    .with_level(10)
    .with_value(5000)
    .with_weight(2000)
    .with_attack(25)
    .with_durability(150, 150)
    .with_weapon_type(WeaponType::Sword)
    .with_unit("把"));

    // 注册基础防具
    reg.register(ItemTemplate::new(
        "armor/basic_vest".to_string(),
        "basic_vest".to_string(),
        "布衣".to_string(),
        ItemType::Armor,
    )
    .with_description("普通的布制上衣，提供基本的防护。\\n")
    .with_level(1)
    .with_value(50)
    .with_weight(500)
    .with_defense(5)
    .with_unit("件"));

    // 注册药品
    reg.register(ItemTemplate::new(
        "consumable/basic_pill".to_string(),
        "basic_pill".to_string(),
        "小还丹".to_string(),
        ItemType::Medicine,
    )
    .with_description("一颗散发着药香的小丹药，可恢复50点生命。\\n")
    .with_level(1)
    .with_value(20)
    .with_weight(10)
    .with_unit("颗"));

    drop(reg);
    ITEM_TEMPLATE_REGISTRY.set(registry).ok().unwrap();
}

/// 获取物品模板注册表
pub async fn get_item_registry() -> Arc<RwLock<ItemTemplateRegistry>> {
    ITEM_TEMPLATE_REGISTRY.get()
        .expect("Item template registry not initialized")
        .clone()
}

/// 通过模板ID创建物品
pub async fn create_item_from_template(template_id: &str) -> Option<Item> {
    let registry = get_item_registry().await;
    let reg = registry.read().await;
    reg.create_item(template_id)
}
