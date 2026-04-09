// gamenv/single/daemons/pkd.rs - PK系统守护进程
// 对应 txpike9/gamenv/single/daemons/pkd.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

/// PK模式
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum PkMode {
    /// 和平模式 - 不能攻击其他玩家
    Peace,
    /// 自由模式 - 可以攻击任何人
    Free,
    /// 组队模式 - 只能攻击敌对帮派
    Team,
    /// 帮派模式 - 只能攻击敌对帮派
    Guild,
}

/// PK状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PkStatus {
    /// 正常
    Normal,
    /// 战斗中
    Fighting,
    /// 逃跑
    Escaped,
    /// 死亡
    Dead,
}

/// PK记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PkRecord {
    /// 战斗ID
    pub battle_id: String,
    /// 挑战者ID
    pub challenger_id: String,
    /// 挑战者名称
    pub challenger_name: String,
    /// 应战者ID
    pub defender_id: String,
    /// 应战者名称
    pub defender_name: String,
    /// 胜者ID
    pub winner: Option<String>,
    /// 开始时间
    pub start_time: i64,
    /// 结束时间
    pub end_time: Option<i64>,
    /// 战斗回合数
    pub rounds: i32,
    /// 挑战者造成的伤害
    pub challenger_damage: u64,
    /// 应战者造成的伤害
    pub defender_damage: u64,
}

/// PK回合结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PkRound {
    /// 回合数
    pub round: i32,
    /// 挑战者伤害
    pub challenger_damage: u32,
    /// 应战者伤害
    pub defender_damage: u32,
    /// 挑战者当前HP
    pub challenger_hp: u32,
    /// 应战者当前HP
    pub defender_hp: u32,
    /// 战斗是否结束
    pub ended: bool,
    /// 胜者
    pub winner: Option<String>,
    /// 战斗日志
    pub log: Vec<String>,
}

/// PK通缉令
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WantedPoster {
    /// 玩家ID
    pub player_id: String,
    /// 玩家名称
    pub player_name: String,
    /// 通缉原因
    pub reason: String,
    /// 发布者ID
    pub issuer_id: String,
    /// 赏金
    pub bounty: u64,
    /// 发布时间
    pub issued_at: i64,
    /// 击杀者ID
    pub killer_id: Option<String>,
    /// 完成时间
    pub completed_at: Option<i64>,
}

impl WantedPoster {
    /// 是否已完成
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// 是否已过期（7天）
    pub fn is_expired(&self) -> bool {
        let expiry = self.issued_at + (7 * 24 * 60 * 60);
        chrono::Utc::now().timestamp() > expiry && !self.is_completed()
    }

    /// 格式化通缉令
    pub fn format(&self) -> String {
        let status = if self.is_completed() {
            "§X已完成§N"
        } else if self.is_expired() {
            "§R已过期§N"
        } else {
            "§G悬赏中§N"
        };

        format!(
            "§H[通缉令]§N {} - {}\n\
             原因: {}\n\
             赏金: §Y{}金币§N\n\
             状态: {}",
            self.player_name, status, self.reason, self.bounty, status
        )
    }
}

/// PK战斗会话
#[derive(Clone, Debug)]
pub struct PkBattle {
    /// 战斗ID
    pub battle_id: String,
    /// 挑战者ID
    pub challenger_id: String,
    /// 挑战者名称
    pub challenger_name: String,
    /// 应战者ID
    pub defender_id: String,
    /// 应战者名称
    pub defender_name: String,
    /// 挑战者HP
    pub challenger_hp: u32,
    /// 挑战者最大HP
    pub challenger_hp_max: u32,
    /// 挑战者攻击力
    pub challenger_attack: u32,
    /// 挑战者防御力
    pub challenger_defense: u32,
    /// 应战者HP
    pub defender_hp: u32,
    /// 应战者最大HP
    pub defender_hp_max: u32,
    /// 应战者攻击力
    pub defender_attack: u32,
    /// 应战者防御力
    pub defender_defense: u32,
    /// 开始时间
    pub start_time: i64,
    /// 回合数
    pub rounds: i32,
    /// 挑战者总伤害
    pub challenger_damage: u64,
    /// 应战者总伤害
    pub defender_damage: u64,
    /// 状态
    pub status: PkStatus,
}

