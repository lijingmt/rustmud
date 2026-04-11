// gamenv/entities/player.rs - 玩家实体
// 对应 txpike9/wapmud2/single/master.pike
// 使用trait组合: Character + Fight + Inventory + Skills + Equip

use crate::gamenv::item::Item;
use crate::gamenv::traits::{Fight, Inventory, Skills, Equip, EquipSlot};
use std::collections::HashMap;

/// 玩家实体 - 组合多个trait
pub struct Player {
    /// 角色基础数据
    pub character: super::Character,
    /// 当前房间ID
    pub current_room: String,
    /// 背包物品
    pub items: Vec<Item>,
    /// 最大负重
    pub max_encumbrance: i32,
    /// 技能列表 (技能ID -> 等级)
    pub skills_map: HashMap<String, u32>,
    /// 装备
    pub equipped: HashMap<String, Item>,
}

impl Player {
    /// 创建新玩家
    pub fn new(id: String, name: String, name_cn: String) -> Self {
        Self {
            character: super::Character::new(id, name, name_cn),
            current_room: "xinshoucun/changetang".to_string(),
            items: Vec::new(),
            max_encumbrance: 1000,
            skills_map: HashMap::new(),
            equipped: HashMap::new(),
        }
    }

    /// 获取ID
    pub fn id(&self) -> &str {
        &self.character.id
    }

    /// 获取姓名
    pub fn name(&self) -> &str {
        &self.character.name
    }

    /// 获取中文名
    pub fn name_cn(&self) -> &str {
        &self.character.name_cn
    }
}

// 实现Fight trait
impl Fight for Player {
    fn hp(&self) -> i32 {
        self.character.hp
    }

    fn hp_max(&self) -> i32 {
        self.character.hp_max
    }

    fn attack_power(&self) -> i32 {
        self.character.attack + self.get_equip_bonus("attack")
    }

    fn defense(&self) -> i32 {
        self.character.defense + self.get_equip_bonus("defense")
    }

    fn level(&self) -> u32 {
        self.character.level
    }

    async fn take_damage(&mut self, damage: i32) -> bool {
        self.character.take_damage(damage)
    }

    async fn heal(&mut self, amount: i32) {
        self.character.heal(amount);
    }
}

// 实现Inventory trait
impl Inventory for Player {
    fn items(&self) -> &[Item] {
        &self.items
    }

    async fn add_item(&mut self, item: Item) -> Result<(), String> {
        // TODO: 检查负重
        self.items.push(item);
        Ok(())
    }

    async fn remove_item(&mut self, item_id: &str) -> Result<Item, String> {
        if let Some(pos) = self.items.iter().position(|i| i.id.to_string() == item_id) {
            Ok(self.items.remove(pos))
        } else {
            Err("物品不存在".to_string())
        }
    }

    fn get_item(&self, item_id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id.to_string() == item_id)
    }

    fn max_encumbrance(&self) -> i32 {
        self.max_encumbrance
    }
}

// 实现Skills trait
impl Skills for Player {
    fn skills(&self) -> &HashMap<String, u32> {
        &self.skills_map
    }

    async fn learn_skill(&mut self, skill_id: &str) -> Result<(), String> {
        if self.skills_map.contains_key(skill_id) {
            return Err("你已经学会了这个技能".to_string());
        }
        self.skills_map.insert(skill_id.to_string(), 1);
        Ok(())
    }

    async fn use_skill(&mut self, skill_id: &str) -> Result<String, String> {
        if !self.skills_map.contains_key(skill_id) {
            return Err("你还没有学会这个技能".to_string());
        }
        Ok(format!("你使用了 {}！", skill_id))
    }
}

// 实现Equip trait
impl Equip for Player {
    async fn equip_item(&mut self, item: Item) -> Result<(), String> {
        let slot_name = match item.item_type {
            crate::gamenv::item::ItemType::Weapon => "weapon",
            crate::gamenv::item::ItemType::Armor => "armor",
            _ => return Err("该物品无法装备".to_string()),
        };
        self.equipped.insert(slot_name.to_string(), item);
        Ok(())
    }

    async fn unequip_item(&mut self, slot: EquipSlot) -> Result<Item, String> {
        let slot_name = match slot {
            EquipSlot::Weapon => "weapon",
            EquipSlot::Armor => "armor",
            _ => return Err("该槽位不存在".to_string()),
        };
        if let Some(item) = self.equipped.remove(slot_name) {
            Ok(item)
        } else {
            Err("该槽位没有装备".to_string())
        }
    }

    fn get_equipped(&self, slot: EquipSlot) -> Option<&Item> {
        let slot_name = match slot {
            EquipSlot::Weapon => "weapon",
            EquipSlot::Armor => "armor",
            _ => return None,
        };
        self.equipped.get(slot_name)
    }

    fn get_all_equipped(&self) -> Vec<(EquipSlot, &Item)> {
        self.equipped.iter()
            .filter_map(|(k, v)| {
                let slot = match k.as_str() {
                    "weapon" => Some(EquipSlot::Weapon),
                    "armor" => Some(EquipSlot::Armor),
                    _ => None,
                };
                slot.map(|s| (s, v))
            })
            .collect()
    }
}

// Player专属方法
impl Player {
    /// 获取装备加成属性
    pub fn get_equip_bonus(&self, stat: &str) -> i32 {
        self.get_all_equipped()
            .iter()
            .map(|(_, item)| {
                // 从extra_data读取装备加成
                item.extra_data[stat]
                    .as_i64()
                    .unwrap_or(0) as i32
            })
            .sum()
    }
}
