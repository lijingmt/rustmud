// gamenv/item/equipment.rs - 装备系统
// 对应 txpike9/gamenv/clone/item/ 中的装备类

use crate::core::*;
use crate::gamenv::item::item::{Item, ItemQuality, ItemType};
use crate::gamenv::item::weapon::WeaponType;
use serde::{Deserialize, Serialize};

/// 装备位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipSlot {
    /// 武器
    Weapon = 0,
    /// 头盔
    Helmet = 1,
    /// 衣服
    Armor = 2,
    /// 手套
    Gloves = 3,
    /// 鞋子
    Boots = 4,
    /// 腰带
    Belt = 5,
    /// 护身符
    Amulet = 6,
    /// 戒指1
    Ring1 = 7,
    /// 戒指2
    Ring2 = 8,
}

impl EquipSlot {
    /// 获取所有装备位置
    pub fn all_slots() -> Vec<EquipSlot> {
        vec![
            EquipSlot::Weapon,
            EquipSlot::Helmet,
            EquipSlot::Armor,
            EquipSlot::Gloves,
            EquipSlot::Boots,
            EquipSlot::Belt,
            EquipSlot::Amulet,
            EquipSlot::Ring1,
            EquipSlot::Ring2,
        ]
    }

    /// 获取位置名称
    pub fn name(&self) -> &str {
        match self {
            EquipSlot::Weapon => "武器",
            EquipSlot::Helmet => "头盔",
            EquipSlot::Armor => "衣服",
            EquipSlot::Gloves => "手套",
            EquipSlot::Boots => "鞋子",
            EquipSlot::Belt => "腰带",
            EquipSlot::Amulet => "护身符",
            EquipSlot::Ring1 => "戒指1",
            EquipSlot::Ring2 => "戒指2",
        }
    }

    /// 从字符串解析装备位置
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "weapon" | "武器" => Some(EquipSlot::Weapon),
            "helmet" | "头盔" => Some(EquipSlot::Helmet),
            "armor" | "衣服" => Some(EquipSlot::Armor),
            "gloves" | "手套" => Some(EquipSlot::Gloves),
            "boots" | "鞋子" => Some(EquipSlot::Boots),
            "belt" | "腰带" => Some(EquipSlot::Belt),
            "amulet" | "护身符" => Some(EquipSlot::Amulet),
            "ring" | "戒指" => Some(EquipSlot::Ring1),
            _ => None,
        }
    }
}

/// 装备属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipStats {
    /// 攻击力
    pub attack: u32,
    /// 防御力
    pub defense: u32,
    /// 生命值上限
    pub hp_max: u32,
    /// 内力值上限
    pub qi_max: u32,
    /// 精神值上限
    pub shen_max: u32,
    /// 暴击率
    pub crit_rate: u32,
    /// 暴击伤害
    pub crit_damage: u32,
    /// 命中率
    pub hit_rate: u32,
    /// 闪避率
    pub dodge_rate: u32,
}

impl Default for EquipStats {
    fn default() -> Self {
        Self {
            attack: 0,
            defense: 0,
            hp_max: 0,
            qi_max: 0,
            shen_max: 0,
            crit_rate: 0,
            crit_damage: 0,
            hit_rate: 0,
            dodge_rate: 0,
        }
    }
}

impl EquipStats {
    /// 计算装备属性 (基于等级和品质)
    pub fn calculate(level: u32, quality: ItemQuality, slot: EquipSlot) -> Self {
        let base = level * match quality {
            ItemQuality::Common => 1,
            ItemQuality::Uncommon => 2,
            ItemQuality::Rare => 4,
            ItemQuality::Epic => 8,
            ItemQuality::Legendary => 16,
            ItemQuality::Mythic => 32,
        };

        let mut stats = Self::default();

        match slot {
            EquipSlot::Weapon => {
                stats.attack = base;
                stats.crit_rate = base / 10;
            }
            EquipSlot::Helmet => {
                stats.defense = base / 2;
                stats.hp_max = base * 5;
            }
            EquipSlot::Armor => {
                stats.defense = base;
                stats.hp_max = base * 10;
            }
            EquipSlot::Gloves => {
                stats.defense = base / 3;
                stats.hit_rate = base / 10;
            }
            EquipSlot::Boots => {
                stats.defense = base / 3;
                stats.dodge_rate = base / 10;
            }
            EquipSlot::Belt => {
                stats.hp_max = base * 3;
                stats.qi_max = base * 2;
            }
            EquipSlot::Amulet => {
                stats.shen_max = base * 3;
                stats.hp_max = base * 2;
            }
            EquipSlot::Ring1 | EquipSlot::Ring2 => {
                stats.attack = base / 5;
                stats.hp_max = base * 2;
            }
        }

        stats
    }

    /// 添加属性
    pub fn add(&mut self, other: &EquipStats) {
        self.attack += other.attack;
        self.defense += other.defense;
        self.hp_max += other.hp_max;
        self.qi_max += other.qi_max;
        self.shen_max += other.shen_max;
        self.crit_rate += other.crit_rate;
        self.crit_damage += other.crit_damage;
        self.hit_rate += other.hit_rate;
        self.dodge_rate += other.dodge_rate;
    }
}

