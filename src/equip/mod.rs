// equip/mod.rs - 装备系统（完全对应 txpike9）
// 使用 JSON 配置驱动

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::core::{MudError, Result};

// ============================================================================
// 装备位置
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipSlot {
    Weapon,      // 武器
    Helmet,      // 头盔
    Armor,       // 衣服/盔甲
    Gloves,      // 手套
    Boots,       // 鞋子
    Belt,        // 腰带
    Amulet,      // 护身符
    Ring,        // 戒指（可装备2个）
}

impl EquipSlot {
    pub fn zh_name(&self) -> &str {
        match self {
            Self::Weapon => "武器",
            Self::Helmet => "头盔",
            Self::Armor => "衣服",
            Self::Gloves => "手套",
            Self::Boots => "鞋子",
            Self::Belt => "腰带",
            Self::Amulet => "护身符",
            Self::Ring => "戒指",
        }
    }
}

// ============================================================================
// 装备品质（对应 ItemQuality）
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EquipQuality {
    Common,    // 普通 - 灰色 - 1倍
    Uncommon,  // 优秀 - 绿色 - 2倍
    Rare,      // 稀有 - 蓝色 - 4倍
    Epic,      // 史诗 - 紫色 - 8倍
    Legendary, // 传说 - 橙色 - 16倍
    Mythic,    // 神话 - 32倍
}

impl EquipQuality {
    /// 获取品质倍率
    pub fn multiplier(&self) -> f64 {
        match self {
            Self::Common => 1.0,
            Self::Uncommon => 2.0,
            Self::Rare => 4.0,
            Self::Epic => 8.0,
            Self::Legendary => 16.0,
            Self::Mythic => 32.0,
        }
    }

    /// 获取品质颜色代码（对应 txpike9 颜色系统）
    pub fn color_code(&self) -> &str {
        match self {
            Self::Common => "#888888",     // 灰色
            Self::Uncommon => "#1eff00",   // 绿色
            Self::Rare => "#0070dd",       // 蓝色
            Self::Epic => "#a335ee",       // 紫色
            Self::Legendary => "#ff8000",  // 橙色
            Self::Mythic => "#ff0000",     // 红色（神话）
        }
    }

    /// 获取品质中文名
    pub fn zh_name(&self) -> &str {
        match self {
            Self::Common => "普通",
            Self::Uncommon => "优秀",
            Self::Rare => "稀有",
            Self::Epic => "史诗",
            Self::Legendary => "传说",
            Self::Mythic => "神话",
        }
    }

    /// 从字符串解析品质
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "common" | "普通" => Some(Self::Common),
            "uncommon" | "优秀" => Some(Self::Uncommon),
            "rare" | "稀有" => Some(Self::Rare),
            "epic" | "史诗" => Some(Self::Epic),
            "legendary" | "传说" => Some(Self::Legendary),
            "mythic" | "神话" => Some(Self::Mythic),
            _ => None,
        }
    }
}

// ============================================================================
// 装备境界（对应 EquipRealm）
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipRealm {
    Mortal,           // 凡人 - 白色 - 0-10级
    Foundation,       // 筑基 - 绿色 - 11-30级
    GoldenCore,       // 金丹 - 蓝色 - 31-60级
    NascentSoul,      // 元婴 - 紫色 - 61-100级
    SoulFormation,    // 化神 - 橙色 - 101-150级
    VoidRefinement,   // 炼虚 - 红色 - 151-200级
    Unity,            // 合体 - 暗红 - 201-300级
    Mahayana,         // 大乘 - 金色 - 301-500级
    Tribulation,      // 渡劫 - 彩虹 - 501-1000级
    GreatDao,         // 大道 - 七彩 - 1000级以上
}

impl EquipRealm {
    /// 获取境界颜色代码（对应 txpike9 § 颜色）
    pub fn color_code(&self) -> &str {
        match self {
            Self::Mortal => "§w",          // 白色
            Self::Foundation => "§g",      // 绿色
            Self::GoldenCore => "§b",      // 蓝色
            Self::NascentSoul => "§p",     // 紫色
            Self::SoulFormation => "§o",   // 橙色
            Self::VoidRefinement => "§r",  // 红色
            Self::Unity => "§dr",          // 暗红
            Self::Mahayana => "§y",        // 金色
            Self::Tribulation => "§rb",    // 彩虹
            Self::GreatDao => "§qc",       // 七彩
        }
    }