impl PkBattle {
    /// 创建新的PK战斗
    pub fn new(
        battle_id: String,
        challenger_id: String,
        challenger_name: String,
        defender_id: String,
        defender_name: String,
        challenger_hp: u32,
        challenger_hp_max: u32,
        challenger_attack: u32,
        challenger_defense: u32,
        defender_hp: u32,
        defender_hp_max: u32,
        defender_attack: u32,
        defender_defense: u32,
    ) -> Self {
        Self {
            battle_id,
            challenger_id,
            challenger_name,
            defender_id,
            defender_name,
            challenger_hp,
            challenger_hp_max,
            challenger_attack,
            challenger_defense,
            defender_hp,
            defender_hp_max,
            defender_attack,
            defender_defense,
            start_time: chrono::Utc::now().timestamp(),
            rounds: 0,
            challenger_damage: 0,
            defender_damage: 0,
            status: PkStatus::Fighting,
        }
    }

    /// 是否战斗中
    pub fn is_fighting(&self) -> bool {
        self.status == PkStatus::Fighting
    }

    /// 是否已结束
    pub fn is_ended(&self) -> bool {
        matches!(self.status, PkStatus::Dead | PkStatus::Escaped)
    }

    /// 获取对手ID
    pub fn get_opponent(&self, player_id: &str) -> Option<&str> {
        if player_id == self.challenger_id {
            Some(&self.defender_id)
        } else if player_id == self.defender_id {
            Some(&self.challenger_id)
        } else {
            None
        }
    }

    /// 计算伤害
    fn calculate_damage(attacker_attack: u32, defender_defense: u32) -> u32 {
        let base_damage = attacker_attack as f32;
        let defense = defender_defense as f32;
        let reduction = defense / (defense + 100.0);
        let damage = base_damage * (1.0 - reduction);

        // 添加随机浮动 +/- 10%
        let random_factor = 0.9 + (rand::random::<f32>() * 0.2);
        let final_damage = (damage * random_factor) as u32;

        final_damage.max(1)
    }

    /// 执行一回合
    pub fn execute_round(&mut self, attacker_is_challenger: bool) -> PkRound {
        self.rounds += 1;

        let mut log = Vec::new();
        let mut challenger_damage = 0;
        let mut defender_damage = 0;

        if attacker_is_challenger {
            // 挑战者攻击
            challenger_damage = Self::calculate_damage(self.challenger_attack, self.defender_defense);
            if challenger_damage >= self.defender_hp {
                self.defender_hp = 0;
            } else {
                self.defender_hp -= challenger_damage;
            }
            self.challenger_damage += challenger_damage as u64;
            log.push(format!("{}攻击{}，造成{}点伤害！",
                self.challenger_name, self.defender_name, challenger_damage));
        } else {
            // 应战者攻击
            defender_damage = Self::calculate_damage(self.defender_attack, self.challenger_defense);
            if defender_damage >= self.challenger_hp {
                self.challenger_hp = 0;
            } else {
                self.challenger_hp -= defender_damage;
            }
            self.defender_damage += defender_damage as u64;
            log.push(format!("{}攻击{}，造成{}点伤害！",
                self.defender_name, self.challenger_name, defender_damage));
        }

        // 检查是否结束
        let ended = self.challenger_hp == 0 || self.defender_hp == 0;
        let winner = if ended {
            self.status = PkStatus::Dead;
            if self.challenger_hp == 0 {
                Some(self.defender_id.clone())
            } else {
                Some(self.challenger_id.clone())
            }
        } else {
            None
        };

        PkRound {
            round: self.rounds,
            challenger_damage,
            defender_damage,
            challenger_hp: self.challenger_hp,
            defender_hp: self.defender_hp,
            ended,
            winner,
            log,
        }
    }
}

