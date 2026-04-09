// gamenv/single/daemons/duanzaod.rs - 锻造系统守护进程
// 对应 txpike9/gamenv/single/daemons/duanzaod.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 锻造结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ForgeResult {
    /// 成功
    Success { item_id: String, name: String },
    /// 失败
    Failed,
    /// 丢失材料
    LostMaterials,
}

/// 装备部位
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    /// 武器
    Weapon,
    /// 盔甲
    Armor,
    /// 头盔
    Helmet,
    /// 项链
    Necklace,
    /// 戒指
    Ring,
    /// 护腕
    Bracelet,
    /// 鞋子
    Boots,
}

/// 装备境界
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EquipmentRealm {
    /// 凡品
    Mortal,
    /// 灵器
    Spirit,
    /// 仙器
    Immortal,
    /// 神器
    Divine,
}

/// 装备品质
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EquipmentQuality {
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
    /// 神话
    Mythical,
}

/// 锻造配方
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForgeRecipe {
    /// 配方ID
    pub id: String,
    /// 名称
    pub name: String,
    /// 装备部位
    pub slot: EquipmentSlot,
    /// 等级要求
    pub level_req: i32,
    /// 所需材料 (material_id -> count)
    pub materials: HashMap<String, i32>,
    /// 成功率
    pub success_rate: i32,
    /// 基础属性
    pub base_stats: EquipmentStats,
    /// 境界
    pub realm: EquipmentRealm,
}

/// 装备属性
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentStats {
    /// 攻击力
    pub attack: i32,
    /// 防御力
    pub defense: i32,
    /// HP加成
    pub hp_bonus: i32,
    /// MP加成
    pub mp_bonus: i32,
    /// 暴击率
    pub crit_rate: i32,
    /// 暴击伤害
    pub crit_damage: i32,
}

/// 锻造材料
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForgeMaterial {
    /// 材料ID
    pub id: String,
    /// 名称
    pub name: String,
    /// 材料类型
    pub material_type: String,
    /// 稀有度
    pub rarity: i32,
    /// 描述
    pub description: String,
}

/// 锻造守护进程
pub struct DuanzaoDaemon {
    /// 所有配方
    recipes: HashMap<String, ForgeRecipe>,
    /// 所有材料
    materials: HashMap<String, ForgeMaterial>,
}

