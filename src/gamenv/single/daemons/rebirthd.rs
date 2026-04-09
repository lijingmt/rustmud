// gamenv/single/daemons/rebirthd.rs - 转生系统守护进程
// 对应 txpike9/gamenv/single/daemons/rebirthd.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 转生状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RebirthStatus {
    /// 可转生
    Available,
    /// 转生中
    InProgress,
    /// 冷却中
    Cooldown,
}

/// 转生记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RebirthRecord {
    /// 转生次数
    pub count: i32,
    /// 总获得属性点
    pub total_bonus_points: i32,
    /// 转生时间
    pub rebirth_time: i64,
}

/// 转生配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RebirthConfig {
    /// 等级要求
    pub level_requirement: i32,
    /// 转生冷却时间（小时）
    pub cooldown_hours: i64,
    /// 每次转生获得的属性点
    pub bonus_points_per_rebirth: i32,
    /// 属性点上限
    pub max_bonus_points: i32,
    /// 是否保留技能
    pub keep_skills: bool,
    /// 是否保留装备
    pub keep_equipment: bool,
    /// 金币保留比例 (0-100)
    pub gold_retention_rate: i32,
}

impl Default for RebirthConfig {
    fn default() -> Self {
        Self {
            level_requirement: 100,
            cooldown_hours: 0,
            bonus_points_per_rebirth: 10,
            max_bonus_points: 5000,
            keep_skills: true,
            keep_equipment: true,
            gold_retention_rate: 50,
        }
    }
}

/// 转生信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RebirthInfo {
    /// 玩家ID
    pub player_id: String,
    /// 转生次数
    pub rebirth_count: i32,
    /// 获得的额外属性点
    pub bonus_points: i32,
    /// 已分配的属性点
    pub allocated_points: i32,
    /// 上次转生时间
    pub last_rebirth_time: Option<i64>,
    /// 转生状态
    pub status: RebirthStatus,
}

impl RebirthInfo {
    pub fn new(player_id: String) -> Self {
        Self {
            player_id,
            rebirth_count: 0,
            bonus_points: 0,
            allocated_points: 0,
            last_rebirth_time: None,
            status: RebirthStatus::Available,
        }
    }

    /// 可用属性点
    pub fn available_points(&self) -> i32 {
        self.bonus_points - self.allocated_points
    }