/// 装备品级 (对应境界装备颜色)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EquipRealm {
    /// 凡人 (白色)
    Mortal = 0,
    /// 筑基 (绿色)
    Foundation = 1,
    /// 金丹 (蓝色)
    GoldenCore = 2,
    /// 元婴 (紫色)
    NascentSoul = 3,
    /// 化神 (橙色)
    SoulFormation = 4,
    /// 炼虚 (红色)
    VoidRefinement = 5,
    /// 合体 (暗红)
    Unity = 6,
    /// 大乘 (金色)
    Mahayana = 7,
    /// 渡劫 (彩虹)
    Tribulation = 8,
    /// 大道 (七彩)
    GreatDao = 9,
}

impl Default for EquipRealm {
    fn default() -> Self {
        EquipRealm::Mortal
    }
}

impl EquipRealm {
    /// 获取境界对应的颜色代码
    pub fn color_code(&self) -> &str {
        match self {
            EquipRealm::Mortal => "§w",
            EquipRealm::Foundation => "§g",
            EquipRealm::GoldenCore => "§b",
            EquipRealm::NascentSoul => "§p",
            EquipRealm::SoulFormation => "§o",
            EquipRealm::VoidRefinement => "§r",
            EquipRealm::Unity => "§dr",
            EquipRealm::Mahayana => "§y",
            EquipRealm::Tribulation => "§rb",
            EquipRealm::GreatDao => "§qc",
        }
    }

    /// 获取境界名称
    pub fn name(&self) -> &str {
        match self {
            EquipRealm::Mortal => "凡人",
            EquipRealm::Foundation => "筑基",
            EquipRealm::GoldenCore => "金丹",
            EquipRealm::NascentSoul => "元婴",
            EquipRealm::SoulFormation => "化神",
            EquipRealm::VoidRefinement => "炼虚",
            EquipRealm::Unity => "合体",
            EquipRealm::Mahayana => "大乘",
            EquipRealm::Tribulation => "渡劫",
            EquipRealm::GreatDao => "大道",
        }
    }

    /// 从等级获取境界
    pub fn from_level(level: u32) -> Self {
        match level {
            0..=10 => EquipRealm::Mortal,
            11..=30 => EquipRealm::Foundation,
            31..=60 => EquipRealm::GoldenCore,
            61..=100 => EquipRealm::NascentSoul,
            101..=150 => EquipRealm::SoulFormation,
            151..=200 => EquipRealm::VoidRefinement,
            201..=300 => EquipRealm::Unity,
            301..=500 => EquipRealm::Mahayana,
            501..=1000 => EquipRealm::Tribulation,
            _ => EquipRealm::GreatDao,
        }
    }
}

/// 装备 (对应 /gamenv/clone/item/ 中的装备)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    /// 基础物品数据
    #[serde(flatten)]
    pub item: Item,
    /// 装备位置
    pub slot: EquipSlot,
    /// 装备属性
    pub stats: EquipStats,
    /// 装备境界
    pub realm: EquipRealm,
    /// 强化等级 (+0 ~ +15)
    pub reinforce_level: u32,
    /// 是否已套装激活
    pub suit_active: bool,
    /// 套装ID
    pub suit_id: Option<String>,
}

impl Equipment {
    /// 创建新装备
    pub fn new(name: String, name_cn: String, slot: EquipSlot) -> Self {
        let item = Item::new(name.clone(), name_cn.clone(), ItemType::Weapon);
        Self {
            item,
            slot,
            stats: EquipStats::default(),
            realm: EquipRealm::Mortal,
            reinforce_level: 0,
            suit_active: false,
            suit_id: None,
        }
    }

    /// 设置属性
    pub fn with_stats(mut self, stats: EquipStats) -> Self {
        self.stats = stats;
        self
    }

    /// 设置境界
    pub fn with_realm(mut self, realm: EquipRealm) -> Self {
        self.realm = realm;
        self
    }

    /// 设置套装
    pub fn with_suit(mut self, suit_id: String) -> Self {
        self.suit_id = Some(suit_id);
        self
    }

    /// 计算装备属性 (基于等级、品质、强化)
    pub fn calculate_stats(&mut self) {
        let base_stats = EquipStats::calculate(
            self.item.level,
            self.item.quality,
            self.slot
        );

        let reinforce_bonus = 1.0 + (self.reinforce_level as f32) * 0.1;

        self.stats.attack = (base_stats.attack as f32 * reinforce_bonus) as u32;
        self.stats.defense = (base_stats.defense as f32 * reinforce_bonus) as u32;
        self.stats.hp_max = (base_stats.hp_max as f32 * reinforce_bonus) as u32;
        self.stats.qi_max = (base_stats.qi_max as f32 * reinforce_bonus) as u32;
        self.stats.shen_max = (base_stats.shen_max as f32 * reinforce_bonus) as u32;
    }

