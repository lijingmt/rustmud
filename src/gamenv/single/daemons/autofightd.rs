// gamenv/single/daemons/autofightd.rs - 自动战斗守护进程
// 对应 txpike9/gamenv/single/daemons/autofightd.pike

use crate::core::*;
use crate::gamenv::combat_system::{CombatSession, CombatSystem, CombatRound, CombatStatus};
use crate::gamenv::player_state::PlayerState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

/// 自动战斗状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AutofightStatus {
    /// 未启动
    Stopped,
    /// 准备中
    Preparing,
    /// 战斗中
    Fighting,
    /// 暂停中
    Paused,
    /// 已停止
    Stopping,
}

/// 自动战斗配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutofightConfig {
    /// 是否启用自动战斗
    pub enabled: bool,
    /// 自动使用药水
    pub auto_use_potion: bool,
    /// HP低于多少时使用药水
    pub hp_threshold: u32,
    /// 自动拾取物品
    pub auto_loot: bool,
    /// 战斗间隔（毫秒）
    pub fight_interval: u64,
    /// 最大战斗次数
    pub max_fights: Option<i32>,
    /// 目标怪物类型
    pub target_monster: Option<String>,
}

impl Default for AutofightConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_use_potion: true,
            hp_threshold: 30,
            auto_loot: true,
            fight_interval: 2000,
            max_fights: None,
            target_monster: None,
        }
    }
}

/// 自动战斗会话
#[derive(Clone, Debug)]
pub struct AutofightSession {
    /// 玩家ID
    pub userid: String,
    /// 当前战斗会话
    pub combat_session: Option<CombatSession>,
    /// 状态
    pub status: AutofightStatus,
    /// 配置
    pub config: AutofightConfig,
    /// 已战斗次数
    pub fight_count: i32,
    /// 总伤害
    pub total_damage: u64,
    /// 获得经验
    pub total_exp: u64,
    /// 获得金币
    pub total_gold: u64,
    /// 开始时间
    pub start_time: i64,
    /// 最后战斗时间
    pub last_fight_time: i64,
}

impl AutofightSession {
    /// 创建新的自动战斗会话
    pub fn new(userid: String, config: AutofightConfig) -> Self {
        Self {
            userid,
            combat_session: None,
            status: AutofightStatus::Preparing,
            config,
            fight_count: 0,
            total_damage: 0,
            total_exp: 0,
            total_gold: 0,
            start_time: chrono::Utc::now().timestamp(),
            last_fight_time: 0,
        }
    }

    /// 是否应该继续战斗
    pub fn should_continue(&self) -> bool {
        // 检查状态
        if self.status != AutofightStatus::Fighting {
            return false;
        }

        // 检查最大次数
        if let Some(max) = self.config.max_fights {
            if self.fight_count >= max {
                return false;
            }
        }

        true
    }

    /// 添加战斗统计
    pub fn add_stats(&mut self, damage: u32, exp: u64, gold: u64) {
        self.fight_count += 1;
        self.total_damage += damage as u64;
        self.total_exp += exp;
        self.total_gold += gold;
        self.last_fight_time = chrono::Utc::now().timestamp();
    }

    /// 获取运行时间（秒）
    pub fn get_duration(&self) -> i64 {
        chrono::Utc::now().timestamp() - self.start_time
    }

    /// 格式化统计信息
    pub fn format_stats(&self) -> String {
        let duration = self.get_duration();
        let mins = duration / 60;
        let secs = duration % 60;

        format!(
            "§H=== 自动战斗统计 ===§N\n\
             运行时间: {}分{}秒\n\
             战斗次数: {}\n\
             总伤害: {}\n\
             获得经验: {}\n\
             获得金币: {}\n\
             平均每战: {}伤害",
            mins, secs,
            self.fight_count,
            self.total_damage,
            self.total_exp,
            self.total_gold,
            if self.fight_count > 0 {
                self.total_damage / self.fight_count as u64
            } else {
                0
            }
        )
    }
}

/// 自动战斗守护进程
pub struct AutofightDaemon {
    /// 活跃的自动战斗会话
    sessions: HashMap<String, AutofightSession>,
}

impl AutofightDaemon {
    /// 创建新的自动战斗守护进程
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// 启动自动战斗
    pub fn start_autofight(&mut self, userid: String, config: AutofightConfig) -> Result<()> {
        if self.sessions.contains_key(&userid) {
            return Err(MudError::RuntimeError("自动战斗已在运行".to_string()));
        }

        let mut session = AutofightSession::new(userid.clone(), config);
        session.status = AutofightStatus::Fighting;
        self.sessions.insert(userid, session);

        Ok(())
    }

