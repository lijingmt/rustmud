// gamenv/npc/npc.rs - NPC基类
// 对应 txpike9/gamenv/clone/npc/ 和 inherit/npc.pike

use crate::core::*;
use crate::gamenv::item::equipment::{Equipment, EquipSlot, EquipmentSlots};
use crate::gamenv::combat::{CombatStats, Combatant};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;

/// NPC类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NpcType {
    /// 普通NPC
    Normal,
    /// 商人
    Merchant,
    /// 怪物
    Monster,
    /// 任务NPC
    Quest,
    /// 训练师
    Trainer,
    /// 守卫
    Guard,
    /// Boss
    Boss,
}

/// NPC行为模式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NpcBehavior {
    /// 静止不动
    Static,
    /// 随机游荡
    Wander,
    /// 巡逻路线
    Patrol,
    /// 追击敌人
    Chase,
    /// 逃跑
    Flee,
}

/// NPC基础属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcBase {
    /// NPC唯一ID
    pub id: ObjectId,
    /// NPC模板ID
    pub template_id: String,
    /// NPC名称
    pub name: String,
    /// NPC中文名
    pub name_cn: String,
    /// NPC描述
    pub desc: String,
    /// NPC类型
    pub npc_type: NpcType,
    /// NPC等级
    pub level: u32,
    /// NPC行为模式
    pub behavior: NpcBehavior,
    /// 对话内容
    pub dialogue: Vec<String>,
    /// 商店物品 (如果是商人)
    pub shop_items: Vec<String>,
    /// 可学技能 (如果是训练师)
    pub teach_skills: Vec<String>,
    /// 任务ID (如果是任务NPC)
    pub quest_id: Option<String>,
    /// 刷新时间 (秒, 0表示不刷新)
    pub respawn_time: u32,
    /// 是否可攻击
    pub attackable: bool,
    /// 是否主动攻击
    pub aggressive: bool,
}

impl NpcBase {
    /// 创建新NPC
    pub fn new(name: String, name_cn: String, npc_type: NpcType) -> Self {
        Self {
            id: ObjectId::new(),
            template_id: name.clone(),
            name,
            name_cn,
            desc: String::new(),
            npc_type,
            level: 1,
            behavior: NpcBehavior::Static,
            dialogue: Vec::new(),
            shop_items: Vec::new(),
            teach_skills: Vec::new(),
            quest_id: None,
            respawn_time: 300, // 默认5分钟刷新
            attackable: false,
            aggressive: false,
        }
    }

    /// 设置描述
    pub fn with_desc(mut self, desc: String) -> Self {
        self.desc = desc;
        self
    }

    /// 设置等级
    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    /// 设置行为
    pub fn with_behavior(mut self, behavior: NpcBehavior) -> Self {
        self.behavior = behavior;
        self
    }

    /// 添加对话
    pub fn add_dialogue(&mut self, text: String) {
        self.dialogue.push(text);
    }

    /// 设置可攻击
    pub fn with_attackable(mut self, attackable: bool) -> Self {
        self.attackable = attackable;
        self
    }

    /// 设置主动攻击
    pub fn with_aggressive(mut self, aggressive: bool) -> Self {
        self.aggressive = aggressive;
        self
    }

    /// 渲染NPC描述
    pub fn render(&self) -> String {
        let mut result = format!("§c{}§r\n", self.name_cn);
        if !self.desc.is_empty() {
            result.push_str(&format!("{}\n", self.desc));
        }
        result
    }

    /// 获取随机对话
    pub fn get_random_dialogue(&self) -> Option<&String> {
        if self.dialogue.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..self.dialogue.len());
            self.dialogue.get(idx)
        }
    }
}

/// NPC完整对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    /// 基础属性
    pub base: NpcBase,
    /// 战斗属性
    pub combat: CombatStats,
    /// 装备
    pub equipment: EquipmentSlots,
    /// 当前位置
    pub current_room: Option<String>,
    /// 创建时间
    pub created_at: i64,
    /// 死亡时间 (0表示未死亡)
    pub died_at: Option<i64>,
}

impl Npc {
    /// 创建新NPC
    pub fn new(name: String, name_cn: String, npc_type: NpcType, level: u32) -> Self {
        let base = NpcBase::new(name, name_cn, npc_type)
            .with_level(level);

        let combat = CombatStats::for_level(level);

        Self {
            base,
            combat,
            equipment: EquipmentSlots::default(),
            current_room: None,
            created_at: chrono::Utc::now().timestamp(),
            died_at: None,
        }
    }

