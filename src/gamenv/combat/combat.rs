// gamenv/combat/combat.rs - 战斗系统核心
// 对应 txpike9 中的战斗逻辑

use crate::core::*;
use crate::gamenv::item::equipment::{EquipStats, EquipRealm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;

/// 战斗属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    /// 当前生命值
    pub hp: u32,
    /// 最大生命值
    pub max_hp: u32,
    /// 当前内力值
    pub qi: u32,
    /// 最大内力值
    pub max_qi: u32,
    /// 当前精神值
    pub shen: u32,
    /// 最大精神值
    pub max_shen: u32,
    /// 攻击力
    pub attack: u32,
    /// 防御力
    pub defense: u32,
    /// 命中率
    pub hit_rate: u32,
    /// 闪避率
    pub dodge_rate: u32,
    /// 暴击率
    pub crit_rate: u32,
    /// 暴击伤害倍率 (100 = 1x, 150 = 1.5x)
    pub crit_damage: u32,
    /// 攻击速度
    pub attack_speed: u32,
    /// 移动速度
    pub move_speed: u32,
}

impl Default for CombatStats {
    fn default() -> Self {
        Self {
            hp: 100,
            max_hp: 100,
            qi: 50,
            max_qi: 50,
            shen: 50,
            max_shen: 50,
            attack: 10,
            defense: 5,
            hit_rate: 90,
            dodge_rate: 10,
            crit_rate: 5,
            crit_damage: 150,
            attack_speed: 100,
            move_speed: 100,
        }
    }
}

impl CombatStats {
    /// 根据等级计算基础属性
    pub fn for_level(level: u32) -> Self {
        let hp_base = 100 + level * 20;
        let attack_base = 10 + level * 3;
        let defense_base = 5 + level * 2;

        Self {
            hp: hp_base,
            max_hp: hp_base,
            qi: 50 + level * 10,
            max_qi: 50 + level * 10,
            shen: 50 + level * 10,
            max_shen: 50 + level * 10,
            attack: attack_base,
            defense: defense_base,
            hit_rate: 85 + level / 5,
            dodge_rate: 5 + level / 10,
            crit_rate: 5 + level / 20,
            crit_damage: 150,
            attack_speed: 100,
            move_speed: 100,
        }
    }

    /// 应用装备加成
    pub fn apply_equip_bonus(&mut self, equip: &EquipStats) {
        self.max_hp += equip.hp_max;
        self.max_qi += equip.qi_max;
        self.max_shen += equip.shen_max;
        self.attack += equip.attack;
        self.defense += equip.defense;
        self.hit_rate += equip.hit_rate;
        self.dodge_rate += equip.dodge_rate;
        self.crit_rate += equip.crit_rate;
        self.crit_damage += equip.crit_damage;
    }

    /// 计算实际伤害
    pub fn calculate_damage(&self, defender: &CombatStats) -> DamageResult {
        let mut rng = rand::thread_rng();

        // 基础伤害 = 攻击力 - 防御力 (最小为1)
        let mut base_damage = self.attack.saturating_sub(defender.defense);
        base_damage = base_damage.max(1);

        // 命中判定
        let hit_roll = rng.gen_range(0..100);
        if hit_roll > self.hit_rate.saturating_sub(defender.dodge_rate) {
            return DamageResult {
                damage: 0,
                is_crit: false,
                is_miss: true,
                is_dodge: false,
            };
        }

        // 闪避判定
        let dodge_roll = rng.gen_range(0..100);
        if dodge_roll < defender.dodge_rate {
            return DamageResult {
                damage: 0,
                is_crit: false,
                is_miss: false,
                is_dodge: true,
            };
        }

        // 暴击判定
        let crit_roll = rng.gen_range(0..100);
        let is_crit = crit_roll < self.crit_rate;

        let final_damage = if is_crit {
            (base_damage * self.crit_damage / 100) as u32
        } else {
            base_damage
        };

        // 浮动 ±10%
        let float_roll = rng.gen_range(-10i32..11);
        let float_factor = 100 + float_roll;
        let final_damage = (final_damage * (float_factor.max(0) as u32) / 100).max(1);

        DamageResult {
            damage: final_damage,
            is_crit,
            is_miss: false,
            is_dodge: false,
        }
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// 受到伤害
    pub fn take_damage(&mut self, damage: u32) -> bool {
        if damage >= self.hp {
            self.hp = 0;
            true // 死亡
        } else {
            self.hp -= damage;
            false
        }
    }

    /// 治疗
    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// 恢复内力
    pub fn restore_qi(&mut self, amount: u32) {
        self.qi = (self.qi + amount).min(self.max_qi);
    }

    /// 恢复精神
    pub fn restore_shen(&mut self, amount: u32) {
        self.shen = (self.shen + amount).min(self.max_shen);
    }

    /// 百分比
    pub fn hp_percent(&self) -> u32 {
        if self.max_hp == 0 {
            return 0;
        }
        self.hp * 100 / self.max_hp
    }

    pub fn qi_percent(&self) -> u32 {
        if self.max_qi == 0 {
            return 0;
        }
        self.qi * 100 / self.max_qi
    }

    pub fn shen_percent(&self) -> u32 {
        if self.max_shen == 0 {
            return 0;
        }
        self.shen * 100 / self.max_shen
    }
}

/// 伤害结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageResult {
    /// 造成的伤害
    pub damage: u32,
    /// 是否暴击
    pub is_crit: bool,
    /// 是否未命中
    pub is_miss: bool,
    /// 是否闪避
    pub is_dodge: bool,
}