    /// 获取境界中文名
    pub fn zh_name(&self) -> &str {
        match self {
            Self::Mortal => "凡人",
            Self::Foundation => "筑基",
            Self::GoldenCore => "金丹",
            Self::NascentSoul => "元婴",
            Self::SoulFormation => "化神",
            Self::VoidRefinement => "炼虚",
            Self::Unity => "合体",
            Self::Mahayana => "大乘",
            Self::Tribulation => "渡劫",
            Self::GreatDao => "大道",
        }
    }

    /// 获取境界对应的等级范围
    pub fn level_range(&self) -> (u32, u32) {
        match self {
            Self::Mortal => (0, 10),
            Self::Foundation => (11, 30),
            Self::GoldenCore => (31, 60),
            Self::NascentSoul => (61, 100),
            Self::SoulFormation => (101, 150),
            Self::VoidRefinement => (151, 200),
            Self::Unity => (201, 300),
            Self::Mahayana => (301, 500),
            Self::Tribulation => (501, 1000),
            Self::GreatDao => (1001, u32::MAX),
        }
    }

    /// 根据等级获取境界
    pub fn from_level(level: u32) -> Self {
        match level {
            0..=10 => Self::Mortal,
            11..=30 => Self::Foundation,
            31..=60 => Self::GoldenCore,
            61..=100 => Self::NascentSoul,
            101..=150 => Self::SoulFormation,
            151..=200 => Self::VoidRefinement,
            201..=300 => Self::Unity,
            301..=500 => Self::Mahayana,
            501..=1000 => Self::Tribulation,
            _ => Self::GreatDao,
        }
    }

    /// 从字符串解析境界
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mortal" | "凡人" => Some(Self::Mortal),
            "foundation" | "筑基" => Some(Self::Foundation),
            "goldencore" | "金丹" => Some(Self::GoldenCore),
            "nescentsoul" | "元婴" => Some(Self::NascentSoul),
            "soulformation" | "化神" => Some(Self::SoulFormation),
            "voidrefinement" | "炼虚" => Some(Self::VoidRefinement),
            "unity" | "合体" => Some(Self::Unity),
            "mahayana" | "大乘" => Some(Self::Mahayana),
            "tribulation" | "渡劫" => Some(Self::Tribulation),
            "greatdao" | "大道" => Some(Self::GreatDao),
            _ => None,
        }
    }
}

// ============================================================================
// 装备属性（对应 EquipStats）
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquipStats {
    // 基础属性
    #[serde(default)]
    pub attack: u32,        // 攻击力
    #[serde(default)]
    pub defense: u32,       // 防御力
    #[serde(default)]
    pub hp_max: u32,        // 生命值上限
    #[serde(default)]
    pub qi_max: u32,        // 内力值上限
    #[serde(default)]
    pub shen_max: u32,      // 精神值上限

    // 战斗属性
    #[serde(default)]
    pub crit_rate: u32,     // 暴击率 (百分比，如10表示10%)
    #[serde(default)]
    pub crit_damage: u32,   // 暴击伤害 (百分比，如150表示150%)
    #[serde(default)]
    pub hit_rate: u32,      // 命中率
    #[serde(default)]
    pub dodge_rate: u32,    // 闪避率

    // 元素抗性
    #[serde(default)]
    pub fire_resist: i32,   // 火抗
    #[serde(default)]
    pub ice_resist: i32,    // 冰抗
    #[serde(default)]
    pub lightning_resist: i32, // 雷抗
    #[serde(default)]
    pub poison_resist: i32, // 毒抗
}

impl EquipStats {
    /// 创建新属性
    pub fn new() -> Self {
        Self::default()
    }