/// PK守护进程
pub struct PkDaemon {
    /// 玩家PK模式
    player_modes: HashMap<String, PkMode>,
    /// 活跃的PK战斗
    active_battles: HashMap<String, PkBattle>,
    /// 玩家到战斗的映射
    player_battles: HashMap<String, String>,
    /// PK记录
    records: Vec<PkRecord>,
    /// 通缉令
    wanted_posters: HashMap<String, WantedPoster>,
    /// PK值（红名值，杀人增加）
    pk_values: HashMap<String, i32>,
    /// 连杀记录
    kill_streaks: HashMap<String, i32>,
    /// 最高连杀
    max_streaks: HashMap<String, i32>,
}

impl PkDaemon {
    /// 创建新的PK守护进程
    pub fn new() -> Self {
        Self {
            player_modes: HashMap::new(),
            active_battles: HashMap::new(),
            player_battles: HashMap::new(),
            records: Vec::new(),
            wanted_posters: HashMap::new(),
            pk_values: HashMap::new(),
            kill_streaks: HashMap::new(),
            max_streaks: HashMap::new(),
        }
    }

    /// 设置玩家PK模式
    pub fn set_pk_mode(&mut self, player_id: String, mode: PkMode) -> Result<()> {
        // 检查是否在战斗中
        if self.player_battles.contains_key(&player_id) {
            return Err(MudError::RuntimeError("战斗中无法切换PK模式".to_string()));
        }

        self.player_modes.insert(player_id, mode);
        Ok(())
    }

    /// 获取玩家PK模式
    pub fn get_pk_mode(&self, player_id: &str) -> PkMode {
        self.player_modes.get(player_id)
            .copied()
            .unwrap_or(PkMode::Peace)
    }

    /// 获取玩家PK值
    pub fn get_pk_value(&self, player_id: &str) -> i32 {
        *self.pk_values.get(player_id).unwrap_or(&0)
    }

    /// 增加PK值
    pub fn add_pk_value(&mut self, player_id: &str, amount: i32) {
        let entry = self.pk_values.entry(player_id.to_string()).or_insert(0);
        *entry = (*entry + amount).max(0);
    }

    /// 获取玩家连杀
    pub fn get_kill_streak(&self, player_id: &str) -> i32 {
        *self.kill_streaks.get(player_id).unwrap_or(&0)
    }

    /// 获取最高连杀
    pub fn get_max_streak(&self, player_id: &str) -> i32 {
        *self.max_streaks.get(player_id).unwrap_or(&0)
    }

    /// 挑战PK
    pub fn challenge_pk(
        &mut self,
        challenger_id: String,
        challenger_name: String,
        defender_id: String,
        defender_name: String,
        challenger_hp: u32,
        challenger_hp_max: u32,
        challenger_attack: u32,
        challenger_defense: u32,
        defender_hp: u32,
        defender_hp_max: u32,
        defender_attack: u32,
        defender_defense: u32,
    ) -> Result<String> {
        // 检查挑战者状态
        if self.player_battles.contains_key(&challenger_id) {
            return Err(MudError::RuntimeError("你已经在战斗中".to_string()));
        }

        // 检查应战者状态
        if self.player_battles.contains_key(&defender_id) {
            return Err(MudError::RuntimeError("对方正在战斗中".to_string()));
        }

        // 检查PK模式
        let defender_mode = self.get_pk_mode(&defender_id);
        if defender_mode == PkMode::Peace {
            return Err(MudError::RuntimeError("对方处于和平模式".to_string()));
        }

        // 创建战斗
        let battle_id = format!("pk_{}_{}",
            chrono::Utc::now().timestamp_nanos(),
            rand::random::<u32>()
        );

        let battle = PkBattle::new(
            battle_id.clone(),
            challenger_id.clone(),
            challenger_name,
            defender_id.clone(),
            defender_name,
            challenger_hp,
            challenger_hp_max,
            challenger_attack,
            challenger_defense,
            defender_hp,
            defender_hp_max,
            defender_attack,
            defender_defense,
        );

        self.active_battles.insert(battle_id.clone(), battle);
        self.player_battles.insert(challenger_id.clone(), battle_id.clone());
        self.player_battles.insert(defender_id.clone(), battle_id.clone());

        Ok(battle_id)
    }

