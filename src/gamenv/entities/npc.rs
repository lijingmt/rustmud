// gamenv/entities/npc.rs - NPC实体
// 对应 txpike9/wapmud2/single/npc.pike
// 使用trait组合: Character + Fight + Equip + Talkable

use crate::gamenv::traits::{Fight, Equip, EquipSlot, Talkable};

/// NPC实体
pub struct Npc {
    /// 角色基础数据
    pub character: super::Character,
    /// NPC模板ID (用于生成多个相同NPC)
    pub template_id: String,
    /// 当前房间ID
    pub current_room: String,
    /// 装备
    pub equipped: Vec<(String, crate::gamenv::item::Item)>,
    /// 对话数据
    pub dialogue_data: Vec<DialogueOption>,
}

/// 对话选项
#[derive(Debug, Clone)]
pub struct DialogueOption {
    pub topic: String,
    pub response: String,
}

impl Npc {
    /// 创建新NPC
    pub fn new(id: String, template_id: String, name: String, name_cn: String) -> Self {
        Self {
            character: super::Character::new(id, name, name_cn),
            template_id,
            current_room: String::new(),
            equipped: Vec::new(),
            dialogue_data: Vec::new(),
        }
    }

    /// 设置当前房间
    pub fn set_room(&mut self, room_id: String) {
        self.current_room = room_id;
    }
}

// 实现Fight trait
impl Fight for Npc {
    fn hp(&self) -> i32 {
        self.character.hp
    }

    fn hp_max(&self) -> i32 {
        self.character.hp_max
    }

    fn attack_power(&self) -> i32 {
        self.character.attack
    }

    fn defense(&self) -> i32 {
        self.character.defense
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

// 实现Equip trait (NPC可能有装备)
impl Equip for Npc {
    async fn equip_item(&mut self, _item: crate::gamenv::item::Item) -> Result<(), String> {
        Err("NPC无法装备物品".to_string())
    }

    async fn unequip_item(&mut self, _slot: EquipSlot) -> Result<crate::gamenv::item::Item, String> {
        Err("NPC无法卸下装备".to_string())
    }

    fn get_equipped(&self, slot: EquipSlot) -> Option<&crate::gamenv::item::Item> {
        match slot {
            EquipSlot::Weapon => self.equipped.iter()
                .find(|(s, _)| s == "weapon")
                .map(|(_, item)| item),
            _ => None,
        }
    }

    fn get_all_equipped(&self) -> Vec<(EquipSlot, &crate::gamenv::item::Item)> {
        self.equipped.iter()
            .filter_map(|(s, item)| {
                let slot = match s.as_str() {
                    "weapon" => Some(EquipSlot::Weapon),
                    _ => None,
                };
                slot.map(|sl| (sl, item))
            })
            .collect()
    }
}

// 实现Talkable trait
impl Talkable for Npc {
    async fn talk(&self, topic: &str) -> String {
        for dialogue in &self.dialogue_data {
            if dialogue.topic == topic {
                return dialogue.response.clone();
            }
        }
        format!("{}不知道你在说什么。", self.character.name_cn)
    }

    fn get_dialogue_options(&self) -> Vec<String> {
        self.dialogue_data.iter().map(|d| d.topic.clone()).collect()
    }
}