    /// 强化装备
    pub fn reinforce(&mut self) -> Result<()> {
        if self.reinforce_level >= 15 {
            return Err(MudError::InvalidOperation("装备已达到最大强化等级".to_string()));
        }
        self.reinforce_level += 1;
        self.calculate_stats();
        Ok(())
    }

    /// 检查是否可以装备
    pub fn can_equip(&self, player_level: u32) -> bool {
        self.item.level <= player_level
    }

    /// 渲染装备信息
    pub fn render_info(&self) -> String {
        let mut info = String::new();

        // 名称 + 境界颜色 + 强化等级
        if self.reinforce_level > 0 {
            info.push_str(&format!("{}+{}{}§r ",
                self.realm.color_code(),
                self.reinforce_level,
                self.item.name_cn
            ));
        } else {
            info.push_str(&format!("{}{}§r ",
                self.realm.color_code(),
                self.item.name_cn
            ));
        }

        info.push_str(&format!("[{}]\n", self.slot.name()));
        info.push_str(&format!("境界: {}{}\n", self.realm.color_code(), self.realm.name()));
        info.push_str(&format!("品质: {}{}\n", self.item.quality.color_code(), self.item.quality.name()));
        info.push_str(&format!("等级: {}\n", self.item.level));

        // 显示属性
        if self.stats.attack > 0 {
            info.push_str(&format!("攻击: +{}\n", self.stats.attack));
        }
        if self.stats.defense > 0 {
            info.push_str(&format!("防御: +{}\n", self.stats.defense));
        }
        if self.stats.hp_max > 0 {
            info.push_str(&format!("生命: +{}\n", self.stats.hp_max));
        }
        if self.stats.qi_max > 0 {
            info.push_str(&format!("内力: +{}\n", self.stats.qi_max));
        }
        if self.stats.shen_max > 0 {
            info.push_str(&format!("精神: +{}\n", self.stats.shen_max));
        }
        if self.stats.crit_rate > 0 {
            info.push_str(&format!("暴击: +{}%\n", self.stats.crit_rate));
        }
        if self.stats.dodge_rate > 0 {
            info.push_str(&format!("闪避: +{}%\n", self.stats.dodge_rate));
        }

        if let Some(ref suit_id) = self.suit_id {
            info.push_str(&format!("套装: {}\n", suit_id));
        }

        info
    }
}

/// 装备栏 (玩家的装备状态)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentSlots {
    /// 当前装备的物品
    pub equipped: std::collections::HashMap<EquipSlot, Equipment>,
}

impl Default for EquipmentSlots {
    fn default() -> Self {
        Self {
            equipped: std::collections::HashMap::new(),
        }
    }
}

impl EquipmentSlots {
    /// 装备物品
    pub fn equip(&mut self, slot: EquipSlot, equipment: Equipment) -> Result<Option<Equipment>> {
        if equipment.slot != slot {
            return Err(MudError::InvalidOperation("装备位置不匹配".to_string()));
        }
        Ok(self.equipped.insert(slot, equipment))
    }

    /// 卸下装备
    pub fn unequip(&mut self, slot: EquipSlot) -> Result<Equipment> {
        self.equipped.remove(&slot)
            .ok_or_else(|| MudError::NotFound(format!("装备位{}为空", slot.name())))
    }

    /// 获取装备
    pub fn get(&self, slot: EquipSlot) -> Option<&Equipment> {
        self.equipped.get(&slot)
    }

    /// 计算总属性
    pub fn total_stats(&self) -> EquipStats {
        let mut total = EquipStats::default();
        for equipment in self.equipped.values() {
            total.add(&equipment.stats);
        }
        total
    }

    /// 渲染装备状态
    pub fn render(&self) -> String {
        let mut result = String::from("=== 装备 ===\n");
        for slot in EquipSlot::all_slots() {
            if let Some(equip) = self.get(slot) {
                result.push_str(&format!("{}: {}{}§r\n",
                    slot.name(),
                    equip.realm.color_code(),
                    equip.item.name_cn
                ));
            } else {
                result.push_str(&format!("{}: [空]\n", slot.name()));
            }
        }
        result
    }
}

/// 装备管理器
pub struct EquipmentManager {
    /// 装备模板缓存
    templates: std::collections::HashMap<String, Equipment>,
}

impl EquipmentManager {
    pub fn new() -> Self {
        Self {
            templates: std::collections::HashMap::new(),
        }
    }

    /// 创建装备实例
    pub fn create_equipment(&self, template_id: &str) -> Result<Equipment> {
        if let Some(template) = self.templates.get(template_id) {
            let mut equip = template.clone();
            equip.item.id = ObjectId::new();
            equip.item.created_at = chrono::Utc::now().timestamp();
            equip.calculate_stats();
            Ok(equip)
        } else {
            Err(MudError::NotFound(format!("装备模板不存在: {}", template_id)))
        }
    }

    /// 注册装备模板
    pub fn register_template(&mut self, equipment: Equipment) {
        self.templates.insert(equipment.item.template_id.clone(), equipment);
    }
}

impl Default for EquipmentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局装备管理器
pub static EQUIPD: once_cell::sync::Lazy<std::sync::Mutex<EquipmentManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(EquipmentManager::default()));