    /// 属性相加
    pub fn add(&self, other: &Self) -> Self {
        Self {
            attack: self.attack.saturating_add(other.attack),
            defense: self.defense.saturating_add(other.defense),
            hp_max: self.hp_max.saturating_add(other.hp_max),
            qi_max: self.qi_max.saturating_add(other.qi_max),
            shen_max: self.shen_max.saturating_add(other.shen_max),
            crit_rate: self.crit_rate.saturating_add(other.crit_rate),
            crit_damage: self.crit_damage.saturating_add(other.crit_damage),
            hit_rate: self.hit_rate.saturating_add(other.hit_rate),
            dodge_rate: self.dodge_rate.saturating_add(other.dodge_rate),
            fire_resist: self.fire_resist.saturating_add(other.fire_resist),
            ice_resist: self.ice_resist.saturating_add(other.ice_resist),
            lightning_resist: self.lightning_resist.saturating_add(other.lightning_resist),
            poison_resist: self.poison_resist.saturating_add(other.poison_resist),
        }
    }

    /// 应用倍率
    pub fn scale(&self, multiplier: f64) -> Self {
        Self {
            attack: (self.attack as f64 * multiplier) as u32,
            defense: (self.defense as f64 * multiplier) as u32,
            hp_max: (self.hp_max as f64 * multiplier) as u32,
            qi_max: (self.qi_max as f64 * multiplier) as u32,
            shen_max: (self.shen_max as f64 * multiplier) as u32,
            crit_rate: (self.crit_rate as f64 * multiplier) as u32,
            crit_damage: (self.crit_damage as f64 * multiplier) as u32,
            hit_rate: (self.hit_rate as f64 * multiplier) as u32,
            dodge_rate: (self.dodge_rate as f64 * multiplier) as u32,
            fire_resist: (self.fire_resist as f64 * multiplier) as i32,
            ice_resist: (self.ice_resist as f64 * multiplier) as i32,
            lightning_resist: (self.lightning_resist as f64 * multiplier) as i32,
            poison_resist: (self.poison_resist as f64 * multiplier) as i32,
        }
    }

    /// 是否全为0
    pub fn is_empty(&self) -> bool {
        self.attack == 0 && self.defense == 0 && self.hp_max == 0
            && self.qi_max == 0 && self.shen_max == 0
            && self.crit_rate == 0 && self.crit_damage == 0
            && self.hit_rate == 0 && self.dodge_rate == 0
    }
}

// ============================================================================
// 装备特效
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EquipEffect {
    /// 攻击时吸血
    LifeSteal { percent: u32 },
    /// 暴击率加成
    Critical { chance: u32 },
    /// 暴击伤害加成
    CriticalDamage { bonus: u32 },
    /// 破甲（忽略防御）
    ArmorBreak { percent: u32 },
    /// 反伤（受到攻击时反弹伤害）
    Thorns { percent: u32 },
    /// 闪避率加成
    Dodge { chance: u32 },
    /// 命中率加成
    Hit { chance: u32 },
    /// 攻击时触发额外伤害
    BonusDamage { value: u32 },
    /// 攻击时触发百分比伤害
    BonusDamagePercent { percent: u32 },
    /// 受到攻击时减少伤害
    DamageReduction { percent: u32 },
    /// HP每秒恢复
    HpRegen { value: u32 },
    /// 内力每秒恢复
    QiRegen { value: u32 },
    /// 技能冷却减少
    CooldownReduction { percent: u32 },
    /// 攻击有概率触发眩晕
    StunChance { chance: u32, duration: u32 },
    /// 攻击有概率触发沉默
    SilenceChance { chance: u32, duration: u32 },
    /// 免疫控制
    CrowdControlImmunity {},
    /// 移动速度加成
    MoveSpeed { bonus: u32 },
    /// 经验加成
    ExpBonus { percent: u32 },
    /// 掉落加成
    DropBonus { percent: u32 },
    /// 金币加成
    GoldBonus { percent: u32 },
    /// 自定义（通过脚本实现复杂逻辑）
    Custom { name: String, script: String },
}