    /// 停止自动战斗
    pub fn stop_autofight(&mut self, userid: &str) -> Result<AutofightSession> {
        let mut session = self.sessions.remove(userid)
            .ok_or_else(|| MudError::NotFound("没有自动战斗会话".to_string()))?;

        session.status = AutofightStatus::Stopped;
        Ok(session)
    }

    /// 暂停自动战斗
    pub fn pause_autofight(&mut self, userid: &str) -> Result<()> {
        let session = self.sessions.get_mut(userid)
            .ok_or_else(|| MudError::NotFound("没有自动战斗会话".to_string()))?;

        session.status = AutofightStatus::Paused;
        Ok(())
    }

    /// 恢复自动战斗
    pub fn resume_autofight(&mut self, userid: &str) -> Result<()> {
        let session = self.sessions.get_mut(userid)
            .ok_or_else(|| MudError::NotFound("没有自动战斗会话".to_string()))?;

        session.status = AutofightStatus::Fighting;
        Ok(())
    }

    /// 获取会话
    pub fn get_session(&self, userid: &str) -> Option<&AutofightSession> {
        self.sessions.get(userid)
    }

    /// 获取可变会话
    pub fn get_session_mut(&mut self, userid: &str) -> Option<&mut AutofightSession> {
        self.sessions.get_mut(userid)
    }

    /// 执行一回合自动战斗
    pub fn execute_round(&mut self, userid: &str, player: &mut PlayerState, monster: &crate::gamenv::world::Npc) -> Result<CombatRound> {
        let session = self.sessions.get_mut(userid)
            .ok_or_else(|| MudError::NotFound("没有自动战斗会话".to_string()))?;

        // 创建战斗会话
        if session.combat_session.is_none() {
            session.combat_session = Some(CombatSystem::start_combat(monster.clone()));
        }

        // 执行战斗
        if let Some(ref mut combat) = session.combat_session {
            let round = CombatSystem::execute_round(combat, player);

            // 检查战斗是否结束
            if round.ended {
                if let Some(CombatStatus::PlayerWin) = round.winner {
                    // 获取奖励
                    let rewards = combat.get_rewards();

                    // 应用奖励
                    let _ = player.add_exp(rewards.exp);
                    player.add_gold(rewards.gold);

                    // 更新统计
                    session.add_stats(round.player_damage, rewards.exp, rewards.gold);

                    // 重置战斗会话
                    session.combat_session = None;
                }
            }

            // 检查是否需要停止
            if player.is_dead() {
                session.status = AutofightStatus::Stopped;
            } else if !session.should_continue() {
                session.status = AutofightStatus::Stopping;
            }

            return Ok(round);
        }

        Err(MudError::RuntimeError("战斗会话不存在".to_string()))
    }

    /// 检查并处理自动使用药水
    pub fn check_auto_potion(&self, player: &PlayerState) -> bool {
        if !player.is_alive() {
            return false;
        }

        let hp_percent = (player.hp * 100) / player.hp_max;
        hp_percent < 30 // HP低于30%
    }

    /// 获取所有正在自动战斗的玩家
    pub fn get_active_players(&self) -> Vec<String> {
        self.sessions.iter()
            .filter(|(_, s)| s.status == AutofightStatus::Fighting)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 清理完成的会话
    pub fn cleanup_finished(&mut self) -> Vec<String> {
        let mut finished = Vec::new();

        self.sessions.retain(|userid, session| {
            if session.status == AutofightStatus::Stopped {
                finished.push(userid.clone());
                false
            } else {
                true
            }
        });

        finished
    }

    /// 格式化自动战斗状态
    pub fn format_status(&self, userid: &str) -> String {
        if let Some(session) = self.get_session(userid) {
            let status_text = match session.status {
                AutofightStatus::Stopped => "§R已停止§N",
                AutofightStatus::Preparing => "§Y准备中§N",
                AutofightStatus::Fighting => "§G战斗中§N",
                AutofightStatus::Paused => "§C暂停中§N",
                AutofightStatus::Stopping => "§Y停止中§N",
            };

            format!(
                "§H=== 自动战斗状态 ===§N\n\
                 状态: {}\n\
                 战斗次数: {}\n\
                 运行时间: {}秒",
                status_text,
                session.fight_count,
                session.get_duration()
            )
        } else {
            "§R未启动自动战斗§N".to_string()
        }
    }
}

impl Default for AutofightDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局自动战斗守护进程
pub static AUTOFIGHTD: std::sync::OnceLock<TokioRwLock<AutofightDaemon>> = std::sync::OnceLock::new();

/// 获取自动战斗守护进程
pub fn get_autofightd() -> &'static TokioRwLock<AutofightDaemon> {
    AUTOFIGHTD.get_or_init(|| TokioRwLock::new(AutofightDaemon::default()))
}