impl DuanzaoDaemon {
    /// 创建新的锻造守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            recipes: HashMap::new(),
            materials: HashMap::new(),
        };

        daemon.init_default_recipes();
        daemon.init_default_materials();
        daemon
    }

    /// 初始化默认配方
    fn init_default_recipes(&mut self) {
        // 铁剑
        let iron_sword = ForgeRecipe {
            id: "iron_sword".to_string(),
            name: "铁剑".to_string(),
            slot: EquipmentSlot::Weapon,
            level_req: 5,
            materials: {
                let mut m = HashMap::new();
                m.insert("iron".to_string(), 5);
                m.insert("wood".to_string(), 2);
                m
            },
            success_rate: 90,
            base_stats: EquipmentStats {
                attack: 15,
                defense: 0,
                hp_bonus: 0,
                mp_bonus: 0,
                crit_rate: 0,
                crit_damage: 0,
            },
            realm: EquipmentRealm::Mortal,
        };

        // 钢剑
        let steel_sword = ForgeRecipe {
            id: "steel_sword".to_string(),
            name: "钢剑".to_string(),
            slot: EquipmentSlot::Weapon,
            level_req: 10,
            materials: {
                let mut m = HashMap::new();
                m.insert("steel".to_string(), 5);
                m.insert("iron".to_string(), 10);
                m
            },
            success_rate: 75,
            base_stats: EquipmentStats {
                attack: 30,
                defense: 0,
                hp_bonus: 0,
                mp_bonus: 0,
                crit_rate: 5,
                crit_damage: 10,
            },
            realm: EquipmentRealm::Mortal,
        };

        // 铁甲
        let iron_armor = ForgeRecipe {
            id: "iron_armor".to_string(),
            name: "铁甲".to_string(),
            slot: EquipmentSlot::Armor,
            level_req: 8,
            materials: {
                let mut m = HashMap::new();
                m.insert("iron".to_string(), 15);
                m.insert("leather".to_string(), 5);
                m
            },
            success_rate: 85,
            base_stats: EquipmentStats {
                attack: 0,
                defense: 20,
                hp_bonus: 50,
                mp_bonus: 0,
                crit_rate: 0,
                crit_damage: 0,
            },
            realm: EquipmentRealm::Mortal,
        };

        self.recipes.insert(iron_sword.id.clone(), iron_sword);
        self.recipes.insert(steel_sword.id.clone(), steel_sword);
        self.recipes.insert(iron_armor.id.clone(), iron_armor);
    }

    /// 初始化默认材料
    fn init_default_materials(&mut self) {
        let iron = ForgeMaterial {
            id: "iron".to_string(),
            name: "铁矿石".to_string(),
            material_type: "metal".to_string(),
            rarity: 1,
            description: "常见的铁矿石，可用于锻造基础装备。".to_string(),
        };

        let steel = ForgeMaterial {
            id: "steel".to_string(),
            name: "钢材".to_string(),
            material_type: "metal".to_string(),
            rarity: 3,
            description: "精炼的钢材，可用于锻造高级装备。".to_string(),
        };

        let wood = ForgeMaterial {
            id: "wood".to_string(),
            name: "木材".to_string(),
            material_type: "wood".to_string(),
            rarity: 1,
            description: "普通的木材，可用于制作武器手柄。".to_string(),
        };

        let leather = ForgeMaterial {
            id: "leather".to_string(),
            name: "皮革".to_string(),
            material_type: "leather".to_string(),
            rarity: 1,
            description: "经过处理的动物皮革，可用于制作护甲。".to_string(),
        };

        self.materials.insert(iron.id.clone(), iron);
        self.materials.insert(steel.id.clone(), steel);
        self.materials.insert(wood.id.clone(), wood);
        self.materials.insert(leather.id.clone(), leather);
    }

    /// 获取配方
    pub fn get_recipe(&self, recipe_id: &str) -> Option<&ForgeRecipe> {
        self.recipes.get(recipe_id)
    }

    /// 获取所有配方
    pub fn get_all_recipes(&self) -> Vec<&ForgeRecipe> {
        self.recipes.values().collect()
    }

    /// 获取可学配方
    pub fn get_available_recipes(&self, player_level: i32) -> Vec<&ForgeRecipe> {
        self.recipes.values()
            .filter(|r| player_level >= r.level_req)
            .collect()
    }

    /// 获取材料
    pub fn get_material(&self, material_id: &str) -> Option<&ForgeMaterial> {
        self.materials.get(material_id)
    }

    /// 锻造装备
    pub fn forge(
        &self,
        recipe_id: &str,
        player_level: i32,
        player_materials: &HashMap<String, i32>,
    ) -> Result<ForgeResult> {
        let recipe = self.get_recipe(recipe_id)
            .ok_or_else(|| MudError::NotFound("配方不存在".to_string()))?;

        // 检查等级
        if player_level < recipe.level_req {
            return Err(MudError::RuntimeError("等级不足".to_string()));
        }

        // 检查材料
        for (mat_id, needed) in &recipe.materials {
            let have = player_materials.get(mat_id).copied().unwrap_or(0);
            if have < *needed {
                if let Some(mat) = self.get_material(mat_id) {
                    return Err(MudError::RuntimeError(format!("缺少材料: {} (需要{}个)", mat.name, needed)));
                }
            }
        }

        // 计算成功率
        let success = rand::random::<i32>() < recipe.success_rate;

        if success {
            Ok(ForgeResult::Success {
                item_id: recipe.id.clone(),
                name: recipe.name.clone(),
            })
        } else {
            Ok(ForgeResult::Failed)
        }
    }

    /// 分解装备
    pub fn decompose(
        &self,
        item_id: &str,
        player_level: i32,
    ) -> Result<Vec<(String, i32)>> {
        // 简化实现：分解随机材料
        let mut materials = Vec::new();

        // 根据装备等级生成材料
        let material_count = (player_level / 3) + 1;

        // 随机材料
        let possible_mats = ["iron", "steel", "wood", "leather"];
        for _ in 0..material_count {
            let idx = rand::random::<usize>() % possible_mats.len();
            materials.push((possible_mats[idx].to_string(), 1));
        }

        Ok(materials)
    }

    /// 格式化配方列表
    pub fn format_recipe_list(&self, recipes: &[&ForgeRecipe]) -> String {
        let mut output = String::from("§H=== 锻造配方 ===§N\n");

        if recipes.is_empty() {
            output.push_str("没有可用的配方。\n");
        } else {
            for recipe in recipes {
                let slot_name = match recipe.slot {
                    EquipmentSlot::Weapon => "武器",
                    EquipmentSlot::Armor => "护甲",
                    EquipmentSlot::Helmet => "头盔",
                    EquipmentSlot::Necklace => "项链",
                    EquipmentSlot::Ring => "戒指",
                    EquipmentSlot::Bracelet => "护腕",
                    EquipmentSlot::Boots => "鞋子",
                };

                let mut material_list = Vec::new();
                for (mat_id, count) in &recipe.materials {
                    if let Some(mat) = self.get_material(mat_id) {
                        material_list.push(format!("{} x{}", mat.name, count));
                    }
                }

                output.push_str(&format!(
                    "§Y[{}]§N {} - Lv.{}\n  部位: {}\n  成功率: {}%\n  材料: {}\n",
                    recipe.name,
                    match recipe.realm {
                        EquipmentRealm::Mortal => "凡品",
                        EquipmentRealm::Spirit => "§C灵器§N",
                        EquipmentRealm::Immortal => "§B仙器§N",
                        EquipmentRealm::Divine => "§Y神器§N",
                    },
                    recipe.level_req,
                    slot_name,
                    recipe.success_rate,
                    material_list.join(", ")
                ));
            }
        }

        output
    }

    /// 格式化属性
    pub fn format_stats(&self, stats: &EquipmentStats) -> String {
        let mut parts = Vec::new();

        if stats.attack > 0 {
            parts.push(format!("攻击+{}", stats.attack));
        }
        if stats.defense > 0 {
            parts.push(format!("防御+{}", stats.defense));
        }
        if stats.hp_bonus > 0 {
            parts.push(format!("HP+{}", stats.hp_bonus));
        }
        if stats.mp_bonus > 0 {
            parts.push(format!("MP+{}", stats.mp_bonus));
        }
        if stats.crit_rate > 0 {
            parts.push(format!("暴击+{}%", stats.crit_rate));
        }

        if parts.is_empty() {
            "无特殊属性".to_string()
        } else {
            parts.join(" ")
        }
    }
}

impl Default for DuanzaoDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局锻造守护进程
pub static DUANZAOD: std::sync::OnceLock<RwLock<DuanzaoDaemon>> = std::sync::OnceLock::new();

/// 获取锻造守护进程
pub fn get_duanzaod() -> &'static RwLock<DuanzaoDaemon> {
    DUANZAOD.get_or_init(|| RwLock::new(DuanzaoDaemon::default()))
}