impl EquipEffect {
    /// 描述特效
    pub fn describe(&self) -> String {
        match self {
            Self::LifeSteal { percent } => format!("攻击时吸血{}%", percent),
            Self::Critical { chance } => format!("暴击率+{}%", chance),
            Self::CriticalDamage { bonus } => format!("暴击伤害+{}%", bonus),
            Self::ArmorBreak { percent } => format!("破甲{}%", percent),
            Self::Thorns { percent } => format!("反伤{}%", percent),
            Self::Dodge { chance } => format!("闪避率+{}%", chance),
            Self::Hit { chance } => format!("命中率+{}%", chance),
            Self::BonusDamage { value } => format!("攻击时额外{}点伤害", value),
            Self::BonusDamagePercent { percent } => format!("攻击时额外{}%伤害", percent),
            Self::DamageReduction { percent } => format!("受到的伤害减少{}%", percent),
            Self::HpRegen { value } => format!("每秒恢复{}点生命", value),
            Self::QiRegen { value } => format!("每秒恢复{}点内力", value),
            Self::CooldownReduction { percent } => format!("技能冷却-{}%", percent),
            Self::StunChance { chance, duration } => format!("攻击时{}%概率眩晕{}回合", chance, duration),
            Self::SilenceChance { chance, duration } => format!("攻击时{}%概率沉默{}回合", chance, duration),
            Self::CrowdControlImmunity {} => "免疫控制效果".to_string(),
            Self::MoveSpeed { bonus } => format!("移动速度+{}", bonus),
            Self::ExpBonus { percent } => format!("获得经验+{}%", percent),
            Self::DropBonus { percent } => format!("掉落率+{}%", percent),
            Self::GoldBonus { percent } => format!("金币获得+{}%", percent),
            Self::Custom { name, .. } => format!("特效: {}", name),
        }
    }
}

// ============================================================================
// 装备模板（JSON配置）
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipTemplate {
    /// 模板ID（唯一标识）
    pub id: String,

    /// 装备名称
    pub name: String,

    /// 装备位置
    pub slot: EquipSlot,

    /// 品质
    pub quality: EquipQuality,

    /// 等级要求
    #[serde(default)]
    pub level_req: u32,

    /// 基础属性（品质倍率应用前的值）
    pub base_stats: EquipStats,

    /// 装备特效列表
    #[serde(default)]
    pub effects: Vec<EquipEffect>,

    /// 套装ID（如果有）
    #[serde(default)]
    pub suit_id: Option<String>,

    /// 装备描述
    #[serde(default)]
    pub description: String,

    /// 是否可交易
    #[serde(default = "default_true")]
    pub tradeable: bool,

    /// 是否可丢弃
    #[serde(default = "default_true")]
    pub droppable: bool,

    /// 售价（金币）
    #[serde(default)]
    pub sell_price: u32,
}

fn default_true() -> bool { true }

impl EquipTemplate {
    /// 计算实际属性（应用品质倍率）
    pub fn calc_stats(&self) -> EquipStats {
        self.base_stats.scale(self.quality.multiplier())
    }
}

// ============================================================================
// 装备实例（玩家实际拥有的装备）
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    /// 实例ID（唯一）
    pub id: String,

    /// 模板ID
    pub template_id: String,

    /// 装备名称（可自定义）
    pub name: String,

    /// 装备位置
    pub slot: EquipSlot,

    /// 品质
    pub quality: EquipQuality,

    /// 境界
    pub realm: EquipRealm,

    /// 强化等级 (+0 ~ +15)
    #[serde(default)]
    pub reinforce_level: u32,

    /// 基础属性（未强化前）
    pub base_stats: EquipStats,

    /// 装备特效
    #[serde(default)]
    pub effects: Vec<EquipEffect>,

    /// 套装ID
    #[serde(default)]
    pub suit_id: Option<String>,

    /// 装备描述
    #[serde(default)]
    pub description: String,

    /// 创建者ID
    pub created_by: String,

    /// 创建时间
    pub created_at: i64,

    /// 是否绑定
    #[serde(default)]
    pub bound: bool,

    /// 耐久度（当前值/最大值）
    #[serde(default)]
    pub durability: Option<(u32, u32)>,
}

impl Equipment {
    /// 从模板创建装备实例
    pub fn from_template(template: &EquipTemplate, player_id: &str) -> Self {
        Self {
            id: format!("equip_{}_{}", template.id, chrono::Utc::now().timestamp_millis()),
            template_id: template.id.clone(),
            name: template.name.clone(),
            slot: template.slot.clone(),
            quality: template.quality,
            realm: EquipRealm::from_level(template.level_req),
            reinforce_level: 0,
            base_stats: template.calc_stats(),
            effects: template.effects.clone(),
            suit_id: template.suit_id.clone(),
            description: template.description.clone(),
            created_by: player_id.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            bound: false,
            durability: None,  // 默认无耐久
        }
    }