    /// 执行战斗回合
    pub fn execute_round(
        &mut self,
        battle_id: &str,
        attacker_id: &str,
    ) -> Result<PkRound> {
        let battle = self.active_battles.get_mut(battle_id)
            .ok_or_else(|| MudError::NotFound("战斗不存在".to_string()))?;

        if !battle.is_fighting() {
            return Err(MudError::RuntimeError("战斗已结束".to_string()));
        }

        // 确定谁是攻击者
        let is_challenger = attacker_id == battle.challenger_id;
        let round = battle.execute_round(is_challenger);

        // 检查战斗是否结束
        if round.ended {
            if let Some(ref winner) = round.winner {
                self.end_battle(battle_id, Some(winner.clone()));
            } else {
                self.end_battle(battle_id, None);
            }
        }

        Ok(round)
    }

    /// 结束战斗
    fn end_battle(&mut self, battle_id: &str, winner_id: Option<String>) {
        if let Some(battle) = self.active_battles.remove(battle_id) {
            // 清除玩家战斗映射
            self.player_battles.remove(&battle.challenger_id);
            self.player_battles.remove(&battle.defender_id);

            // 记录战斗结果
            let record = PkRecord {
                battle_id: battle.battle_id,
                challenger_id: battle.challenger_id.clone(),
                challenger_name: battle.challenger_name,
                defender_id: battle.defender_id.clone(),
                defender_name: battle.defender_name,
                winner: winner_id.clone(),
                start_time: battle.start_time,
                end_time: Some(chrono::Utc::now().timestamp()),
                rounds: battle.rounds,
                challenger_damage: battle.challenger_damage,
                defender_damage: battle.defender_damage,
            };

            self.records.push(record);

            // 更新连杀
            if let Some(ref winner) = winner_id {
                let streak = self.kill_streaks.entry(winner.clone()).or_insert(0);
                *streak += 1;

                let max_streak = self.max_streaks.entry(winner.clone()).or_insert(0);
                if *streak > *max_streak {
                    *max_streak = *streak;
                }

                // 增加PK值
                self.add_pk_value(winner, 10);

                // 重置败者连杀
                let loser = if winner == &battle.challenger_id {
                    &battle.defender_id
                } else {
                    &battle.challenger_id
                };
                self.kill_streaks.insert(loser.clone(), 0);
            }
        }
    }

    /// 逃跑
    pub fn escape(&mut self, player_id: &str) -> Result<()> {
        let battle_id = self.player_battles.get(player_id)
            .ok_or_else(|| MudError::NotFound("未在战斗中".to_string()))?
            .clone();

        if let Some(battle) = self.active_battles.get_mut(&battle_id) {
            battle.status = PkStatus::Escaped;
        }

        // 结束战斗
        self.end_battle(&battle_id, None);
        Ok(())
    }

    /// 获取战斗
    pub fn get_battle(&self, battle_id: &str) -> Option<&PkBattle> {
        self.active_battles.get(battle_id)
    }

    /// 获取可变战斗
    pub fn get_battle_mut(&mut self, battle_id: &str) -> Option<&mut PkBattle> {
        self.active_battles.get_mut(battle_id)
    }

    /// 获取玩家当前战斗
    pub fn get_player_battle(&self, player_id: &str) -> Option<&PkBattle> {
        if let Some(battle_id) = self.player_battles.get(player_id) {
            self.active_battles.get(battle_id)
        } else {
            None
        }
    }

    /// 发布通缉令
    pub fn issue_wanted(
        &mut self,
        issuer_id: String,
        target_id: String,
        target_name: String,
        reason: String,
        bounty: u64,
    ) -> Result<()> {
        // 检查是否已有通缉令
        if self.wanted_posters.contains_key(&target_id) {
            return Err(MudError::RuntimeError("该玩家已被通缉".to_string()));
        }

        let poster = WantedPoster {
            player_id: target_id.clone(),
            player_name: target_name,
            reason,
            issuer_id,
            bounty,
            issued_at: chrono::Utc::now().timestamp(),
            killer_id: None,
            completed_at: None,
        };

        self.wanted_posters.insert(target_id, poster);
        Ok(())
    }

