// gamenv/single/daemons/pkd.rs - PK战斗守护进程
// 1:1 复刻自 txpike9/pikenv/wapmud2/inherit/feature/fight.pike

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::gamenv::combat::skill::{Skill, SkillEffect, SkillTarget, SKILLD};

/// PK模式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PkMode {
    Peace,      // 和平模式 - 不能攻击其他玩家
    Free,       // 自由模式 - 可以攻击任何人（除了和平模式）
    Team,       // 组队模式 - 只能攻击敌方队伍成员
    Guild,      // 帮派模式 - 只能攻击敌方帮派成员
}

impl Default for PkMode {
    fn default() -> Self {
        PkMode::Peace
    }
}

impl PkMode {
    pub fn as_str(&self) -> &str {
        match self {
            PkMode::Peace => "和平模式",
            PkMode::Free => "自由模式",
            PkMode::Team => "组队模式",
            PkMode::Guild => "帮派模式",
        }
    }

    pub fn can_attack(self, other_mode: PkMode) -> bool {
        match (self, other_mode) {
            // 和平模式不能主动攻击
            (PkMode::Peace, _) => false,
            // 和平模式的人可以被自由/组队/帮派模式攻击
            (_, PkMode::Peace) => false,
            // 自由模式可以攻击任何非和平模式
            (PkMode::Free, _) => other_mode != PkMode::Peace,
            // 其他模式需要进一步判断
            _ => true,
        }
    }
}

/// PK值等级
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PkLevel {
    Citizen = 0,     // 良民 §G
    Gray = 1,        // 灰名 §C (1-19)
    Red = 20,        // 红名 §Y (20-49)
    Evil = 50,       // 恶人 §R (50-99)
    Demon = 100,     // 恶魔 §X (100+)
}

impl PkLevel {
    pub fn from_value(value: i32) -> Self {
        if value < 1 {
            PkLevel::Citizen
        } else if value < 20 {
            PkLevel::Gray
        } else if value < 50 {
            PkLevel::Red
        } else if value < 100 {
            PkLevel::Evil
        } else {
            PkLevel::Demon
        }
    }

    pub fn color_code(&self) -> &str {
        match self {
            PkLevel::Citizen => "#00ff00",  // §G
            PkLevel::Gray => "#cccccc",      // §C
            PkLevel::Red => "#ffff00",       // §Y
            PkLevel::Evil => "#ff0000",      // §R
            PkLevel::Demon => "#ff00ff",     // §X
        }
    }

    pub fn name(&self) -> &str {
        match self {
            PkLevel::Citizen => "良民",
            PkLevel::Gray => "灰名",
            PkLevel::Red => "红名",
            PkLevel::Evil => "恶人",
            PkLevel::Demon => "恶魔",
        }
    }
}

/// 战斗状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CombatStatus {
    Normal,      // 正常
    Fighting,    // 战斗中
    Escaped,     // 逃跑
    Dead,        // 死亡
    Unconscious, // 昏迷
}

/// 战斗动作
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CombatAction {
    Attack,
    Escape,
    Perform(String),
    Cast(String),
    Surrender,
}

/// 战斗者数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatStats {
    pub id: String,
    pub name: String,
    pub name_cn: String,
    pub level: i32,

    // 战斗属性
    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
    pub jing: i32,       // 精力
    pub jing_max: i32,
    pub qi: i32,         // 内力
    pub qi_max: i32,

    // 战斗技能
    pub attack: i32,     // 攻击力
    pub defense: i32,    // 防御力
    pub dodge: i32,      // 轻功
    pub parry: i32,      // 招架

    // 状态
    pub pk_mode: PkMode,
    pub pk_value: i32,
    pub kill_streak: i32,
    pub is_killing: bool,  // 是否想杀死对方
}

impl CombatStats {
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn hp_percent(&self) -> i32 {
        if self.hp_max == 0 {
            0
        } else {
            (self.hp * 100 / self.hp_max).max(0).min(100)
        }
    }

    /// 检查是否可以攻击对方
    pub fn can_attack(&self, target: &CombatStats) -> Result<(), String> {
        // 检查是否在和平区域
        // TODO: 检查房间是否和平

        // 检查PK模式
        if !self.pk_mode.can_attack(target.pk_mode) {
            return Err(format!("对方的PK模式不允许被攻击"));
        }

        // 检查是否已经在战斗中
        // TODO: 检查战斗状态

        Ok(())
    }
}

/// 战斗回合结果
#[derive(Clone, Debug)]
pub struct CombatRound {
    pub round_number: u32,
    pub attacker_damage: i32,
    pub defender_damage: i32,
    pub attacker_hp: i32,
    pub defender_hp: i32,
    pub log: Vec<String>,
    pub ended: bool,
    pub winner: Option<String>,
    pub skill_used: Option<String>,  // 使用的技能
    pub skill_effect: Option<String>, // 技能效果描述
}

/// 战斗中的技能状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattleSkillState {
    /// 技能ID
    pub skill_id: String,
    /// 当前冷却回合
    pub current_cooldown: u32,
    /// 原始冷却回合
    pub base_cooldown: u32,
}

impl BattleSkillState {
    pub fn new(skill_id: String, base_cooldown: u32) -> Self {
        Self {
            skill_id,
            current_cooldown: 0,
            base_cooldown,
        }
    }

    /// 检查是否可用
    pub fn is_ready(&self) -> bool {
        self.current_cooldown == 0
    }

    /// 使用技能后设置冷却
    pub fn use_skill(&mut self) {
        self.current_cooldown = self.base_cooldown;
    }

    /// 每回合减少冷却
    pub fn tick_cooldown(&mut self) {
        if self.current_cooldown > 0 {
            self.current_cooldown -= 1;
        }
    }
}