    /// 获取最终属性（应用强化加成）
    pub fn final_stats(&self) -> EquipStats {
        // 强化公式：基础属性 × (1 + 强化等级 × 0.1)
        let reinforce_mult = 1.0 + (self.reinforce_level as f64 * 0.1);
        self.base_stats.scale(reinforce_mult)
    }

    /// 强化装备
    pub fn reinforce(&mut self) -> Result<()> {
        if self.reinforce_level >= 15 {
            return Err(MudError::RuntimeError("装备已达到最大强化等级 +15".to_string()));
        }
        self.reinforce_level += 1;
        Ok(())
    }

    /// 获取显示名称（带颜色前缀）
    pub fn display_name(&self) -> String {
        format!("{}{}[+{}]§f",
            self.realm.color_code(),
            self.name,
            self.reinforce_level
        )
    }

    /// 获取完整描述
    pub fn describe(&self) -> String {
        let stats = self.final_stats();
        let mut desc = format!("{}【{}】{}[+{}] §f\n",
            self.realm.color_code(),
            self.realm.zh_name(),
            self.name,
            self.reinforce_level
        );
        desc.push_str(&format!("品质: {} {}\n", self.quality.color_code(), self.quality.zh_name()));
        desc.push_str(&format!("位置: {}\n", self.slot.zh_name()));

        if stats.attack > 0 {
            desc.push_str(&format!("攻击: {}\n", stats.attack));
        }
        if stats.defense > 0 {
            desc.push_str(&format!("防御: {}\n", stats.defense));
        }
        if stats.hp_max > 0 {
            desc.push_str(&format!("生命: {}\n", stats.hp_max));
        }
        if stats.qi_max > 0 {
            desc.push_str(&format!("内力: {}\n", stats.qi_max));
        }
        if stats.crit_rate > 0 {
            desc.push_str(&format!("暴击: {}%\n", stats.crit_rate));
        }
        if stats.dodge_rate > 0 {
            desc.push_str(&format!("闪避: {}%\n", stats.dodge_rate));
        }

        if !self.effects.is_empty() {
            desc.push_str("特效:\n");
            for effect in &self.effects {
                desc.push_str(&format!("  - {}\n", effect.describe()));
            }
        }

        if let Some(ref suit_id) = self.suit_id {
            desc.push_str(&format!("套装: {}\n", suit_id));
        }

        desc
    }

    /// 计算战斗伤害（包含暴击、破甲等）
    pub fn calc_damage(&self, base_damage: u32, target_defense: u32) -> u32 {
        let mut damage = base_damage.saturating_add(self.final_stats().attack);

        // 破甲计算
        let armor_break = self.effects.iter()
            .filter_map(|e| {
                if let EquipEffect::ArmorBreak { percent } = e {
                    Some(target_defense * *percent as u32 / 100)
                } else { None }
            })
            .sum::<u32>();

        let actual_defense = target_defense.saturating_sub(armor_break);
        damage = damage.saturating_sub(actual_defense / 2);
        damage = damage.max(1);

        // 暴击计算
        let crit_chance = self.final_stats().crit_rate;
        if crit_chance > 0 {
            use rand::Rng;
            let roll = rand::thread_rng().gen_range(1..=100);
            if roll <= crit_chance {
                let crit_mult = (self.final_stats().crit_damage as f64 / 100.0).max(1.5);
                damage = (damage as f64 * crit_mult) as u32;
            }
        }

        // 额外伤害特效
        for effect in &self.effects {
            match effect {
                EquipEffect::BonusDamage { value } => {
                    damage = damage.saturating_add(*value);
                }
                EquipEffect::BonusDamagePercent { percent } => {
                    damage = (damage as f64 * (1.0 + *percent as f64 / 100.0)) as u32;
                }
                _ => {}
            }
        }

        damage
    }

    /// 攻击后处理（吸血等）
    pub fn after_attack(&self, damage_dealt: u32) -> u32 {
        let mut heal = 0u32;

        for effect in &self.effects {
            if let EquipEffect::LifeSteal { percent } = effect {
                heal = heal.saturating_add(damage_dealt * *percent as u32 / 100);
            }
        }

        heal
    }
}

