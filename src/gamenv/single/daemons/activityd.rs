// gamenv/single/daemons/activityd.rs - 活动系统守护进程
// 对应 txpike9/gamenv/single/daemons/activityd.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 活动类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActivityType {
    /// 日常活动
    Daily,
    /// 周常活动
    Weekly,
    /// 节日活动
    Event,
    /// 限时活动
    Limited,
    /// 在线活动
    Online,
    /// 累计活动
    Cumulative,
}

/// 活动状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActivityStatus {
    /// 未开始
    NotStarted,
    /// 进行中
    Ongoing,
    /// 已结束
    Ended,
    /// 已完成
    Completed,
}

/// 活动条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityCondition {
    /// 条件类型
    pub condition_type: String,
    /// 目标值
    pub target_value: i32,
    /// 描述
    pub description: String,
}

/// 活动奖励
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityReward {
    /// 经验奖励
    pub exp: u64,
    /// 金币奖励
    pub gold: u64,
    /// 物品奖励
    pub items: Vec<String>,
    /// 称号奖励
    pub title: Option<String>,
}

/// 活动进度
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityProgress {
    /// 活动ID
    pub activity_id: String,
    /// 玩家ID
    pub player_id: String,
    /// 当前进度值
    pub current_value: i32,
    /// 是否已领取奖励
    pub claimed: bool,
    /// 更新时间
    pub updated_at: i64,
}

impl ActivityProgress {
    pub fn new(activity_id: String, player_id: String) -> Self {
        Self {
            activity_id,
            player_id,
            current_value: 0,
            claimed: false,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    /// 是否完成
    pub fn is_completed(&self, target_value: i32) -> bool {
        self.current_value >= target_value
    }

    /// 是否可领取
    pub fn can_claim(&self, target_value: i32) -> bool {
        self.is_completed(target_value) && !self.claimed
    }
}

/// 活动
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    /// 活动ID
    pub id: String,
    /// 活动名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 活动类型
    pub activity_type: ActivityType,
    /// 活动状态
    pub status: ActivityStatus,
    /// 开始时间
    pub start_time: i64,
    /// 结束时间
    pub end_time: i64,
    /// 条件
    pub conditions: Vec<ActivityCondition>,
    /// 奖励
    pub rewards: ActivityReward,
    /// VIP等级要求
    pub vip_requirement: i32,
    /// 等级要求
    pub level_requirement: i32,
    /// 显示顺序
    pub sort_order: i32,
}

impl Activity {
    /// 是否可用
    pub fn is_available(&self) -> bool {
        if self.status != ActivityStatus::Ongoing {
            return false;
        }

        let now = chrono::Utc::now().timestamp();
        if now < self.start_time || now > self.end_time {
            return false;
        }

        true
    }

    /// 是否已过期
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.end_time
    }

    /// 格式化活动信息
    pub fn format_info(&self, progress: Option<&ActivityProgress>) -> String {
        let status_text = match self.status {
            ActivityStatus::NotStarted => "§C未开始§N",
            ActivityStatus::Ongoing => "§G进行中§N",
            ActivityStatus::Ended => "§R已结束§N",
            ActivityStatus::Completed => "§Y已完成§N",
        };

        let mut output = format!(
            "§H[{}]§N {} - {}\n\
             状态: {}",
            if self.activity_type == ActivityType::Daily { "日" }
             else if self.activity_type == ActivityType::Weekly { "周" }
             else { "特" },
            self.name,
            self.description,
            status_text
        );

        if let Some(prog) = progress {
            if !prog.claimed {
                if let Some(cond) = self.conditions.first() {
                    let completed = prog.is_completed(cond.target_value);
                    output.push_str(&format!(
                        "\n进度: {}/{} {}",
                        prog.current_value,
                        cond.target_value,
                        if completed { "§G[已完成]§N" } else { "" }
                    ));
                }
            } else {
                output.push_str(" §Y[已领取]§N");
            }
        }

        output
    }
}