/// 战斗者的技能列表
#[derive(Clone, Debug)]
pub struct FighterSkills {
    /// 技能状态映射 (skill_id -> state)
    pub skills: HashMap<String, BattleSkillState>,
    /// 本回合使用的技能
    pub this_round_skill: Option<String>,
}

impl FighterSkills {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            this_round_skill: None,
        }
    }

    /// 添加技能
    pub fn add_skill(&mut self, skill_id: String, cooldown: u32) {
        self.skills.insert(skill_id.clone(), BattleSkillState::new(skill_id, cooldown));
    }

    /// 检查技能是否可用
    pub fn can_use_skill(&self, skill_id: &str) -> bool {
        if let Some(state) = self.skills.get(skill_id) {
            state.is_ready()
        } else {
            false
        }
    }

    /// 使用技能
    pub fn use_skill(&mut self, skill_id: &str) -> bool {
        if let Some(state) = self.skills.get_mut(skill_id) {
            if state.is_ready() {
                state.use_skill();
                self.this_round_skill = Some(skill_id.to_string());
                return true;
            }
        }
        false
    }

    /// 回合结束，减少冷却
    pub fn tick_all_cooldowns(&mut self) {
        for state in self.skills.values_mut() {
            state.tick_cooldown();
        }
        self.this_round_skill = None;
    }

    /// 获取可用技能列表
    pub fn get_available_skills(&self) -> Vec<String> {
        self.skills.iter()
            .filter(|(_, s)| s.is_ready())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 获取所有技能信息（用于UI显示）
    pub fn get_skills_info(&self) -> Vec<(String, u32, u32)> {
        self.skills.iter()
            .map(|(id, s)| (id.clone(), s.current_cooldown, s.base_cooldown))
            .collect()
    }
}

/// PK战斗会话
#[derive(Clone, Debug)]
pub struct PkBattle {
    pub battle_id: String,
    pub challenger: CombatStats,
    pub defender: CombatStats,
    pub round: u32,
    pub total_damage_dealt: i32,
    pub total_damage_taken: i32,
    pub start_time: i64,
    pub status: CombatStatus,
    pub combat_log: Vec<String>,
    /// 挑战者技能状态
    pub challenger_skills: FighterSkills,
    /// 防守者技能状态
    pub defender_skills: FighterSkills,
    /// 玩家选择的技能 (player_id -> skill_id)
    pub pending_skills: HashMap<String, String>,
    /// 上一回合执行时间（用于心跳自动推进）
    pub last_round_time: i64,
    /// 战斗发生的房间ID
    pub room_id: String,
}