// ============================================================================
// 套装配置
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuitConfig {
    /// 套装ID
    pub id: String,

    /// 套装名称
    pub name: String,

    /// 包含的装备模板ID列表
    pub items: Vec<String>,

    /// 套装效果（按激活件数）
    pub bonuses: HashMap<usize, SuitBonus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuitBonus {
    /// 激活件数描述（如 "2件套"）
    pub description: String,

    /// 属性加成
    pub stats: EquipStats,

    /// 特效列表
    #[serde(default)]
    pub effects: Vec<String>,
}

// ============================================================================
// 打造配方
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeRecipe {
    /// 配方ID
    pub id: String,

    /// 配方名称
    pub name: String,

    /// 产出装备模板ID
    pub template_id: String,

    /// 所需材料 (材料ID -> 数量)
    pub materials: HashMap<String, u32>,

    /// 成功率 (0-100)
    pub success_rate: u32,

    /// 等级要求
    #[serde(default)]
    pub level_req: u32,

    /// 打造消耗金币
    #[serde(default)]
    pub gold_cost: u32,

    /// 失败时是否返还部分材料
    #[serde(default)]
    pub partial_refund: bool,

    /// 配方描述
    #[serde(default)]
    pub description: String,
}

// ============================================================================
// 材料配置
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialConfig {
    /// 材料ID
    pub id: String,

    /// 材料名称
    pub name: String,

    /// 材料类型
    #[serde(default)]
    pub material_type: String,

    /// 稀有度（影响打造加成）
    #[serde(default)]
    pub rarity: u32,

    /// 单价
    #[serde(default)]
    pub price: u32,

    /// 描述
    #[serde(default)]
    pub description: String,
}

// ============================================================================
// 装备系统管理器
// ============================================================================

pub struct EquipSystem {
    /// 装备模板 (id -> template)
    templates: HashMap<String, EquipTemplate>,

    /// 套装配置 (id -> config)
    suits: HashMap<String, SuitConfig>,

    /// 打造配方 (id -> recipe)
    recipes: HashMap<String, ForgeRecipe>,

    /// 材料配置 (id -> material)
    materials: HashMap<String, MaterialConfig>,
}

impl EquipSystem {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            suits: HashMap::new(),
            recipes: HashMap::new(),
            materials: HashMap::new(),
        }
    }

    /// 加载装备模板
    pub fn load_templates(&mut self, json: &str) -> Result<()> {
        let loaded: Vec<EquipTemplate> = serde_json::from_str(json)
            .map_err(|e| MudError::RuntimeError(format!("解析装备模板失败: {}", e)))?;
        for template in loaded {
            self.templates.insert(template.id.clone(), template);
        }
        tracing::info!("已加载 {} 个装备模板", self.templates.len());
        Ok(())
    }

    /// 加载套装配置
    pub fn load_suits(&mut self, json: &str) -> Result<()> {
        let loaded: Vec<SuitConfig> = serde_json::from_str(json)
            .map_err(|e| MudError::RuntimeError(format!("解析套装配置失败: {}", e)))?;
        for suit in loaded {
            self.suits.insert(suit.id.clone(), suit);
        }
        tracing::info!("已加载 {} 个套装配置", self.suits.len());
        Ok(())
    }

    /// 加载打造配方
    pub fn load_recipes(&mut self, json: &str) -> Result<()> {
        let loaded: Vec<ForgeRecipe> = serde_json::from_str(json)
            .map_err(|e| MudError::RuntimeError(format!("解析打造配方失败: {}", e)))?;
        for recipe in loaded {
            self.recipes.insert(recipe.id.clone(), recipe);
        }
        tracing::info!("已加载 {} 个打造配方", self.recipes.len());
        Ok(())
    }

    /// 加载材料配置
    pub fn load_materials(&mut self, json: &str) -> Result<()> {
        let loaded: Vec<MaterialConfig> = serde_json::from_str(json)
            .map_err(|e| MudError::RuntimeError(format!("解析材料配置失败: {}", e)))?;
        for material in loaded {
            self.materials.insert(material.id.clone(), material);
        }
        tracing::info!("已加载 {} 种材料", self.materials.len());
        Ok(())
    }

    /// 获取装备模板
    pub fn get_template(&self, id: &str) -> Option<&EquipTemplate> {
        self.templates.get(id)
    }

    /// 根据打造配方创建装备
    pub fn forge(&self, recipe_id: &str, player_id: &str) -> Result<Equipment> {
        let recipe = self.recipes.get(recipe_id)
            .ok_or_else(|| MudError::ObjectNotFound(format!("配方不存在: {}", recipe_id)))?;

        let template = self.templates.get(&recipe.template_id)
            .ok_or_else(|| MudError::ObjectNotFound(format!("装备模板不存在: {}", recipe.template_id)))?;

        // 检查成功率
        use rand::Rng;
        let roll = rand::thread_rng().gen_range(1..=100);
        if roll > recipe.success_rate {
            return Err(MudError::RuntimeError("打造失败！".to_string()));
        }

        Ok(Equipment::from_template(template, player_id))
    }

    /// 列出所有模板
    pub fn list_templates(&self) -> Vec<&EquipTemplate> {
        self.templates.values().collect()
    }

    /// 列出所有配方
    pub fn list_recipes(&self) -> Vec<&ForgeRecipe> {
        self.recipes.values().collect()
    }
}