    /// 设置战斗属性
    pub fn with_combat(mut self, combat: CombatStats) -> Self {
        self.combat = combat;
        self
    }

    /// 设置装备
    pub fn with_equipment(mut self, equipment: EquipmentSlots) -> Self {
        self.equipment = equipment;
        self
    }

    /// 设置位置
    pub fn with_room(mut self, room_id: String) -> Self {
        self.current_room = Some(room_id);
        self
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.combat.hp > 0
    }

    /// 是否是怪物
    pub fn is_monster(&self) -> bool {
        matches!(self.base.npc_type, NpcType::Monster | NpcType::Boss)
    }

    /// 渲染NPC信息
    pub fn render_info(&self) -> String {
        let mut info = self.base.render();
        info.push_str(&format!("等级: {}\n", self.base.level));

        if self.base.attackable {
            info.push_str(&format!("生命: {}/{}\n", self.combat.hp, self.combat.max_hp));
        }

        if !self.base.dialogue.is_empty() {
            if let Some(dialogue) = self.base.get_random_dialogue() {
                info.push_str(&format!("\"{}\"\n", dialogue));
            }
        }

        info
    }

    /// 受到伤害
    pub fn take_damage(&mut self, damage: u32) -> bool {
        if damage >= self.combat.hp {
            self.combat.hp = 0;
            self.died_at = Some(chrono::Utc::now().timestamp());
            true // 死亡
        } else {
            self.combat.hp -= damage;
            false
        }
    }

    /// 治疗
    pub fn heal(&mut self, amount: u32) {
        self.combat.hp = (self.combat.hp + amount).min(self.combat.max_hp);
    }

    /// 复活
    pub fn respawn(&mut self) {
        self.combat.hp = self.combat.max_hp;
        self.combat.qi = self.combat.max_qi;
        self.combat.shen = self.combat.max_shen;
        self.died_at = None;
        self.created_at = chrono::Utc::now().timestamp();
    }
}

impl Combatant for Npc {
    fn get_name(&self) -> &str {
        &self.base.name_cn
    }

    fn get_level(&self) -> u32 {
        self.base.level
    }

    fn get_combat_stats(&self) -> &CombatStats {
        &self.combat
    }

    fn get_combat_stats_mut(&mut self) -> &mut CombatStats {
        &mut self.combat
    }

    fn is_alive(&self) -> bool {
        self.combat.hp > 0
    }
}

/// NPC管理器
pub struct NpcManager {
    /// NPC模板缓存
    templates: HashMap<String, Npc>,
    /// 活跃NPC
    active_npcs: HashMap<ObjectId, Npc>,
}

impl NpcManager {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            active_npcs: HashMap::new(),
        }
    }

    /// 创建NPC实例
    pub fn create_npc(&mut self, template_id: &str) -> Result<Npc> {
        if let Some(template) = self.templates.get(template_id) {
            let mut npc = template.clone();
            npc.base.id = ObjectId::new();
            npc.created_at = chrono::Utc::now().timestamp();
            npc.died_at = None;
            Ok(npc)
        } else {
            Err(MudError::NotFound(format!("NPC模板不存在: {}", template_id)))
        }
    }

    /// 注册NPC模板
    pub fn register_template(&mut self, npc: Npc) {
        self.templates.insert(npc.base.template_id.clone(), npc);
    }

    /// 生成NPC到房间
    pub fn spawn_npc(&mut self, template_id: &str, room_id: String) -> Result<ObjectId> {
        let mut npc = self.create_npc(template_id)?;
        npc.current_room = Some(room_id);
        let id = npc.base.id;
        self.active_npcs.insert(id, npc);
        Ok(id)
    }

    /// 获取NPC
    pub fn get_npc(&self, npc_id: ObjectId) -> Option<&Npc> {
        self.active_npcs.get(&npc_id)
    }

    /// 获取可变NPC
    pub fn get_npc_mut(&mut self, npc_id: ObjectId) -> Option<&mut Npc> {
        self.active_npcs.get_mut(&npc_id)
    }

    /// 移除NPC
    pub fn remove_npc(&mut self, npc_id: ObjectId) -> Option<Npc> {
        self.active_npcs.remove(&npc_id)
    }

    /// 获取房间内的所有NPC
    pub fn get_npcs_in_room(&self, room_id: &str) -> Vec<&Npc> {
        self.active_npcs.values()
            .filter(|npc| npc.current_room.as_deref() == Some(room_id))
            .collect()
    }
}

impl Default for NpcManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局NPC管理器
pub static NPCD: once_cell::sync::Lazy<std::sync::Mutex<NpcManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(NpcManager::default()));