impl PkBattle {
    pub fn new(challenger: CombatStats, defender: CombatStats, room_id: String) -> Self {
        let battle_id = format!("pk_{}_{}", challenger.id, defender.id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 初始化技能列表 - 每个玩家默认有基础攻击技能
        let mut challenger_skills = FighterSkills::new();
        let mut defender_skills = FighterSkills::new();

        // 为双方添加基础技能
        challenger_skills.add_skill("skill_basic_attack".to_string(), 0);
        defender_skills.add_skill("skill_basic_attack".to_string(), 0);

        // 根据等级添加更多技能
        Self::add_skills_for_level(&mut challenger_skills, challenger.level as u32);
        Self::add_skills_for_level(&mut defender_skills, defender.level as u32);

        Self {
            battle_id,
            challenger,
            defender,
            round: 0,
            total_damage_dealt: 0,
            total_damage_taken: 0,
            start_time: now,
            status: CombatStatus::Fighting,
            combat_log: vec![],
            challenger_skills,
            defender_skills,
            pending_skills: HashMap::new(),
            last_round_time: now,
            room_id,
        }
    }

    /// 检查是否是NPC（ID包含'/'）
    pub fn is_npc(id: &str) -> bool {
        id.contains('/')
    }

    /// 检查战斗中是否包含NPC
    pub fn has_npc(&self) -> bool {
        Self::is_npc(&self.challenger.id) || Self::is_npc(&self.defender.id)
    }

    /// 获取NPC的ID（如果存在）
    pub fn get_npc_id(&self) -> Option<String> {
        if Self::is_npc(&self.challenger.id) {
            Some(self.challenger.id.clone())
        } else if Self::is_npc(&self.defender.id) {
            Some(self.defender.id.clone())
        } else {
            None
        }
    }

    /// 检查是否应该自动执行下一回合（有NPC且距离上次执行超过2秒）
    pub fn should_auto_execute(&self) -> bool {
        if !self.has_npc() {
            return false;
        }
        if self.status != CombatStatus::Fighting {
            return false;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        now - self.last_round_time >= 2  // 2秒后自动执行
    }

    /// NPC自动选择技能
    pub fn npc_select_skill(&self, npc_id: &str) -> String {
        let skills = if npc_id == &self.challenger.id {
            &self.challenger_skills
        } else {
            &self.defender_skills
        };

        // 获取所有可用技能
        let available = skills.get_available_skills();
        if available.is_empty() {
            return "skill_basic_attack".to_string();
        }

        // 30%几率使用技能，否则普通攻击
        if rand::random::<f32>() < 0.3 && available.len() > 1 {
            // 随机选择一个非基础技能
            let special_skills: Vec<_> = available.iter()
                .filter(|s| *s != "skill_basic_attack")
                .collect();
            if !special_skills.is_empty() {
                return special_skills[rand::random::<usize>() % special_skills.len()].clone();
            }
        }

        "skill_basic_attack".to_string()
    }

    /// 根据等级添加技能
    fn add_skills_for_level(skills: &mut FighterSkills, level: u32) {
        // Lv.1+: 强力一击 (让新手也能用)
        skills.add_skill("skill_power_strike".to_string(), 3);

        // Lv.2+: 快速连击
        if level >= 2 {
            skills.add_skill("skill_quick_strike".to_string(), 2);
        }
        // Lv.5+: 防御姿态
        if level >= 5 {
            skills.add_skill("skill_defense".to_string(), 4);
        }
        // Lv.10+: 治疗术
        if level >= 10 {
            skills.add_skill("skill_heal".to_string(), 5);
        }
        // Lv.15+: 旋风斩
        if level >= 15 {
            skills.add_skill("skill_whirlwind".to_string(), 4);
        }
        // Lv.20+: 致命一击
        if level >= 20 {
            skills.add_skill("skill_critical_strike".to_string(), 5);
        }
    }

    /// 获取指定玩家的技能状态
    pub fn get_player_skills(&self, player_id: &str) -> Option<&FighterSkills> {
        if player_id == self.challenger.id {
            Some(&self.challenger_skills)
        } else if player_id == self.defender.id {
            Some(&self.defender_skills)
        } else {
            None
        }
    }

    /// 获取指定玩家的可变技能状态
    pub fn get_player_skills_mut(&mut self, player_id: &str) -> Option<&mut FighterSkills> {
        if player_id == self.challenger.id {
            Some(&mut self.challenger_skills)
        } else if player_id == self.defender.id {
            Some(&mut self.defender_skills)
        } else {
            None
        }
    }

    /// 玩家选择技能
    pub fn select_skill(&mut self, player_id: &str, skill_id: &str) -> Result<(), String> {
        // 检查玩家是否在战斗中
        let skills = self.get_player_skills(player_id)
            .ok_or("你不是战斗参与者！")?;

        // 检查技能是否可用
        if !skills.can_use_skill(skill_id) {
            return Err("技能不可用或正在冷却中！".to_string());
        }

        // 检查内力是否足够
        let skill = SKILLD.lock().ok().and_then(|s| s.get_skill(skill_id).cloned());
        if let Some(skill) = skill {
            let player_qi = if player_id == self.challenger.id {
                self.challenger.qi
            } else {
                self.defender.qi
            };
            if player_qi < skill.qi_cost as i32 {
                return Err(format!("内力不足！需要 {}", skill.qi_cost));
            }
        }

        self.pending_skills.insert(player_id.to_string(), skill_id.to_string());
        Ok(())
    }

    /// 使用技能计算伤害（不修改defender，返回伤害值）
    fn calculate_skill_damage(
        &self,
        attacker: &CombatStats,
        defender: &CombatStats,
        skill_id: &str,
        attacker_id: &str,
    ) -> (i32, String) {
        let skill_mgr = match SKILLD.lock() {
            Ok(guard) => guard,
            Err(_) => return (self.calculate_damage(attacker, defender), "普通攻击".to_string()),
        };

        let skill = match skill_mgr.get_skill(skill_id) {
            Some(s) => s,
            None => return (self.calculate_damage(attacker, defender), "普通攻击".to_string()),
        };

        // 获取玩家技能等级（如果有）
        let skill_level = self.get_player_skill_level(attacker_id, skill_id);
        // 简化的技能等级加成: 每级增加5%伤害
        let skill_bonus = skill_level as f32 * 0.05;

        let mut total_damage = 0;
        let mut effect_desc = format!("§Y使用{}§N", skill.name_cn);
        if skill_level > 0 {
            effect_desc.push_str(&format!("(Lv.{})", skill_level));
        }

        // 应用技能效果
        for effect in &skill.effects {
            match effect {
                SkillEffect::Damage(multiplier) => {
                    // 基础伤害 + 技能等级加成
                    let base_damage = ((attacker.attack as f32 * multiplier * (1.0 + skill_bonus)) as i32 - defender.defense / 2).max(1);
                    total_damage += base_damage;
                    effect_desc.push_str(&format!(" 造成§R{}§N点伤害", base_damage));
                }
                SkillEffect::FixedDamage(dmg) => {
                    let damage = ((*dmg as f32 * (1.0 + skill_bonus)) as i32 - defender.defense / 2).max(1);
                    total_damage += damage;
                    effect_desc.push_str(&format!(" 造成§R{}§N点伤害", damage));
                }
                SkillEffect::HealPercent(percent) => {
                    // 治疗效果不在此处处理，需要在回合开始时
                }
                SkillEffect::HealFixed(amount) => {
                    // 治疗效果不在此处处理
                }
                _ => {}
            }
        }

        // 应用闪避和暴击
        let dodge_chance = defender.dodge as f64 / (attacker.dodge + defender.dodge) as f64;
        if rand::random::<f64>() < dodge_chance && total_damage > 0 {
            return (0, format!("§Y使用{}§N 被§R闪避§N", skill.name_cn));
        }

        // 暴击检查 (技能等级越高，暴击率越高)
        let crit_chance = 0.1 + (skill_level as f64 * 0.01);
        let is_crit = rand::random::<f64>() < crit_chance.min(0.5);
        if is_crit && total_damage > 0 {
            total_damage = (total_damage as f64 * 1.5) as i32;
            effect_desc.push_str(" §c暴击!§N");
        }

        (total_damage, effect_desc)
    }

    /// 获取玩家技能等级（简化版，暂时返回固定值）
    fn get_player_skill_level(&self, player_id: &str, skill_id: &str) -> u32 {
        // TODO: 实现从PlayerState获取玩家技能等级
        // 目前返回固定值，等PlayerStateManager API完善后再实现
        10
    }

    /// 计算伤害
    pub fn calculate_damage(&self, attacker: &CombatStats, defender: &CombatStats) -> i32 {
        // 基础伤害 = 攻击 - 防御 (最低1)
        let base_damage = (attacker.attack - defender.defense / 2).max(1);

        // 命中检查 (轻功闪避)
        let dodge_chance = defender.dodge as f64 / (attacker.dodge + defender.dodge) as f64;
        if rand::random::<f64>() < dodge_chance {
            return 0; // 被闪避
        }

        // 招架检查
        let parry_chance = defender.parry as f64 / (attacker.attack + defender.parry) as f64;
        let mut damage = base_damage;
        if rand::random::<f64>() < parry_chance {
            damage = base_damage / 2; // 招架减半
        }

        // 暴击检查
        let crit_chance = 0.1; // 10%暴击
        if rand::random::<f64>() < crit_chance {
            damage = (damage as f64 * 1.5) as i32;
        }

        // +/- 10% 随机浮动
        let variance = (damage as f32 * 0.1) as i32;
        damage = damage + rand::random::<i32>().rem_euclid(variance * 2 + 1) - variance;

        damage.max(1)
    }

    /// 执行一个战斗回合
    pub fn execute_round(&mut self) -> CombatRound {
        // 更新最后执行时间
        self.last_round_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.round += 1;

        let mut attacker_damage = 0;
        let mut defender_damage = 0;
        let mut log = vec![];
        let mut skill_used = None;
        let mut skill_effect = None;

        // 获取挑战者选择的技能（如果有）
        let challenger_skill_id = self.pending_skills.remove(&self.challenger.id)
            .unwrap_or_else(|| "skill_basic_attack".to_string());

        // 获取防守者选择的技能（如果有）
        let defender_skill_id = self.pending_skills.remove(&self.defender.id)
            .unwrap_or_else(|| "skill_basic_attack".to_string());

        // 检查并消耗内力
        let challenger_skill = SKILLD.lock().ok().and_then(|s| s.get_skill(&challenger_skill_id).cloned());
        if let Some(skill) = challenger_skill {
            if self.challenger.qi >= skill.qi_cost as i32 {
                self.challenger.qi -= skill.qi_cost as i32;
            }
        }
        let defender_skill = SKILLD.lock().ok().and_then(|s| s.get_skill(&defender_skill_id).cloned());
        if let Some(skill) = defender_skill {
            if self.defender.qi >= skill.qi_cost as i32 {
                self.defender.qi -= skill.qi_cost as i32;
            }
        }

        // 挑战者攻击（使用技能）
        if self.challenger.is_alive() {
            let (damage, effect_desc) = self.calculate_skill_damage(
                &self.challenger,
                &self.defender,
                &challenger_skill_id,
                &self.challenger.id,
            );
            attacker_damage = damage;

            if challenger_skill_id != "skill_basic_attack" {
                skill_used = Some(challenger_skill_id.clone());
                skill_effect = Some(effect_desc.clone());
            }

            log.push(format!(
                "§Y{}§N{}",
                self.challenger.name_cn,
                effect_desc
            ));

            if attacker_damage > 0 {
                self.defender.hp = (self.defender.hp - attacker_damage).max(0);
                self.total_damage_dealt += attacker_damage;
            }

            // 标记技能已使用
            self.challenger_skills.use_skill(&challenger_skill_id);
        }

        // 防守者反击（如果还活着）
        if self.defender.is_alive() && self.defender.hp > 0 {
            let (damage, effect_desc) = self.calculate_skill_damage(
                &self.defender,
                &self.challenger,
                &defender_skill_id,
                &self.defender.id,
            );
            defender_damage = damage;

            log.push(format!(
                "§R{}§N{}",
                self.defender.name_cn,
                effect_desc
            ));

            if defender_damage > 0 {
                self.challenger.hp = (self.challenger.hp - defender_damage).max(0);
                self.total_damage_taken += defender_damage;
            }

            // 标记技能已使用
            self.defender_skills.use_skill(&defender_skill_id);
        }

        // 减少所有技能的冷却
        self.challenger_skills.tick_all_cooldowns();
        self.defender_skills.tick_all_cooldowns();

        // 检查战斗是否结束
        let ended = !self.challenger.is_alive() || !self.defender.is_alive();
        let winner = if ended {
            if !self.challenger.is_alive() && !self.defender.is_alive() {
                None // 平局
            } else if !self.defender.is_alive() {
                Some(self.challenger.id.clone())
            } else {
                Some(self.defender.id.clone())
            }
        } else {
            None
        };

        if winner.is_some() {
            self.status = CombatStatus::Dead;
        }

        self.combat_log.extend(log.clone());

        CombatRound {
            round_number: self.round,
            attacker_damage,
            defender_damage,
            attacker_hp: self.challenger.hp,
            defender_hp: self.defender.hp,
            log,
            ended,
            winner,
            skill_used,
            skill_effect,
        }
    }

    /// 生成战斗结束信息
    pub fn generate_result(&self) -> String {
        let winner = if !self.challenger.is_alive() && !self.defender.is_alive() {
            "平局"
        } else if !self.defender.is_alive() {
            &self.challenger.name_cn
        } else {
            &self.defender.name_cn
        };

        let mut output = String::new();

        // 返回按钮 - 放在最上面
        output.push_str(&format!("[§Y返回房间§N:look]\n\n"));

        output.push_str(&format!("§Y胜利者: {}§N\n", winner));
        output.push_str(&format!("战斗回合: {}\n", self.round));
        output.push_str(&format!("战斗时长: {}秒\n",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64 - self.start_time
        ));

        output
    }

    /// 生成战斗状态（用于前端显示）
    pub fn generate_status(&self) -> String {
        let mut output = String::new();

        // 操作按钮 - 放在最上面方便操作
        output.push_str(&format!("§H【操作】§N\n"));
        output.push_str("[继续战斗:pk continue]\n");
        output.push_str("[§Y逃跑§N:escape]\n");
        output.push_str("[§Y投降§N:surrender]\n");
        output.push_str("\n");

        output.push_str(&format!("§Y回合: {}§N\n\n", self.round));

        // 挑战者状态 - 使用颜色代码
        let (challenger_hp_color, challenger_hp_end) = if self.challenger.hp_percent() > 50 {
            ("§G", "§N")  // 绿色
        } else if self.challenger.hp_percent() > 20 {
            ("§Y", "§N")  // 黄色
        } else {
            ("§R", "§N")  // 红色
        };

        output.push_str(&format!("§Y【挑战者】§N {} (Lv.{})\n",
            self.challenger.name_cn, self.challenger.level));
        output.push_str(&format!("§Y生命: {}{}/{}{}\n",
            challenger_hp_color, self.challenger.hp, self.challenger.hp_max, challenger_hp_end));
        output.push_str(&format!("§Y内力: {}/{}\n",
            self.challenger.qi, self.challenger.qi_max));

        // 防守者状态 - 使用颜色代码
        let (defender_hp_color, defender_hp_end) = if self.defender.hp_percent() > 50 {
            ("§G", "§N")  // 绿色
        } else if self.defender.hp_percent() > 20 {
            ("§Y", "§N")  // 黄色
        } else {
            ("§R", "§N")  // 红色
        };

        output.push_str(&format!("\n§R【防守者】§N {} (Lv.{})\n",
            self.defender.name_cn, self.defender.level));
        output.push_str(&format!("§R生命: {}{}/{}{}\n",
            defender_hp_color, self.defender.hp, self.defender.hp_max, defender_hp_end));
        output.push_str(&format!("§R内力: {}/{}\n",
            self.defender.qi, self.defender.qi_max));

        output
    }

    /// 为指定玩家生成战斗状态（包含技能列表）
    pub fn generate_status_for_player(&self, player_id: &str) -> String {
        let mut output = String::new();

        let skills = match self.get_player_skills(player_id) {
            Some(s) => s,
            None => return "你不在战斗中！".to_string(),
        };

        // 操作按钮 - 放在同一行
        output.push_str("[查看技能:skills] [继续战斗:pk continue] [逃跑:escape] [投降:surrender]\n\n");

        // 显示已选择的技能（如果有）
        if let Some(ref pending_skill_id) = self.pending_skills.get(player_id) {
            if let Some(skill) = SKILLD.lock().ok().and_then(|s| s.get_skill(pending_skill_id).cloned()) {
                output.push_str(&format!("§Y已选择技能: {}§N\n\n", skill.name_cn));
            }
        } else {
            output.push_str("§c未选择技能（使用普通攻击）§N\n\n");
        }

        output.push_str(&format!("§Y回合: {}§N\n\n", self.round));

        // 挑战者状态
        let (challenger_hp_color, challenger_hp_end) = if self.challenger.hp_percent() > 50 {
            ("§G", "§N")
        } else if self.challenger.hp_percent() > 20 {
            ("§Y", "§N")
        } else {
            ("§R", "§N")
        };

        output.push_str(&format!("§Y【挑战者】§N {} (Lv.{})\n",
            self.challenger.name_cn, self.challenger.level));
        output.push_str(&format!("§Y生命: {}{}/{}{} 内力: {}/{}\n",
            challenger_hp_color, self.challenger.hp, self.challenger.hp_max, challenger_hp_end,
            self.challenger.qi, self.challenger.qi_max));

        // 防守者状态
        let (defender_hp_color, defender_hp_end) = if self.defender.hp_percent() > 50 {
            ("§G", "§N")
        } else if self.defender.hp_percent() > 20 {
            ("§Y", "§N")
        } else {
            ("§R", "§N")
        };

        output.push_str(&format!("\n§R【防守者】§N {} (Lv.{})\n",
            self.defender.name_cn, self.defender.level));
        output.push_str(&format!("§R生命: {}{}/{}{} 内力: {}/{}\n",
            defender_hp_color, self.defender.hp, self.defender.hp_max, defender_hp_end,
            self.defender.qi, self.defender.qi_max));

        // 显示冷却中的技能
        let all_skills = skills.get_skills_info();
        let cooling_skills: Vec<_> = all_skills.iter()
            .filter(|(_, cd, _)| *cd > 0)
            .collect();

        if !cooling_skills.is_empty() {
            output.push_str(&format!("\n§c【冷却中】§N\n"));
            for (skill_id, cd, _) in cooling_skills {
                // Clone the skill to avoid lifetime issues
                let skill = SKILLD.lock().ok().and_then(|s| s.get_skill(skill_id).cloned());
                if let Some(skill) = skill {
                    output.push_str(&format!("{} (冷却:{}回合)\n", skill.name_cn, cd));
                }
            }
        }

        output
    }

    /// 为指定玩家生成战斗状态（包含战斗日志）
    pub fn generate_status_with_log(&self, player_id: &str, combat_log: &[String], skill_effect: Option<&String>) -> String {
        let mut output = String::new();

        let skills = match self.get_player_skills(player_id) {
            Some(s) => s,
            None => return "你不在战斗中！".to_string(),
        };

        // 操作按钮 - 放在同一行
        output.push_str("[查看技能:skills] [继续战斗:pk continue] [逃跑:escape] [投降:surrender]\n\n");

        // 战斗日志 - 放在操作按钮下面
        if !combat_log.is_empty() {
            output.push_str(&format!("§H【本回合】§N\n"));
            for log in combat_log {
                output.push_str(log);
                output.push_str("\n");
            }

            // 显示使用的技能效果
            if let Some(effect) = skill_effect {
                output.push_str(&format!("\n§Y技能效果: {}§N\n", effect));
            }
            output.push_str("\n");
        }

        // 显示已选择的技能（如果有）
        if let Some(ref pending_skill_id) = self.pending_skills.get(player_id) {
            if let Some(skill) = SKILLD.lock().ok().and_then(|s| s.get_skill(pending_skill_id).cloned()) {
                output.push_str(&format!("§Y已选择技能: {}§N\n\n", skill.name_cn));
            }
        } else {
            output.push_str("§c未选择技能（使用普通攻击）§N\n\n");
        }

        output.push_str(&format!("§Y回合: {}§N\n\n", self.round));

        // 挑战者状态
        let (challenger_hp_color, challenger_hp_end) = if self.challenger.hp_percent() > 50 {
            ("§G", "§N")
        } else if self.challenger.hp_percent() > 20 {
            ("§Y", "§N")
        } else {
            ("§R", "§N")
        };

        output.push_str(&format!("§Y【挑战者】§N {} (Lv.{})\n",
            self.challenger.name_cn, self.challenger.level));
        output.push_str(&format!("§Y生命: {}{}/{}{} 内力: {}/{}\n",
            challenger_hp_color, self.challenger.hp, self.challenger.hp_max, challenger_hp_end,
            self.challenger.qi, self.challenger.qi_max));

        // 防守者状态
        let (defender_hp_color, defender_hp_end) = if self.defender.hp_percent() > 50 {
            ("§G", "§N")
        } else if self.defender.hp_percent() > 20 {
            ("§Y", "§N")
        } else {
            ("§R", "§N")
        };

        output.push_str(&format!("\n§R【防守者】§N {} (Lv.{})\n",
            self.defender.name_cn, self.defender.level));
        output.push_str(&format!("§R生命: {}{}/{}{} 内力: {}/{}\n",
            defender_hp_color, self.defender.hp, self.defender.hp_max, defender_hp_end,
            self.defender.qi, self.defender.qi_max));

        // 显示冷却中的技能
        let all_skills = skills.get_skills_info();
        let cooling_skills: Vec<_> = all_skills.iter()
            .filter(|(_, cd, _)| *cd > 0)
            .collect();

        if !cooling_skills.is_empty() {
            output.push_str(&format!("\n§c【冷却中】§N\n"));
            for (skill_id, cd, _) in cooling_skills {
                // Clone the skill to avoid lifetime issues
                let skill = SKILLD.lock().ok().and_then(|s| s.get_skill(skill_id).cloned());
                if let Some(skill) = skill {
                    output.push_str(&format!("{} (冷却:{}回合)\n", skill.name_cn, cd));
                }
            }
        }

        output
    }

    /// 生成技能列表（供 skills 命令使用）
    pub fn generate_skills_list(&self, player_id: &str) -> String {
        let mut output = String::new();

        let skills = match self.get_player_skills(player_id) {
            Some(s) => s,
            None => return "你不在战斗中！\n[返回:look]".to_string(),
        };

        let player_qi = if player_id == self.challenger.id {
            self.challenger.qi
        } else {
            self.defender.qi
        };

        output.push_str("§H【战斗技能】§N\n\n");

        // 获取所有技能信息
        let all_skills_info = skills.get_skills_info();

        if all_skills_info.is_empty() {
            output.push_str("§c没有可用技能§N\n");
        } else {
            for (skill_id, current_cd, _base_cd) in &all_skills_info {
                // 跳过基础攻击（它是默认技能）
                if skill_id == "skill_basic_attack" {
                    continue;
                }

                // 从 SKILLD 获取技能信息
                let skill = SKILLD.lock().ok().and_then(|s| s.get_skill(skill_id).cloned());

                if let Some(skill) = skill {
                    let can_afford = player_qi >= skill.qi_cost as i32;
                    let is_ready = *current_cd == 0;

                    // 构建技能选项
                    let status = if !is_ready {
                        format!("§c(冷却:{})§N", current_cd)
                    } else if !can_afford {
                        format!("§c(内力不足-需{})§N", skill.qi_cost)
                    } else {
                        String::new()
                    };

                    // 显示技能按钮 - 使用中文名称
                    if is_ready && can_afford {
                        output.push_str(&format!(
                            "§Y[{}:cast {}]§N\n",
                            skill.name_cn, skill_id
                        ));
                    } else {
                        output.push_str(&format!(
                            "§c{} {}§N\n",
                            skill.name_cn, status.trim()
                        ));
                    }
                }
            }
        }

        // 默认技能提示 - 移除颜色代码避免解析问题
        output.push_str("\n§H默认技能§N\n基础攻击 (无消耗)\n");

        // 返回按钮
        output.push_str("[返回:look]\n");

        output
    }
}

/// PK守护进程
pub struct PkDaemon {
    battles: Arc<RwLock<HashMap<String, PkBattle>>>,
    // 玩家ID -> 战斗ID 映射（用于快速查找玩家所在的战斗）
    player_battles: Arc<RwLock<HashMap<String, String>>>,
}

impl PkDaemon {
    pub fn new() -> Self {
        Self {
            battles: Arc::new(RwLock::new(HashMap::new())),
            player_battles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 发起PK挑战
    pub async fn challenge(&self, challenger: CombatStats, defender: CombatStats, room_id: String) -> Result<PkBattle, String> {
        tracing::info!("[PKD] challenge() called with room_id='{}'", room_id);
        // 检查是否可以发起攻击
        challenger.can_attack(&defender)?;

        // 检查是否已经在战斗中（使用player_battles映射）
        let player_battles = self.player_battles.read().await;
        if player_battles.contains_key(&challenger.id) {
            println!("[PKD] Challenge FAILED: {} is already in battle", challenger.id);
            return Err("你正在战斗中！".to_string());
        }
        if player_battles.contains_key(&defender.id) {
            println!("[PKD] Challenge FAILED: {} is already in battle", defender.id);
            return Err("对方正在战斗中！".to_string());
        }
        drop(player_battles);

        println!("[PKD] Challenge: {} vs {} in room {}", challenger.id, defender.id, room_id);

        // 创建战斗
        let challenger_id = challenger.id.clone();
        let defender_id = defender.id.clone();
        let battle = PkBattle::new(challenger, defender, room_id);
        let battle_id = battle.battle_id.clone();

        let mut battles = self.battles.write().await;
        let mut player_battles = self.player_battles.write().await;

        // 建立玩家ID -> 战斗ID 的映射
        player_battles.insert(challenger_id, battle_id.clone());
        player_battles.insert(defender_id, battle_id.clone());

        // 只用 battle_id 存储战斗（最后插入，因为battle会被移动）
        battles.insert(battle_id.clone(), battle);

        // 由于battle被移动，我们无法返回它，需要重新获取
        Ok(battles.get(&battle_id).unwrap().clone())
    }

    /// 获取战斗
    pub async fn get_battle(&self, battle_id: &str) -> Option<PkBattle> {
        self.battles.read().await.get(battle_id).cloned()
    }

    /// 获取玩家当前的战斗
    pub async fn get_player_battle(&self, player_id: &str) -> Option<PkBattle> {
        let player_battles = self.player_battles.read().await;
        if let Some(battle_id) = player_battles.get(player_id) {
            self.battles.read().await.get(battle_id).cloned()
        } else {
            None
        }
    }

    /// 执行下一个回合
    pub async fn next_round(&self, battle_id: &str) -> Option<CombatRound> {
        let mut battles = self.battles.write().await;
        if let Some(battle) = battles.get_mut(battle_id) {
            if battle.status == CombatStatus::Fighting {
                let round = battle.execute_round();
                Some(round)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 结束战斗
    pub async fn end_battle(&self, battle_id: &str) -> Option<PkBattle> {
        println!("[PKD] end_battle called for {}", battle_id);
        let mut battles = self.battles.write().await;
        let mut player_battles = self.player_battles.write().await;

        if let Some(battle) = battles.remove(battle_id) {
            // 检查是否有NPC死亡，通知runtime_npc_d
            Self::handle_npc_death(&battle).await;

            // 清理玩家映射
            println!("[PKD] Removing player mappings: {} and {}", battle.challenger.id, battle.defender.id);
            player_battles.remove(&battle.challenger.id);
            player_battles.remove(&battle.defender.id);
            println!("[PKD] Battle ended, remaining battles: {}", battles.len());
            Some(battle)
        } else {
            println!("[PKD] Battle {} not found!", battle_id);
            None
        }
    }

    /// 处理NPC死亡
    async fn handle_npc_death(battle: &PkBattle) {
        use crate::gamenv::world::get_world;

        tracing::info!("=== handle_npc_death called ===");
        tracing::info!("Challenger: {} (alive: {})", battle.challenger.id, battle.challenger.is_alive());
        tracing::info!("Defender: {} (alive: {})", battle.defender.id, battle.defender.is_alive());
        tracing::info!("Battle room: {}", battle.room_id);

        // 检查防守者是否是NPC（ID格式为 room_id/npc_id）
        let defender_id = &battle.defender.id;
        let challenger_id = &battle.challenger.id;

        // 判断哪个是NPC
        let (npc_id, killed) = if defender_id.contains('/') {
            (defender_id, !battle.defender.is_alive())
        } else if challenger_id.contains('/') {
            (challenger_id, !battle.challenger.is_alive())
        } else {
            tracing::info!("Both are players, skipping NPC death handling");
            return; // 双方都是玩家，不需要处理
        };

        tracing::info!("NPC identified: npc_id={}, killed={}", npc_id, killed);

        if killed {
            // 使用battle中的room_id（战斗发生的实际房间）
            let room_id = &battle.room_id;
            let template_id = npc_id.to_string();  // 完整ID作为模板ID

            tracing::info!("[PKD] NPC {} killed in room {}, notifying room for respawn",
                template_id, room_id);

            // 先从runtime_npc_d中标记NPC为死亡
            {
                use crate::gamenv::single::daemons::runtime_npc_d::get_runtime_npc_d;
                let runtime_npc_d_read = get_runtime_npc_d().read().await;
                let alive_before = runtime_npc_d_read.get_alive_npcs(room_id);
                tracing::info!("[PKD] BEFORE: runtime_npc_d alive_npcs for room {}: {:?}", room_id, alive_before);
                drop(runtime_npc_d_read);

                let mut runtime_npc_d = get_runtime_npc_d().write().await;
                runtime_npc_d.on_npc_killed(&template_id, room_id);
                tracing::info!("[PKD] Marked NPC {} as dead in runtime_npc_d in room {}", npc_id, room_id);
            }

            // 验证runtime_npc_d更新后的状态
            {
                use crate::gamenv::single::daemons::runtime_npc_d::get_runtime_npc_d;
                let runtime_npc_d = get_runtime_npc_d().read().await;
                let alive_after = runtime_npc_d.get_alive_npcs(room_id);
                tracing::info!("[PKD] AFTER: runtime_npc_d alive_npcs for room {}: {:?}", room_id, alive_after);
            }

            // 通知房间的NPC死亡，由房间重置负责刷新
            let world = get_world();
            let mut world = world.write().await;
            if let Some(room) = world.rooms.get_mut(room_id) {
                tracing::info!("[PKD] Found room {}, current NPCs: {:?}", room_id, room.npcs);
                room.on_npc_killed(template_id.clone());
                // 从房间NPC列表中移除一个实例（只移除第一个匹配的）
                let before_count = room.npcs.len();
                if let Some(pos) = room.npcs.iter().position(|id| id == npc_id) {
                    room.npcs.remove(pos);
                    tracing::info!("[PKD] Removed NPC at position {}", pos);
                } else {
                    tracing::warn!("[PKD] NPC {} not found in room.npcs!", npc_id);
                }
                let after_count = room.npcs.len();
                tracing::info!("[PKD] AFTER: room.npcs = {:?}", room.npcs);
                tracing::info!("[PKD] NPC {} removed from room {} NPCs list: {} -> {} NPCs",
                    npc_id, room_id, before_count, after_count);
            } else {
                tracing::error!("[PKD] Room {} not found in world!", room_id);
            }
        } else {
            tracing::info!("NPC was not killed (survived the battle)");
        }
    }

    /// 玩家逃跑
    pub async fn escape(&self, player_id: &str) -> Result<String, String> {
        // 获取战斗数据
        let (battle, dodge_a, dodge_d) = {
            let player_battles = self.player_battles.read().await;
            if let Some(battle_id) = player_battles.get(player_id) {
                let battles = self.battles.read().await;
                if let Some(battle) = battles.get(battle_id) {
                    (battle.clone(), battle.challenger.dodge, battle.defender.dodge)
                } else {
                    return Err("你不在战斗中！".to_string());
                }
            } else {
                return Err("你不在战斗中！".to_string());
            }
        };

        // 检查逃跑成功率
        let success_rate = dodge_a as f64 / (dodge_a + dodge_d) as f64;

        if rand::random::<f64>() < success_rate * 0.8 {
            // 逃跑成功 - 移除战斗
            self.end_battle(&battle.battle_id).await;
            Ok("§Y你成功逃脱了！§N".to_string())
        } else {
            Err("§R你逃跑失败了！§N".to_string())
        }
    }

    /// 投降
    pub async fn surrender(&self, player_id: &str) -> Result<String, String> {
        // 获取战斗数据
        let (battle, is_challenger) = {
            let player_battles = self.player_battles.read().await;
            if let Some(battle_id) = player_battles.get(player_id) {
                let battles = self.battles.read().await;
                if let Some(battle) = battles.get(battle_id) {
                    (battle.clone(), battle.challenger.id == player_id)
                } else {
                    return Err("你不在战斗中！".to_string());
                }
            } else {
                return Err("你不在战斗中！".to_string());
            }
        };

        if is_challenger {
            // 移除战斗
            self.end_battle(&battle.battle_id).await;
            Ok("§Y你投降了！§N".to_string())
        } else {
            Err("只有发起者可以投降！".to_string())
        }
    }

    /// 玩家选择技能
    pub async fn select_skill(&self, player_id: &str, skill_id: &str) -> Result<String, String> {
        // 先获取 battle_id（只需要读锁）
        let battle_id = {
            let player_battles = self.player_battles.read().await;
            player_battles.get(player_id)
                .ok_or("你不在战斗中！".to_string())?
                .clone()
        };

        // 然后获取 battles 的写锁来修改战斗状态
        let mut battles = self.battles.write().await;
        if let Some(battle) = battles.get_mut(&battle_id) {
            battle.select_skill(player_id, skill_id)?;
            Ok(format!("§Y你选择了 {}§N\n[继续战斗:pk continue]", skill_id))
        } else {
            Err("战斗不存在！".to_string())
        }
    }

    /// 获取玩家战斗状态（包含技能）
    pub async fn get_player_battle_status(&self, player_id: &str) -> Option<String> {
        if let Some(battle) = self.get_player_battle(player_id).await {
            Some(battle.generate_status_for_player(player_id))
        } else {
            None
        }
    }

    /// 获取玩家技能列表
    pub async fn get_player_skills_list(&self, player_id: &str) -> Option<String> {
        if let Some(battle) = self.get_player_battle(player_id).await {
            Some(battle.generate_skills_list(player_id))
        } else {
            None
        }
    }

    /// 处理心跳 - 自动执行包含NPC的战斗回合
    /// 返回执行的战斗数量和需要通知的玩家
    pub async fn process_heartbeat(&self) -> (usize, Vec<(String, String)>) {
        let mut battles = self.battles.write().await;
        let mut executed_count = 0;
        let mut notifications = Vec::new();

        // 收集需要处理的战斗ID（避免在迭代时修改）
        let battles_to_process: Vec<String> = battles.values()
            .filter(|b| b.should_auto_execute())
            .map(|b| b.battle_id.clone())
            .collect();

        for battle_id in battles_to_process {
            if let Some(battle) = battles.get_mut(&battle_id) {
                // NPC自动选择技能
                if let Some(npc_id) = battle.get_npc_id() {
                    let npc_skill = battle.npc_select_skill(&npc_id);
                    battle.pending_skills.insert(npc_id, npc_skill);
                }

                // 执行回合
                if battle.status == CombatStatus::Fighting {
                    let round = battle.execute_round();

                    // 收集需要通知的玩家
                    if !PkBattle::is_npc(&battle.challenger.id) {
                        notifications.push((battle.challenger.id.clone(), battle_id.clone()));
                    }
                    if !PkBattle::is_npc(&battle.defender.id) {
                        notifications.push((battle.defender.id.clone(), battle_id.clone()));
                    }

                    // 检查战斗是否结束
                    if round.ended {
                        battle.status = CombatStatus::Dead;
                        // 战斗结束，将在下次查询时清理
                    }

                    executed_count += 1;
                }
            }
        }

        (executed_count, notifications)
    }

    /// 启动心跳任务
    pub async fn start_heartbeat_task(daemon: Arc<PkDaemon>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            interval.tick().await;

            let (executed, notifications) = daemon.process_heartbeat().await;

            if executed > 0 {
                tracing::debug!("[PKD Heartbeat] Executed {} battles", executed);
            }

            // 通知在线玩家刷新战斗界面
            for (player_id, _battle_id) in notifications {
                // 这里可以通过HTTP API通知前端刷新
                // 目前先记录日志
                tracing::debug!("[PKD Heartbeat] Notify player {} to refresh", player_id);
            }
        }
    }
}

impl Default for PkDaemon {
    fn default() -> Self {
        Self::new()
    }
}

// 全局PK守护进程
lazy_static::lazy_static! {
    pub static ref PKD: Arc<PkDaemon> = Arc::new(PkDaemon::new());
}

/// 便捷函数
pub async fn get_pkd() -> Arc<PkDaemon> {
    PKD.clone()
}
