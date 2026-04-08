// gamenv/item/armor.rs - 防具系统
// 对应 txpike9/gamenv/clone/item/ 中的防具

use crate::core::*;
use crate::gamenv::item::equipment::{Equipment, EquipSlot, EquipRealm};
use serde::{Deserialize, Serialize};

/// 防具类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArmorType {
    /// 布衣
    Cloth,
    /// 皮甲
    Leather,
    /// 铁甲
    Iron,
    /// 精钢甲
    Steel,
    /// 丝绸袍
    Silk,
    /// 道袍
    Taoist,
    /// 战甲
    Battle,
}

impl ArmorType {
    /// 获取防具类型名称
    pub fn name(&self) -> &str {
        match self {
            ArmorType::Cloth => "布衣",
            ArmorType::Leather => "皮甲",
            ArmorType::Iron => "铁甲",
            ArmorType::Steel => "精钢甲",
            ArmorType::Silk => "丝绸袍",
            ArmorType::Taoist => "道袍",
            ArmorType::Battle => "战甲",
        }
    }
}

/// 防具 (对应 /gamenv/clone/item/ 中的防具)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Armor {
    /// 基础装备数据
    #[serde(flatten)]
    pub equipment: Equipment,
    /// 防具类型
    pub armor_type: ArmorType,
}

impl Armor {
    /// 创建新防具
    pub fn new(name: String, name_cn: String, slot: EquipSlot, armor_type: ArmorType) -> Self {
        let mut equipment = Equipment::new(name, name_cn, slot);
        // 防具的基础防御加成
        equipment.stats.defense = 10;
        Self {
            equipment,
            armor_type,
        }
    }

    /// 渲染防具信息
    pub fn render_info(&self) -> String {
        let mut info = self.equipment.render_info();
        info.push_str(&format!("防具类型: {}\n", self.armor_type.name()));
        info
    }
}

/// 预设防具列表
pub fn create_preset_armors() -> Vec<Armor> {
    vec![
        // 新手防具
        Armor::new(
            "armor_cloth_shirt".to_string(),
            "布衣".to_string(),
            EquipSlot::Armor,
            ArmorType::Cloth,
        ),
        Armor::new(
            "helmet_cloth_hat".to_string(),
            "布帽".to_string(),
            EquipSlot::Helmet,
            ArmorType::Cloth,
        ),
        Armor::new(
            "boots_cloth_shoes".to_string(),
            "布鞋".to_string(),
            EquipSlot::Boots,
            ArmorType::Cloth,
        ),

        // 进阶防具
        Armor::new(
            "armor_leather_vest".to_string(),
            "皮甲".to_string(),
            EquipSlot::Armor,
            ArmorType::Leather,
        ),
        Armor::new(
            "helmet_leather_cap".to_string(),
            "皮帽".to_string(),
            EquipSlot::Helmet,
            ArmorType::Leather,
        ),

        // 高级防具
        Armor::new(
            "armor_iron_mail".to_string(),
            "铁甲".to_string(),
            EquipSlot::Armor,
            ArmorType::Iron,
        ),
        Armor::new(
            "armor_steel_plate".to_string(),
            "精钢甲".to_string(),
            EquipSlot::Armor,
            ArmorType::Steel,
        ),
    ]
}

/// 境界装备颜色映射
pub fn get_realm_color(realm: EquipRealm) -> String {
    realm.color_code().to_string()
}

/// 根据境界获取装备颜色CSS类
pub fn get_realm_css_class(realm: EquipRealm) -> &'static str {
    match realm {
        EquipRealm::Mortal => "color-white",
        EquipRealm::Foundation => "color-green",
        EquipRealm::GoldenCore => "color-blue",
        EquipRealm::NascentSoul => "color-purple",
        EquipRealm::SoulFormation => "color-orange",
        EquipRealm::VoidRefinement => "color-red",
        EquipRealm::Unity => "color-darkred",
        EquipRealm::Mahayana => "color-gold",
        EquipRealm::Tribulation => "color-rainbow",
        EquipRealm::GreatDao => "color-colorful",
    }
}