impl Default for EquipSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局装备系统实例
pub static EQUIP_SYSTEM: once_cell::sync::Lazy<tokio::sync::RwLock<EquipSystem>> =
    once_cell::sync::Lazy::new(|| tokio::sync::RwLock::new(EquipSystem::new()));

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_multiplier() {
        assert_eq!(EquipQuality::Common.multiplier(), 1.0);
        assert_eq!(EquipQuality::Legendary.multiplier(), 16.0);
        assert_eq!(EquipQuality::Mythic.multiplier(), 32.0);
    }

    #[test]
    fn test_realm_from_level() {
        assert_eq!(EquipRealm::from_level(5), EquipRealm::Mortal);
        assert_eq!(EquipRealm::from_level(15), EquipRealm::Foundation);
        assert_eq!(EquipRealm::from_level(50), EquipRealm::GoldenCore);
        assert_eq!(EquipRealm::from_level(80), EquipRealm::NascentSoul);
        assert_eq!(EquipRealm::from_level(200), EquipRealm::VoidRefinement);
        assert_eq!(EquipRealm::from_level(500), EquipRealm::Mahayana);
        assert_eq!(EquipRealm::from_level(1000), EquipRealm::Tribulation);
        assert_eq!(EquipRealm::from_level(2000), EquipRealm::GreatDao);
    }

    #[test]
    fn test_stats_scale() {
        let stats = EquipStats {
            attack: 100,
            defense: 50,
            ..Default::default()
        };

        let scaled = stats.scale(2.0);
        assert_eq!(scaled.attack, 200);
        assert_eq!(scaled.defense, 100);
    }

    #[test]
    fn test_reinforce() {
        let mut equip = Equipment {
            id: "test".to_string(),
            template_id: "sword".to_string(),
            name: "测试剑".to_string(),
            slot: EquipSlot::Weapon,
            quality: EquipQuality::Rare,
            realm: EquipRealm::Foundation,
            reinforce_level: 0,
            base_stats: EquipStats {
                attack: 100,
                ..Default::default()
            },
            effects: vec![],
            suit_id: None,
            description: String::new(),
            created_by: "test".to_string(),
            created_at: 0,
            bound: false,
            durability: None,
        };

        // +5 强化
        for _ in 0..5 {
            equip.reinforce().unwrap();
        }
        assert_eq!(equip.reinforce_level, 5);

        // 强化后属性 = 100 * (1 + 5 * 0.1) = 150
        assert_eq!(equip.final_stats().attack, 150);
    }

    #[test]
    fn test_damage_calc() {
        let equip = Equipment {
            id: "test".to_string(),
            template_id: "sword".to_string(),
            name: "测试剑".to_string(),
            slot: EquipSlot::Weapon,
            quality: EquipQuality::Rare,
            realm: EquipRealm::Foundation,
            reinforce_level: 0,
            base_stats: EquipStats {
                attack: 100,
                crit_rate: 20,
                crit_damage: 150,
                ..Default::default()
            },
            effects: vec![],
            suit_id: None,
            description: String::new(),
            created_by: "test".to_string(),
            created_at: 0,
            bound: false,
            durability: None,
        };

        // 测试伤害计算（基础100 + 装备100 - 防御一半）
        let damage = equip.calc_damage(100, 50);
        // base_damage(100) + attack(100) - defense(50)/2 = 175
        assert!(damage >= 150);
    }
}
