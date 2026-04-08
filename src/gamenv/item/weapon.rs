// gamenv/item/weapon.rs - 武器系统
// 对应 txpike9/gamenv/clone/item/ 中的武器

use crate::core::*;
use crate::gamenv::item::equipment::{Equipment, EquipSlot};
use serde::{Deserialize, Serialize};

/// 武器类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WeaponType {
    /// 剑
    Sword,
    /// 刀
    Blade,
    /// 枪
    Spear,
    /// 棍
    Staff,
    /// 棒
    Club,
    /// 拳套
    Fist,
    /// 扇子
    Fan,
    /// 笔
    Brush,
    /// 琴
    Zither,
    /// 暗器
    Hidden,
}

impl WeaponType {
    /// 获取武器类型名称
    pub fn name(&self) -> &str {
        match self {
            WeaponType::Sword => "剑",
            WeaponType::Blade => "刀",
            WeaponType::Spear => "枪",
            WeaponType::Staff => "棍",
            WeaponType::Club => "棒",
            WeaponType::Fist => "拳套",
            WeaponType::Fan => "扇子",
            WeaponType::Brush => "笔",
            WeaponType::Zither => "琴",
            WeaponType::Hidden => "暗器",
        }
    }

    /// 从字符串解析武器类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "剑" | "sword" => Some(WeaponType::Sword),
            "刀" | "blade" => Some(WeaponType::Blade),
            "枪" | "spear" => Some(WeaponType::Spear),
            "棍" | "staff" => Some(WeaponType::Staff),
            "棒" | "club" => Some(WeaponType::Club),
            "拳套" | "fist" => Some(WeaponType::Fist),
            "扇子" | "fan" => Some(WeaponType::Fan),
            "笔" | "brush" => Some(WeaponType::Brush),
            "琴" | "zither" => Some(WeaponType::Zither),
            "暗器" | "hidden" => Some(WeaponType::Hidden),
            _ => None,
        }
    }
}

/// 武器 (对应 /gamenv/clone/item/ 中的武器)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    /// 基础装备数据
    #[serde(flatten)]
    pub equipment: Equipment,
    /// 武器类型
    pub weapon_type: WeaponType,
    /// 武器攻速 (每秒攻击次数)
    pub attack_speed: f32,
    /// 武器射程
    pub range: u32,
}

impl Weapon {
    /// 创建新武器
    pub fn new(name: String, name_cn: String, weapon_type: WeaponType) -> Self {
        let equipment = Equipment::new(name.clone(), name_cn.clone(), EquipSlot::Weapon);
        Self {
            equipment,
            weapon_type,
            attack_speed: 1.0,
            range: 1,
        }
    }

    /// 设置攻速
    pub fn with_attack_speed(mut self, speed: f32) -> Self {
        self.attack_speed = speed;
        self
    }

    /// 设置射程
    pub fn with_range(mut self, range: u32) -> Self {
        self.range = range;
        self
    }

    /// 计算每秒伤害 (DPS)
    pub fn dps(&self) -> f32 {
        self.equipment.stats.attack as f32 * self.attack_speed
    }

    /// 渲染武器信息
    pub fn render_info(&self) -> String {
        let mut info = self.equipment.render_info();
        info.push_str(&format!("武器类型: {}\n", self.weapon_type.name()));
        info.push_str(&format!("攻击速度: {}/秒\n", self.attack_speed));
        info.push_str(&format!("射程: {}\n", self.range));
        info.push_str(&format!("每秒伤害: {:.1}\n", self.dps()));
        info
    }
}

/// 预设武器列表
pub fn create_preset_weapons() -> Vec<Weapon> {
    vec![
        // 新手武器
        Weapon::new(
            "weapon_iron_sword".to_string(),
            "铁剑".to_string(),
            WeaponType::Sword,
        )
        .with_attack_speed(1.2)
        .with_range(1),

        Weapon::new(
            "weapon_wood_staff".to_string(),
            "木棍".to_string(),
            WeaponType::Staff,
        )
        .with_attack_speed(1.0)
        .with_range(2),

        // 进阶武器
        Weapon::new(
            "weapon_steel_blade".to_string(),
            "钢刀".to_string(),
            WeaponType::Blade,
        )
        .with_attack_speed(1.1)
        .with_range(1),

        Weapon::new(
            "weapon_jade_sword".to_string(),
            "玉剑".to_string(),
            WeaponType::Sword,
        )
        .with_attack_speed(1.5)
        .with_range(1),

        // 高级武器
        Weapon::new(
            "weapon_dragon_spear".to_string(),
            "龙枪".to_string(),
            WeaponType::Spear,
        )
        .with_attack_speed(0.9)
        .with_range(3),

        Weapon::new(
            "weapon_heaven_sword".to_string(),
            "倚天剑".to_string(),
            WeaponType::Sword,
        )
        .with_attack_speed(1.8)
        .with_range(1),
    ]
}