    /// 完成通缉
    pub fn complete_wanted(&mut self, target_id: &str, killer_id: String) -> Result<u64> {
        let poster = self.wanted_posters.get_mut(target_id)
            .ok_or_else(|| MudError::NotFound("通缉令不存在".to_string()))?;

        if poster.is_completed() {
            return Err(MudError::RuntimeError("通缉令已完成".to_string()));
        }

        poster.killer_id = Some(killer_id.clone());
        poster.completed_at = Some(chrono::Utc::now().timestamp());

        Ok(poster.bounty)
    }

    /// 获取通缉令列表
    pub fn get_wanted_posters(&self) -> Vec<&WantedPoster> {
        self.wanted_posters.values()
            .filter(|p| !p.is_completed() && !p.is_expired())
            .collect()
    }

    /// 获取玩家的通缉令
    pub fn get_player_wanted(&self, player_id: &str) -> Option<&WantedPoster> {
        self.wanted_posters.get(player_id)
    }

    /// 清理过期通缉令
    pub fn cleanup_expired_wanted(&mut self) -> usize {
        let mut to_remove = Vec::new();

        for (id, poster) in &self.wanted_posters {
            if poster.is_expired() {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            self.wanted_posters.remove(&id);
        }

        self.wanted_posters.len()
    }

    /// 获取PK记录
    pub fn get_player_records(&self, player_id: &str) -> Vec<&PkRecord> {
        self.records.iter()
            .filter(|r| r.challenger_id == player_id || r.defender_id == player_id)
            .collect()
    }

    /// 格式化PK模式
    pub fn format_pk_mode(mode: PkMode) -> &'static str {
        match mode {
            PkMode::Peace => "§C和平模式§N",
            PkMode::Free => "§R自由模式§N",
            PkMode::Team => "§B组队模式§N",
            PkMode::Guild => "§Y帮派模式§N",
        }
    }

    /// 格式化PK状态
    pub fn format_pk_status(&self, player_id: &str) -> String {
        let mode = self.get_pk_mode(player_id);
        let pk_value = self.get_pk_value(player_id);
        let streak = self.get_kill_streak(player_id);
        let max_streak = self.get_max_streak(player_id);

        let pk_status = if pk_value >= 100 {
            "§X恶魔§N"
        } else if pk_value >= 50 {
            "§R恶人§N"
        } else if pk_value >= 20 {
            "§Y红名§N"
        } else if pk_value > 0 {
            "§C灰名§N"
        } else {
            "§G良民§N"
        };

        format!(
            "§H=== PK状态 ===§N\n\
             模式: {}\n\
             状态: {} (PK值: {})\n\
             连杀: {} | 最高: {}",
            Self::format_pk_mode(mode),
            pk_status,
            pk_value,
            streak,
            max_streak
        )
    }

    /// 格式化通缉令列表
    pub fn format_wanted_list(&self) -> String {
        let posters = self.get_wanted_posters();
        let mut output = format!("§H=== 通缉令 ({}张) ===§N\n", posters.len());

        if posters.is_empty() {
            output.push_str("暂无通缉令。\n");
        } else {
            for poster in posters {
                output.push_str(&format!("  {}\n", poster.format()));
            }
        }

        output
    }

    /// 是否可以攻击
    pub fn can_attack(&self, attacker_id: &str, defender_id: &str) -> bool {
        let attacker_mode = self.get_pk_mode(attacker_id);
        let defender_mode = self.get_pk_mode(defender_id);

        match attacker_mode {
            PkMode::Peace => false,
            PkMode::Free => defender_mode != PkMode::Peace,
            PkMode::Team => defender_mode == PkMode::Free,
            PkMode::Guild => defender_mode == PkMode::Free,
        }
    }
}

impl Default for PkDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局PK守护进程
pub static PKD: std::sync::OnceLock<TokioRwLock<PkDaemon>> = std::sync::OnceLock::new();

/// 获取PK守护进程
pub fn get_pkd() -> &'static TokioRwLock<PkDaemon> {
    PKD.get_or_init(|| TokioRwLock::new(PkDaemon::default()))
}
