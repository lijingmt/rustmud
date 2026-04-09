// gamenv/single/daemons/achieved.rs - 成就系统守护进程
// 对应 txpike9/gamenv/single/daemons/achieved.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 成就类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AchievementType {
    /// 等级成就
    Level,
    /// 战斗成就
    Combat,
    /// 探索成就
    Exploration,
    /// 社交成就
    Social,
    /// 收集成就
    Collection,
    /// 经济成就
    Economic,
    /// 副本成就
    Dungeon,
    /// PK成就
    Pk,
    /// 特殊成就
    Special,
}

/// 成就难度
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AchievementDifficulty {
    /// 普通
    Common,
    /// 困难
    Uncommon,
    /// 稀有
    Rare,
    /// 史诗
    Epic,
    /// 传说
    Legendary,
}

impl PartialEq for AchievementDifficulty {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl PartialOrd for AchievementDifficulty {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            AchievementDifficulty::Common => 0,
            AchievementDifficulty::Uncommon => 1,
            AchievementDifficulty::Rare => 2,
            AchievementDifficulty::Epic => 3,
            AchievementDifficulty::Legendary => 4,
        };
        let other_val = match other {
            AchievementDifficulty::Common => 0,
            AchievementDifficulty::Uncommon => 1,
            AchievementDifficulty::Rare => 2,
            AchievementDifficulty::Epic => 3,
            AchievementDifficulty::Legendary => 4,
        };
        Some(self_val.cmp(&other_val))
    }
}

/// 成就条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementCondition {
    /// 条件类型
    pub condition_type: String,
    /// 目标值
    pub target_value: i32,
    /// 当前值（玩家）
    pub current_value: i32,
}

/// 成就奖励
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementReward {
    /// 经验奖励
    pub exp: u64,
    /// 金币奖励
    pub gold: u64,
    /// 物品奖励
    pub items: Vec<String>,
    /// 称号奖励
    pub title: Option<String>,
}

/// 成就
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Achievement {
    /// 成就ID
    pub id: String,
    /// 成就名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 成就类型
    pub achievement_type: AchievementType,
    /// 难度
    pub difficulty: AchievementDifficulty,
    /// 前置成就
    pub prerequisite: Option<String>,
    /// 条件
    pub conditions: Vec<AchievementCondition>,
    /// 奖励
    pub reward: AchievementReward,
    /// 点数
    pub points: i32,
    /// 隐藏成就
    pub hidden: bool,
}

/// 玩家成就进度
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerAchievement {
    /// 成就ID
    pub achievement_id: String,
    /// 是否已完成
    pub completed: bool,
    /// 完成时间
    pub completed_at: Option<i64>,
    /// 条件进度
    pub progress: HashMap<String, i32>,
}

/// 成就统计
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementStats {
    /// 总完成数
    pub total_completed: i32,
    /// 总点数
    pub total_points: i32,
    /// 各类型完成数
    pub by_type: HashMap<String, i32>,
}

/// 成就守护进程
pub struct AchievementDaemon {
    /// 所有成就
    achievements: HashMap<String, Achievement>,
    /// 玩家成就进度
    player_progress: HashMap<String, HashMap<String, PlayerAchievement>>,
    /// 玩家统计
    player_stats: HashMap<String, AchievementStats>,
}