impl DamageResult {
    /// 获取伤害描述
    pub fn description(&self) -> String {
        if self.is_miss {
            "未命中".to_string()
        } else if self.is_dodge {
            "被闪避".to_string()
        } else if self.is_crit {
            format!("§c暴击!§r {}点伤害", self.damage)
        } else {
            format!("{}点伤害", self.damage)
        }
    }
}

/// 战斗单位Trait
pub trait Combatant {
    /// 获取名称
    fn get_name(&self) -> &str;
    /// 获取等级
    fn get_level(&self) -> u32;
    /// 获取战斗属性
    fn get_combat_stats(&self) -> &CombatStats;
    /// 获取可变战斗属性
    fn get_combat_stats_mut(&mut self) -> &mut CombatStats;
    /// 是否存活
    fn is_alive(&self) -> bool;

    /// 执行攻击
    fn attack(&mut self, defender: &mut impl Combatant) -> DamageResult {
        let attacker_stats = self.get_combat_stats().clone();
        let defender_stats = defender.get_combat_stats().clone();

        let result = attacker_stats.calculate_damage(&defender_stats);

        if result.damage > 0 {
            let defender_stats_mut = defender.get_combat_stats_mut();
            defender_stats_mut.take_damage(result.damage);
        }

        result
    }

    /// 获取战斗状态文本
    fn get_combat_status(&self) -> String {
        let stats = self.get_combat_stats();
        let hp_bar = Self::render_hp_bar(stats.hp, stats.max_hp);
        format!("{} HP:{}/{} {}",
            self.get_name(),
            stats.hp,
            stats.max_hp,
            hp_bar
        )
    }

    /// 渲染血条
    fn render_hp_bar(current: u32, max: u32) -> String {
        if max == 0 {
            return "§c[----------]§r".to_string();
        }

        let percent = (current * 10 / max.min(1)) as usize;
        let filled = "█".repeat(percent);
        let empty = "─".repeat(10 - percent);

        let color = if percent >= 7 {
            "§g" // 绿色
        } else if percent >= 4 {
            "§y" // 黄色
        } else {
            "§r" // 红色
        };

        format!("§c[{}{}{}{}]§r", color, filled, "§r", empty)
    }
}

/// 战斗状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CombatState {
    /// 未战斗
    Idle,
    /// 战斗中
    Fighting,
    /// 逃跑中
    Fleeing,
    /// 死亡
    Dead,
}

/// 战斗回合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatRound {
    /// 回合数
    pub round_number: u32,
    /// 攻击者ID
    pub attacker_id: String,
    /// 防御者ID
    pub defender_id: String,
    /// 伤害结果
    pub damage_result: DamageResult,
    /// 使用的技能
    pub skill_used: Option<String>,
}

/// 战斗记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatLog {
    /// 战斗ID
    pub combat_id: String,
    /// 参与者
    pub participants: Vec<String>,
    /// 战斗回合
    pub rounds: Vec<CombatRound>,
    /// 开始时间
    pub start_time: i64,
    /// 结束时间
    pub end_time: Option<i64>,
    /// 胜利者
    pub winner: Option<String>,
}

impl CombatLog {
    /// 创建新战斗记录
    pub fn new(participants: Vec<String>) -> Self {
        Self {
            combat_id: ObjectId::new().to_string(),
            participants,
            rounds: Vec::new(),
            start_time: chrono::Utc::now().timestamp(),
            end_time: None,
            winner: None,
        }
    }

    /// 添加回合
    pub fn add_round(&mut self, round: CombatRound) {
        self.rounds.push(round);
    }

    /// 结束战斗
    pub fn end(&mut self, winner: String) {
        self.end_time = Some(chrono::Utc::now().timestamp());
        self.winner = Some(winner);
    }

    /// 渲染战斗日志
    pub fn render(&self) -> String {
        let mut result = format!("=== 战斗记录 {} ===\n", self.combat_id);
        result.push_str(&format!("参与者: {}\n", self.participants.join(", ")));
        result.push_str(&format!("开始时间: {}\n\n", self.start_time));

        for round in &self.rounds {
            result.push_str(&format!("回合{}: ", round.round_number));
            result.push_str(&format!("{} 攻击 {}",
                round.attacker_id,
                round.defender_id
            ));
            if let Some(ref skill) = round.skill_used {
                result.push_str(&format!(" 使用 {}", skill));
            }
            result.push_str(&format!(" - {}\n", round.damage_result.description()));
        }

        if let Some(ref winner) = self.winner {
            result.push_str(&format!("\n胜利者: {}\n", winner));
        }

        if let Some(end_time) = self.end_time {
            let duration = end_time - self.start_time;
            result.push_str(&format!("战斗时长: {}秒\n", duration));
        }

        result
    }
}

/// 战斗管理器
pub struct CombatManager {
    /// 当前进行的战斗
    active_combats: HashMap<String, CombatLog>,
}

impl CombatManager {
    pub fn new() -> Self {
        Self {
            active_combats: HashMap::new(),
        }
    }

    /// 开始战斗
    pub fn start_combat(&mut self, combat_id: String, participants: Vec<String>) {
        let log = CombatLog::new(participants);
        self.active_combats.insert(combat_id, log);
    }

    /// 结束战斗
    pub fn end_combat(&mut self, combat_id: &str, winner: String) -> Option<CombatLog> {
        if let Some(log) = self.active_combats.get_mut(combat_id) {
            log.end(winner);
            Some(log.clone())
        } else {
            None
        }
    }

    /// 获取战斗记录
    pub fn get_combat(&self, combat_id: &str) -> Option<&CombatLog> {
        self.active_combats.get(combat_id)
    }
}

impl Default for CombatManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局战斗管理器
pub static COMBATD: once_cell::sync::Lazy<std::sync::Mutex<CombatManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(CombatManager::default()));
