// gamenv/item/medicine.rs - 药品系统
// 对应 txpike9/gamenv/clone/item/ 中的药品

use crate::core::*;
use crate::gamenv::item::item::{Item, ItemType, ItemQuality};
use serde::{Deserialize, Serialize};

/// 药品类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MedicineType {
    /// 恢复生命
    Hp,
    /// 恢复内力
    Qi,
    /// 恢复精神
    Shen,
    /// 恢复全部
    All,
    /// 解毒
    Detox,
    /// 治疗伤势
    Heal,
}

impl MedicineType {
    /// 获取药品类型名称
    pub fn name(&self) -> &str {
        match self {
            MedicineType::Hp => "生命药水",
            MedicineType::Qi => "内力药水",
            MedicineType::Shen => "精神药水",
            MedicineType::All => "全能药水",
            MedicineType::Detox => "解毒丹",
            MedicineType::Heal => "疗伤药",
        }
    }
}

/// 药品效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicineEffect {
    /// 恢复生命值
    pub hp_restore: u32,
    /// 恢复内力值
    pub qi_restore: u32,
    /// 恢复精神值
    pub shen_restore: u32,
    /// 是否百分比恢复
    pub is_percent: bool,
}

impl Default for MedicineEffect {
    fn default() -> Self {
        Self {
            hp_restore: 0,
            qi_restore: 0,
            shen_restore: 0,
            is_percent: false,
        }
    }
}

/// 药品 (对应 /gamenv/clone/item/ 中的药品)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medicine {
    /// 基础物品数据
    #[serde(flatten)]
    pub item: Item,
    /// 药品类型
    pub medicine_type: MedicineType,
    /// 药品效果
    pub effect: MedicineEffect,
    /// 使用冷却时间 (秒)
    pub cooldown: u32,
}

impl Medicine {
    /// 创建新药品
    pub fn new(name: String, name_cn: String, medicine_type: MedicineType) -> Self {
        let mut item = Item::new(name.clone(), name_cn.clone(), ItemType::Medicine);
        item.max_stack = 99; // 药品可堆叠

        Self {
            item,
            medicine_type,
            effect: MedicineEffect::default(),
            cooldown: 10,
        }
    }

    /// 设置效果
    pub fn with_effect(mut self, effect: MedicineEffect) -> Self {
        self.effect = effect;
        self
    }

    /// 设置冷却
    pub fn with_cooldown(mut self, cooldown: u32) -> Self {
        self.cooldown = cooldown;
        self
    }

    /// 使用药品
    pub fn use_medicine(&self, current_hp: u32, max_hp: u32,
                        current_qi: u32, max_qi: u32,
                        current_shen: u32, max_shen: u32) -> (u32, u32, u32) {
        let (new_hp, new_qi, new_shen) = if self.effect.is_percent {
            // 百分比恢复
            let hp_add = (max_hp as f32 * self.effect.hp_restore as f32 / 100.0) as u32;
            let qi_add = (max_qi as f32 * self.effect.qi_restore as f32 / 100.0) as u32;
            let shen_add = (max_shen as f32 * self.effect.shen_restore as f32 / 100.0) as u32;
            (
                (current_hp + hp_add).min(max_hp),
                (current_qi + qi_add).min(max_qi),
                (current_shen + shen_add).min(max_shen),
            )
        } else {
            // 固定值恢复
            (
                (current_hp + self.effect.hp_restore).min(max_hp),
                (current_qi + self.effect.qi_restore).min(max_qi),
                (current_shen + self.effect.shen_restore).min(max_shen),
            )
        };

        (new_hp, new_qi, new_shen)
    }

    /// 渲染药品信息
    pub fn render_info(&self) -> String {
        let mut info = format!("{}【{}】{}§r\n",
            self.item.quality.color_code(),
            self.medicine_type.name(),
            self.item.name_cn
        );

        if !self.item.desc.is_empty() {
            info.push_str(&format!("{}\n", self.item.desc));
        }

        if self.effect.hp_restore > 0 {
            if self.effect.is_percent {
                info.push_str(&format!("恢复生命: +{}%\n", self.effect.hp_restore));
            } else {
                info.push_str(&format!("恢复生命: +{}\n", self.effect.hp_restore));
            }
        }
        if self.effect.qi_restore > 0 {
            if self.effect.is_percent {
                info.push_str(&format!("恢复内力: +{}%\n", self.effect.qi_restore));
            } else {
                info.push_str(&format!("恢复内力: +{}\n", self.effect.qi_restore));
            }
        }
        if self.effect.shen_restore > 0 {
            if self.effect.is_percent {
                info.push_str(&format!("恢复精神: +{}%\n", self.effect.shen_restore));
            } else {
                info.push_str(&format!("恢复精神: +{}\n", self.effect.shen_restore));
            }
        }

        info.push_str(&format!("冷却时间: {}秒\n", self.cooldown));

        if self.item.quantity > 1 {
            info.push_str(&format!("数量: {}/{}\n", self.item.quantity, self.item.max_stack));
        }

        info
    }
}

/// 预设药品列表
pub fn create_preset_medicines() -> Vec<Medicine> {
    vec![
        // 基础药品
        Medicine::new(
            "med_hp_small".to_string(),
            "小生命药水".to_string(),
            MedicineType::Hp,
        )
        .with_effect(MedicineEffect {
            hp_restore: 50,
            qi_restore: 0,
            shen_restore: 0,
            is_percent: false,
        })
        .with_cooldown(10),

        Medicine::new(
            "med_hp_medium".to_string(),
            "中生命药水".to_string(),
            MedicineType::Hp,
        )
        .with_effect(MedicineEffect {
            hp_restore: 200,
            qi_restore: 0,
            shen_restore: 0,
            is_percent: false,
        })
        .with_cooldown(15),

        Medicine::new(
            "med_hp_large".to_string(),
            "大生命药水".to_string(),
            MedicineType::Hp,
        )
        .with_effect(MedicineEffect {
            hp_restore: 500,
            qi_restore: 0,
            shen_restore: 0,
            is_percent: false,
        })
        .with_cooldown(20),

        // 内力药水
        Medicine::new(
            "med_qi_small".to_string(),
            "小内力药水".to_string(),
            MedicineType::Qi,
        )
        .with_effect(MedicineEffect {
            hp_restore: 0,
            qi_restore: 50,
            shen_restore: 0,
            is_percent: false,
        })
        .with_cooldown(10),

        Medicine::new(
            "med_qi_large".to_string(),
            "大内力药水".to_string(),
            MedicineType::Qi,
        )
        .with_effect(MedicineEffect {
            hp_restore: 0,
            qi_restore: 300,
            shen_restore: 0,
            is_percent: false,
        })
        .with_cooldown(20),

        // 全能药水
        Medicine::new(
            "med_all_percent".to_string(),
            "全能药水".to_string(),
            MedicineType::All,
        )
        .with_effect(MedicineEffect {
            hp_restore: 30,
            qi_restore: 30,
            shen_restore: 30,
            is_percent: true,
        })
        .with_cooldown(60),

        // 高级药品
        Medicine::new(
            "med_elixir".to_string(),
            "九转金丹".to_string(),
            MedicineType::All,
        )
        .with_effect(MedicineEffect {
            hp_restore: 100,
            qi_restore: 100,
            shen_restore: 100,
            is_percent: true,
        })
        .with_cooldown(300),
    ]
}