impl AchievementDaemon {
    /// 创建新的成就守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            achievements: HashMap::new(),
            player_progress: HashMap::new(),
            player_stats: HashMap::new(),
        };

        daemon.init_default_achievements();
        daemon
    }

    /// 初始化默认成就
    fn init_default_achievements(&mut self) {
        // 等级成就
        for (level, points) in [(10, 10), (20, 20), (30, 30), (50, 50), (80, 80), (100, 100)] {
            let achievement = Achievement {
                id: format!("level_{}", level),
                name: format!("达到{}级", level),
                description: format!("角色等级达到{}级", level),
                achievement_type: AchievementType::Level,
                difficulty: if level >= 80 {
                    AchievementDifficulty::Epic
                } else if level >= 50 {
                    AchievementDifficulty::Rare
                } else {
                    AchievementDifficulty::Common
                },
                prerequisite: if level > 10 {
                    Some(format!("level_{}", level - 10))
                } else {
                    None
                },
                conditions: vec![
                    AchievementCondition {
                        condition_type: "level".to_string(),
                        target_value: level,
                        current_value: 0,
                    }
                ],
                reward: AchievementReward {
                    exp: level as u64 * 1000,
                    gold: level as u64 * 100,
                    items: vec![],
                    title: if level >= 100 { Some("百级强者".to_string()) } else { None },
                },
                points,
                hidden: false,
            };
            self.achievements.insert(achievement.id.clone(), achievement);
        }

        // 战斗成就
        let monster_killer = Achievement {
            id: "monster_killer_100".to_string(),
            name: "怪物猎人".to_string(),
            description: "击杀100只怪物".to_string(),
            achievement_type: AchievementType::Combat,
            difficulty: AchievementDifficulty::Common,
            prerequisite: None,
            conditions: vec![
                AchievementCondition {
                    condition_type: "kills".to_string(),
                    target_value: 100,
                    current_value: 0,
                }
            ],
            reward: AchievementReward {
                exp: 5000,
                gold: 500,
                items: vec![],
                title: None,
            },
            points: 10,
            hidden: false,
        };
        self.achievements.insert(monster_killer.id.clone(), monster_killer);

        // PK成就
        let pk_master = Achievement {
            id: "pk_winner_10".to_string(),
            name: "PK高手".to_string(),
            description: "赢得10场PK战斗".to_string(),
            achievement_type: AchievementType::Pk,
            difficulty: AchievementDifficulty::Uncommon,
            prerequisite: None,
            conditions: vec![
                AchievementCondition {
                    condition_type: "pk_wins".to_string(),
                    target_value: 10,
                    current_value: 0,
                }
            ],
            reward: AchievementReward {
                exp: 10000,
                gold: 1000,
                items: vec![],
                title: Some("PK高手".to_string()),
            },
            points: 30,
            hidden: false,
        };
        self.achievements.insert(pk_master.id.clone(), pk_master);

        // 收集成就
        let collector = Achievement {
            id: "item_collector_100".to_string(),
            name: "收藏家".to_string(),
            description: "收集100种不同的物品".to_string(),
            achievement_type: AchievementType::Collection,
            difficulty: AchievementDifficulty::Uncommon,
            prerequisite: None,
            conditions: vec![
                AchievementCondition {
                    condition_type: "unique_items".to_string(),
                    target_value: 100,
                    current_value: 0,
                }
            ],
            reward: AchievementReward {
                exp: 8000,
                gold: 800,
                items: vec![],
                title: None,
            },
            points: 20,
            hidden: false,
        };
        self.achievements.insert(collector.id.clone(), collector);

        // 经济成就
        let rich = Achievement {
            id: "gold_100000".to_string(),
            name: "大富翁".to_string(),
            description: "拥有10万金币".to_string(),
            achievement_type: AchievementType::Economic,
            difficulty: AchievementDifficulty::Rare,
            prerequisite: None,
            conditions: vec![
                AchievementCondition {
                    condition_type: "gold_held".to_string(),
                    target_value: 100000,
                    current_value: 0,
                }
            ],
            reward: AchievementReward {
                exp: 20000,
                gold: 5000,
                items: vec![],
                title: Some("大富翁".to_string()),
            },
            points: 50,
            hidden: false,
        };
        self.achievements.insert(rich.id.clone(), rich);
    }

    /// 获取成就
    pub fn get_achievement(&self, achievement_id: &str) -> Option<&Achievement> {
        self.achievements.get(achievement_id)
    }

    /// 获取所有成就
    pub fn get_all_achievements(&self) -> Vec<&Achievement> {
        self.achievements.values().collect()
    }

    /// 获取玩家成就进度
    pub fn get_player_progress(&self, player_id: &str) -> Vec<(&Achievement, &PlayerAchievement)> {
        let mut result = Vec::new();

        if let Some(progress) = self.player_progress.get(player_id) {
            for (achievement_id, player_achievement) in progress {
                if let Some(achievement) = self.get_achievement(achievement_id) {
                    result.push((achievement, player_achievement));
                }
            }
        }

        result
    }

    /// 初始化玩家成就
    pub fn init_player(&mut self, player_id: &str) {
        if !self.player_progress.contains_key(player_id) {
            let mut progress = HashMap::new();
            for achievement_id in self.achievements.keys() {
                progress.insert(achievement_id.clone(), PlayerAchievement {
                    achievement_id: achievement_id.clone(),
                    completed: false,
                    completed_at: None,
                    progress: HashMap::new(),
                });
            }
            self.player_progress.insert(player_id.to_string(), progress);
            self.player_stats.insert(player_id.to_string(), AchievementStats {
                total_completed: 0,
                total_points: 0,
                by_type: HashMap::new(),
            });
        }
    }

    /// 更新进度
    pub fn update_progress(
        &mut self,
        player_id: &str,
        condition_type: &str,
        value: i32,
    ) -> Vec<String> {
        let mut completed_achievements = Vec::new();

        // 第一步：收集需要更新的成就的所有数据
        struct UpdateData {
            achievement_id: String,
            target_value: i32,
            conditions: Vec<(String, i32)>, // (condition_type, target_value)
            points: i32,
            achievement_type: String,
        }

        let mut to_update: Vec<UpdateData> = Vec::new();
        if let Some(progress) = self.player_progress.get(player_id) {
            for (achievement_id, player_achievement) in progress.iter() {
                if player_achievement.completed {
                    continue;
                }
                if let Some(achievement) = self.get_achievement(achievement_id) {
                    for condition in &achievement.conditions {
                        if condition.condition_type == condition_type {
                            let conditions: Vec<_> = achievement.conditions.iter()
                                .map(|c| (c.condition_type.clone(), c.target_value))
                                .collect();
                            to_update.push(UpdateData {
                                achievement_id: achievement_id.clone(),
                                target_value: condition.target_value,
                                conditions,
                                points: achievement.points,
                                achievement_type: format!("{:?}", achievement.achievement_type),
                            });
                            break;
                        }
                    }
                }
            }
        }

        // 第二步：执行更新和检查完成
        if let Some(progress) = self.player_progress.get_mut(player_id) {
            for update_data in to_update {
                if let Some(player_achievement) = progress.get_mut(&update_data.achievement_id) {
                    let current = player_achievement.progress.entry(condition_type.to_string()).or_insert(0);
                    *current = (*current + value).min(update_data.target_value);

                    // 检查是否完成
                    let mut all_done = true;
                    for (cond_type, target_val) in &update_data.conditions {
                        let current_val = *player_achievement.progress.get(cond_type).unwrap_or(&0);
                        if current_val < *target_val {
                            all_done = false;
                            break;
                        }
                    }

                    if all_done {
                        player_achievement.completed = true;
                        player_achievement.completed_at = Some(chrono::Utc::now().timestamp());

                        completed_achievements.push(update_data.achievement_id.clone());

                        if let Some(stats) = self.player_stats.get_mut(player_id) {
                            stats.total_completed += 1;
                            stats.total_points += update_data.points;
                            *stats.by_type.entry(update_data.achievement_type).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        completed_achievements
    }

    /// 检查成就是否完成
    fn check_achievement_completed(&self, achievement: &Achievement, player_achievement: &PlayerAchievement) -> bool {
        for condition in &achievement.conditions {
            let current = *player_achievement.progress.get(&condition.condition_type).unwrap_or(&0);
            if current < condition.target_value {
                return false;
            }
        }
        true
    }

    /// 完成成就（手动完成，如特殊成就）
    pub fn complete_achievement(&mut self, player_id: &str, achievement_id: &str) -> Result<AchievementReward> {
        // 先获取成就数据
        let achievement = self.get_achievement(achievement_id)
            .ok_or_else(|| MudError::NotFound("成就不存在".to_string()))?;

        let reward = achievement.reward.clone();
        let points = achievement.points;

        if let Some(progress) = self.player_progress.get_mut(player_id) {
            if let Some(player_achievement) = progress.get_mut(achievement_id) {
                if player_achievement.completed {
                    return Err(MudError::RuntimeError("成就已完成".to_string()));
                }

                player_achievement.completed = true;
                player_achievement.completed_at = Some(chrono::Utc::now().timestamp());

                // 更新统计
                if let Some(stats) = self.player_stats.get_mut(player_id) {
                    stats.total_completed += 1;
                    stats.total_points += points;
                }

                return Ok(reward);
            }
        }
        Err(MudError::NotFound("成就不存在".to_string()))
    }

    /// 获取玩家统计
    pub fn get_player_stats(&self, player_id: &str) -> Option<&AchievementStats> {
        self.player_stats.get(player_id)
    }

    /// 获取可完成成就
    pub fn get_available_achievements(&self, player_id: &str) -> Vec<&Achievement> {
        let mut result = Vec::new();
        let completed: Vec<_> = if let Some(progress) = self.player_progress.get(player_id) {
            progress.values()
                .filter(|p| p.completed)
                .map(|p| p.achievement_id.clone())
                .collect()
        } else {
            Vec::new()
        };

        for achievement in self.achievements.values() {
            // 跳过已完成的
            if completed.contains(&achievement.id) {
                continue;
            }

            // 跳过隐藏的
            if achievement.hidden {
                continue;
            }

            // 检查前置
            if let Some(ref prereq) = achievement.prerequisite {
                if !completed.contains(prereq) {
                    continue;
                }
            }

            result.push(achievement);
        }

        result.sort_by(|a, b| {
            a.difficulty.partial_cmp(&b.difficulty).unwrap_or(std::cmp::Ordering::Equal)
        });

        result
    }

    /// 格式化成就列表
    pub fn format_achievement_list(&self, player_id: &str) -> String {
        let achievements = self.get_available_achievements(player_id);
        let stats = self.get_player_stats(player_id);
        let total_count = self.achievements.len();

        let mut output = if let Some(stats) = stats {
            format!(
                "§H=== 成就系统 ===§N\n\
                 进度: {}/{} ({:.1}%) | 点数: {}\n",
                stats.total_completed,
                total_count,
                (stats.total_completed as f32 / total_count as f32) * 100.0,
                stats.total_points
            )
        } else {
            format!("§H=== 成就系统 ===§N\n")
        };

        if achievements.is_empty() {
            output.push_str("§Y恭喜！你已完成所有成就！§N\n");
        } else {
            output.push_str("\n§H--- 可完成成就 ---§N\n");
            for achievement in achievements.iter().take(20) {
                let difficulty = match achievement.difficulty {
                    AchievementDifficulty::Common => "§G普通§N",
                    AchievementDifficulty::Uncommon => "§C困难§N",
                    AchievementDifficulty::Rare => "§B稀有§N",
                    AchievementDifficulty::Epic => "§M史诗§N",
                    AchievementDifficulty::Legendary => "§Y传说§N",
                };

                output.push_str(&format!(
                    "  {} [{}] {} - {}点\n",
                    achievement.name,
                    difficulty,
                    achievement.description,
                    achievement.points
                ));
            }
        }

        output
    }

    /// 格式化已完成成就
    pub fn format_completed_achievements(&self, player_id: &str) -> String {
        let completed = self.get_player_progress(player_id)
            .into_iter()
            .filter(|(_, p)| p.completed)
            .collect::<Vec<_>>();

        let mut output = format!("§H=== 已完成成就 ({}) ===§N\n", completed.len());

        for (achievement, player_achievement) in completed {
            let time = player_achievement.completed_at
                .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "未知".to_string());

            output.push_str(&format!(
                "  {} - {}点 [{}]\n",
                achievement.name,
                achievement.points,
                time
            ));
        }

        output
    }
}

impl Default for AchievementDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局成就守护进程
pub static ACHIEVED: std::sync::OnceLock<RwLock<AchievementDaemon>> = std::sync::OnceLock::new();

/// 获取成就守护进程
pub fn get_achieved() -> &'static RwLock<AchievementDaemon> {
    ACHIEVED.get_or_init(|| RwLock::new(AchievementDaemon::default()))
}