/// 活动守护进程
pub struct ActivityDaemon {
    /// 所有活动
    activities: HashMap<String, Activity>,
    /// 玩家进度
    player_progress: HashMap<String, HashMap<String, ActivityProgress>>,
}

impl ActivityDaemon {
    /// 创建新的活动守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            activities: HashMap::new(),
            player_progress: HashMap::new(),
        };

        daemon.init_default_activities();
        daemon
    }

    /// 初始化默认活动
    fn init_default_activities(&mut self) {
        let now = chrono::Utc::now().timestamp();
        let day_end = now + (24 * 60 * 60);

        // 日常签到
        let daily_checkin = Activity {
            id: "daily_checkin".to_string(),
            name: "每日签到".to_string(),
            description: "每天登录即可获得奖励".to_string(),
            activity_type: ActivityType::Daily,
            status: ActivityStatus::Ongoing,
            start_time: now,
            end_time: day_end,
            conditions: vec![
                ActivityCondition {
                    condition_type: "login".to_string(),
                    target_value: 1,
                    description: "登录游戏".to_string(),
                }
            ],
            rewards: ActivityReward {
                exp: 1000,
                gold: 500,
                items: vec![],
                title: None,
            },
            vip_requirement: 0,
            level_requirement: 1,
            sort_order: 1,
        };

        // 日常刷怪
        let daily_monster = Activity {
            id: "daily_monster_100".to_string(),
            name: "猎魔先锋".to_string(),
            description: "击杀100只怪物".to_string(),
            activity_type: ActivityType::Daily,
            status: ActivityStatus::Ongoing,
            start_time: now,
            end_time: day_end,
            conditions: vec![
                ActivityCondition {
                    condition_type: "kill_monster".to_string(),
                    target_value: 100,
                    description: "击杀怪物".to_string(),
                }
            ],
            rewards: ActivityReward {
                exp: 5000,
                gold: 2000,
                items: vec![],
                title: None,
            },
            vip_requirement: 0,
            level_requirement: 1,
            sort_order: 2,
        };

        // 日常副本
        let daily_dungeon = Activity {
            id: "daily_dungeon_5".to_string(),
            name: "副本挑战者".to_string(),
            description: "完成5次副本".to_string(),
            activity_type: ActivityType::Daily,
            status: ActivityStatus::Ongoing,
            start_time: now,
            end_time: day_end,
            conditions: vec![
                ActivityCondition {
                    condition_type: "complete_dungeon".to_string(),
                    target_value: 5,
                    description: "完成副本".to_string(),
                }
            ],
            rewards: ActivityReward {
                exp: 10000,
                gold: 5000,
                items: vec![],
                title: None,
            },
            vip_requirement: 0,
            level_requirement: 10,
            sort_order: 3,
        };

        self.activities.insert(daily_checkin.id.clone(), daily_checkin);
        self.activities.insert(daily_monster.id.clone(), daily_monster);
        self.activities.insert(daily_dungeon.id.clone(), daily_dungeon);
    }

    /// 获取活动
    pub fn get_activity(&self, activity_id: &str) -> Option<&Activity> {
        self.activities.get(activity_id)
    }

    /// 获取所有活动
    pub fn get_all_activities(&self) -> Vec<&Activity> {
        self.activities.values().collect()
    }

    /// 获取可用活动
    pub fn get_available_activities(&self, player_level: i32, player_vip: i32) -> Vec<&Activity> {
        self.activities.values()
            .filter(|activity| {
                activity.is_available()
                    && activity.level_requirement <= player_level
                    && activity.vip_requirement <= player_vip
            })
            .collect()
    }

    /// 获取玩家进度
    pub fn get_player_progress(&self, player_id: &str, activity_id: &str) -> Option<&ActivityProgress> {
        self.player_progress.get(player_id)
            .and_then(|p| p.get(activity_id))
    }

    /// 更新进度
    pub fn update_progress(
        &mut self,
        player_id: &str,
        condition_type: &str,
        value: i32,
    ) -> Vec<String> {
        let mut completed_activities = Vec::new();

        // 找到匹配的活动
        let matching_activities: Vec<_> = self.activities.values()
            .filter(|activity| {
                activity.is_available()
                    && activity.conditions.iter().any(|c| c.condition_type == condition_type)
            })
            .map(|a| (a.id.clone(), a.conditions.clone()))
            .collect();

        // 更新进度
        for (activity_id, conditions) in matching_activities {
            let progress = self.player_progress
                .entry(player_id.to_string())
                .or_insert_with(HashMap::new)
                .entry(activity_id.clone())
                .or_insert_with(|| ActivityProgress::new(activity_id.clone(), player_id.to_string()));

            if !progress.claimed {
                progress.current_value += value;
                progress.updated_at = chrono::Utc::now().timestamp();

                // 检查是否完成
                for condition in &conditions {
                    if condition.condition_type == condition_type {
                        if progress.current_value >= condition.target_value {
                            completed_activities.push(activity_id);
                        }
                        break;
                    }
                }
            }
        }

        completed_activities
    }

    /// 领取奖励
    pub fn claim_reward(&mut self, player_id: &str, activity_id: &str) -> Result<ActivityReward> {
        let activity = self.activities.get(activity_id)
            .ok_or_else(|| MudError::NotFound("活动不存在".to_string()))?;

        let progress = self.player_progress.get_mut(player_id)
            .and_then(|p| p.get_mut(activity_id))
            .ok_or_else(|| MudError::NotFound("活动进度不存在".to_string()))?;

        if progress.claimed {
            return Err(MudError::RuntimeError("奖励已领取".to_string()));
        }

        let target_value = activity.conditions.first()
            .map(|c| c.target_value)
            .unwrap_or(0);

        if progress.current_value < target_value {
            return Err(MudError::RuntimeError("活动未完成".to_string()));
        }

        progress.claimed = true;
        Ok(activity.rewards.clone())
    }

    /// 格式化活动列表
    pub fn format_activity_list(&self, player_id: &str, player_level: i32, player_vip: i32) -> String {
        let activities = self.get_available_activities(player_level, player_vip);

        let mut output = format!("§H=== 活动列表 ({}个) ===§N\n", activities.len());

        if activities.is_empty() {
            output.push_str("暂无可用活动。\n");
        } else {
            for activity in activities {
                let progress = self.get_player_progress(player_id, &activity.id);
                output.push_str(&format!("  {}\n", activity.format_info(progress)));
            }
        }

        output
    }

    /// 重置日常活动
    pub fn reset_daily_activities(&mut self, player_id: &str) {
        if let Some(progress) = self.player_progress.get_mut(player_id) {
            progress.retain(|_, p| {
                let activity = self.activities.get(&p.activity_id);
                activity.map_or(false, |a| a.activity_type != ActivityType::Daily)
            });
        }
    }

    /// 清理过期活动
    pub fn cleanup_expired_activities(&mut self) {
        let expired: Vec<_> = self.activities.values()
            .filter(|a| a.is_expired() && a.activity_type != ActivityType::Daily)
            .map(|a| a.id.clone())
            .collect();

        for id in expired {
            if let Some(activity) = self.activities.get_mut(&id) {
                activity.status = ActivityStatus::Ended;
            }
        }
    }
}

impl Default for ActivityDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局活动守护进程
pub static ACTIVITYD: std::sync::OnceLock<RwLock<ActivityDaemon>> = std::sync::OnceLock::new();

/// 获取活动守护进程
pub fn get_activityd() -> &'static RwLock<ActivityDaemon> {
    ACTIVITYD.get_or_init(|| RwLock::new(ActivityDaemon::default()))
}