    /// 是否可以转生
    pub fn can_rebirth(&self, config: &RebirthConfig, current_level: i32) -> bool {
        if current_level < config.level_requirement {
            return false;
        }

        match self.status {
            RebirthStatus::Available => true,
            RebirthStatus::Cooldown => {
                if let Some(last_time) = self.last_rebirth_time {
                    let cooldown_end = last_time + (config.cooldown_hours * 3600);
                    chrono::Utc::now().timestamp() >= cooldown_end
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// 分配属性点
    pub fn allocate_points(&mut self, points: i32) -> Result<()> {
        if points > self.available_points() {
            return Err(MudError::RuntimeError("可用点数不足".to_string()));
        }
        self.allocated_points += points;
        Ok(())
    }
}

/// 转生守护进程
pub struct RebirthDaemon {
    /// 玩家转生信息
    player_rebirths: HashMap<String, RebirthInfo>,
    /// 转生记录
    records: Vec<RebirthRecord>,
    /// 配置
    config: RebirthConfig,
}

impl RebirthDaemon {
    /// 创建新的转生守护进程
    pub fn new() -> Self {
        Self {
            player_rebirths: HashMap::new(),
            records: Vec::new(),
            config: RebirthConfig::default(),
        }
    }

    /// 获取或创建玩家转生信息
    fn get_or_create_info(&mut self, player_id: &str) -> &mut RebirthInfo {
        self.player_rebirths
            .entry(player_id.to_string())
            .or_insert_with(|| RebirthInfo::new(player_id.to_string()))
    }

    /// 获取玩家转生信息
    pub fn get_rebirth_info(&self, player_id: &str) -> Option<&RebirthInfo> {
        self.player_rebirths.get(player_id)
    }

    /// 执行转生
    pub fn rebirth(&mut self, player_id: &str, player_name: String, current_level: i32) -> Result<RebirthResult> {
        // 先检查条件（不可变借用）
        let can_rebirth = if let Some(info) = self.player_rebirths.get(player_id) {
            info.can_rebirth(&self.config, current_level)
        } else {
            true // 新玩家总是可以转生
        };

        if !can_rebirth {
            return Err(MudError::RuntimeError("不满足转生条件".to_string()));
        }

        let bonus_points = self.config.bonus_points_per_rebirth;

        // 执行转生（可变借用）
        let info = self.get_or_create_info(player_id);
        let new_count = info.rebirth_count + 1;
        let total_points = info.bonus_points + bonus_points;

        info.rebirth_count = new_count;
        info.bonus_points = total_points;
        info.last_rebirth_time = Some(chrono::Utc::now().timestamp());
        info.status = RebirthStatus::Available;

        // 记录转生
        let record = RebirthRecord {
            count: new_count,
            total_bonus_points: total_points,
            rebirth_time: chrono::Utc::now().timestamp(),
        };
        self.records.push(record);

        Ok(RebirthResult {
            player_id: player_id.to_string(),
            player_name,
            rebirth_count: new_count,
            bonus_points_gained: bonus_points,
            total_bonus_points: total_points,
        })
    }

    /// 分配属性点
    pub fn allocate_points(&mut self, player_id: &str, points: i32) -> Result<()> {
        if let Some(info) = self.player_rebirths.get_mut(player_id) {
            info.allocate_points(points)
        } else {
            Err(MudError::NotFound("转生信息不存在".to_string()))
        }
    }

    /// 获取转生次数
    pub fn get_rebirth_count(&self, player_id: &str) -> i32 {
        self.player_rebirths
            .get(player_id)
            .map_or(0, |info| info.rebirth_count)
    }

    /// 获取属性点加成
    pub fn get_stat_bonus(&self, player_id: &str) -> i32 {
        self.player_rebirths
            .get(player_id)
            .map_or(0, |info| info.allocated_points)
    }

    /// 格式化转生信息
    pub fn format_rebirth_info(&self, player_id: &str) -> String {
        if let Some(info) = self.get_rebirth_info(player_id) {
            format!(
                "§H=== 转生系统 ===§N\n\
                 转生次数: {}\n\
                 获得属性点: {}\n\
                 已分配: {}\n\
                 可分配: {}\n\
                 等级要求: {}级\n\
                 每次转生: +{}点",
                info.rebirth_count,
                info.bonus_points,
                info.allocated_points,
                info.available_points(),
                self.config.level_requirement,
                self.config.bonus_points_per_rebirth
            )
        } else {
            "§H=== 转生系统 ===§N\n你还没有转过生。".to_string()
        }
    }

    /// 获取排行榜
    pub fn get_leaderboard(&self) -> Vec<(&str, i32)> {
        let mut entries: Vec<_> = self.player_rebirths
            .iter()
            .map(|(id, info)| (id.as_str(), info.rebirth_count))
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.into_iter().take(100).collect()
    }

    /// 格式化排行榜
    pub fn format_leaderboard(&self) -> String {
        let leaderboard = self.get_leaderboard();

        let mut output = format!("§H=== 转生排行榜 (前10名) ===§N\n");

        if leaderboard.is_empty() {
            output.push_str("暂无转生记录。\n");
        } else {
            for (i, (player_id, count)) in leaderboard.iter().take(10).enumerate() {
                output.push_str(&format!("  {}. {} - {}转\n", i + 1, player_id, count));
            }
        }

        output
    }
}

impl Default for RebirthDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 转生结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RebirthResult {
    pub player_id: String,
    pub player_name: String,
    pub rebirth_count: i32,
    pub bonus_points_gained: i32,
    pub total_bonus_points: i32,
}

/// 全局转生守护进程
pub static REBIRTHD: std::sync::OnceLock<RwLock<RebirthDaemon>> = std::sync::OnceLock::new();

/// 获取转生守护进程
pub fn get_rebirthd() -> &'static RwLock<RebirthDaemon> {
    REBIRTHD.get_or_init(|| RwLock::new(RebirthDaemon::default()))
}
