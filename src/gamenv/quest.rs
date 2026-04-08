// gamenv/quest.rs - 任务系统
// 对应 txpike9/gamenv/single/quests/ 目录

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 任务目标类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestObjectiveType {
    /// 击杀怪物
    KillMonster,
    /// 收集物品
    CollectItem,
    /// 与NPC对话
    TalkToNpc,
    /// 到达指定地点
    ReachRoom,
    /// 护送NPC
    EscortNpc,
    /// 使用物品
    UseItem,
}

/// 任务目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    /// 目标类型
    pub objective_type: QuestObjectiveType,
    /// 目标ID (怪物ID、物品ID、NPC ID、房间ID等)
    pub target_id: String,
    /// 目标数量
    pub target_count: u32,
    /// 当前完成数量
    pub current_count: u32,
    /// 是否已完成
    pub completed: bool,
}

impl QuestObjective {
    pub fn new(objective_type: QuestObjectiveType, target_id: String, target_count: u32) -> Self {
        Self {
            objective_type,
            target_id,
            target_count,
            current_count: 0,
            completed: false,
        }
    }

    /// 更新进度
    pub fn update(&mut self, count: u32) -> bool {
        self.current_count = self.current_count.saturating_add(count);
        self.completed = self.current_count >= self.target_count;
        self.completed
    }

    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// 获取进度百分比
    pub fn progress_percent(&self) -> u32 {
        if self.target_count == 0 {
            return 100;
        }
        (self.current_count * 100 / self.target_count).min(100)
    }
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestStatus {
    /// 未接受
    NotStarted,
    /// 进行中
    InProgress,
    /// 已完成（可提交）
    Completed,
    /// 已提交
    TurnedIn,
    /// 已失败
    Failed,
}

/// 任务类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestType {
    /// 主线任务
    Main,
    /// 支线任务
    Side,
    /// 日常任务
    Daily,
    /// 周常任务
    Weekly,
    /// 一次性任务
    OneTime,
    /// 重复任务
    Repeatable,
}

/// 任务奖励
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestReward {
    /// 经验值
    pub exp: u64,
    /// 金币
    pub money: u64,
    /// 物品奖励 (物品ID, 数量)
    pub items: Vec<(String, u32)>,
    /// 潜能
    pub potential: u32,
}

impl Default for QuestReward {
    fn default() -> Self {
        Self {
            exp: 0,
            money: 0,
            items: Vec::new(),
            potential: 0,
        }
    }
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务中文名
    pub name_cn: String,
    /// 任务类型
    pub quest_type: QuestType,
    /// 任务状态
    pub status: QuestStatus,
    /// 任务目标
    pub objectives: Vec<QuestObjective>,
    /// 任务奖励
    pub reward: QuestReward,
    /// 前置任务ID
    pub prerequisite: Option<String>,
    /// 接取等级要求
    pub required_level: u32,
    /// 接取NPC
    pub giver_npc: Option<String>,
    /// 提交NPC
    pub turn_in_npc: Option<String>,
    /// 任务描述
    pub description: String,
    /// 任务完成提示
    pub completion_message: String,
    /// 接受时间
    pub accepted_time: Option<i64>,
    /// 完成时间
    pub completed_time: Option<i64>,
    /// 任务时限 (秒, 0表示无时限)
    pub time_limit: u32,
}

impl Quest {
    /// 创建新任务
    pub fn new(id: String, name_cn: String, quest_type: QuestType) -> Self {
        Self {
            id: id.clone(),
            name: id.clone(),
            name_cn,
            quest_type,
            status: QuestStatus::NotStarted,
            objectives: Vec::new(),
            reward: Default::default(),
            prerequisite: None,
            required_level: 1,
            giver_npc: None,
            turn_in_npc: None,
            description: String::new(),
            completion_message: String::new(),
            accepted_time: None,
            completed_time: None,
            time_limit: 0,
        }
    }

    /// 添加目标
    pub fn with_objective(mut self, objective: QuestObjective) -> Self {
        self.objectives.push(objective);
        self
    }

    /// 设置奖励
    pub fn with_reward(mut self, reward: QuestReward) -> Self {
        self.reward = reward;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    /// 检查是否可以接受
    pub fn can_accept(&self, player_level: u32, completed_quests: &[String]) -> bool {
        // 检查等级要求
        if player_level < self.required_level {
            return false;
        }

        // 检查前置任务
        if let Some(ref prereq) = self.prerequisite {
            if !completed_quests.contains(prereq) {
                return false;
            }
        }

        true
    }

    /// 接受任务
    pub fn accept(&mut self) -> Result<()> {
        if self.status != QuestStatus::NotStarted {
            return Err(MudError::InvalidOperation("任务状态不正确".to_string()));
        }

        self.status = QuestStatus::InProgress;
        self.accepted_time = Some(chrono::Utc::now().timestamp());
        Ok(())
    }

    /// 更新目标进度
    pub fn update_objective(&mut self, objective_type: QuestObjectiveType, target_id: &str, count: u32) -> bool {
        if self.status != QuestStatus::InProgress {
            return false;
        }

        let mut all_completed = true;

        for objective in &mut self.objectives {
            if objective.objective_type == objective_type && objective.target_id == target_id {
                if !objective.completed {
                    objective.update(count);
                }
            }

            if !objective.completed {
                all_completed = false;
            }
        }

        if all_completed {
            self.complete();
        }

        all_completed
    }

    /// 完成任务
    pub fn complete(&mut self) {
        if self.status == QuestStatus::InProgress {
            self.status = QuestStatus::Completed;
            self.completed_time = Some(chrono::Utc::now().timestamp());
        }
    }

    /// 检查任务是否失败
    pub fn check_failed(&self) -> bool {
        if self.time_limit == 0 {
            return false;
        }

        if let Some(accepted_time) = self.accepted_time {
            let now = chrono::Utc::now().timestamp();
            let elapsed = (now - accepted_time) as u32;
            if elapsed > self.time_limit {
                return true;
            }
        }

        false
    }

    /// 检查是否所有目标都已完成
    pub fn is_all_objectives_completed(&self) -> bool {
        self.objectives.iter().all(|o| o.is_completed())
    }

    /// 渲染任务信息
    pub fn render_info(&self) -> String {
        let mut result = format!("§e【{}】§r {}\n",
            match self.quest_type {
                QuestType::Main => "主线",
                QuestType::Side => "支线",
                QuestType::Daily => "日常",
                QuestType::Weekly => "周常",
                QuestType::OneTime => "一次性",
                QuestType::Repeatable => "重复",
            },
            self.name_cn
        );

        result.push_str(&format!("{}\n\n", self.description));

        result.push_str("任务目标:\n");
        for (i, obj) in self.objectives.iter().enumerate() {
            let status = if obj.completed {
                "§g[已完成]§r"
            } else {
                &format!("§c({}/{})§r", obj.current_count, obj.target_count)
            };
            result.push_str(&format!("  {}. {} {}\n", i + 1, obj.target_id, status));
        }

        if !self.reward.items.is_empty() || self.reward.exp > 0 || self.reward.money > 0 {
            result.push_str("\n奖励:\n");
            if self.reward.exp > 0 {
                result.push_str(&format!("  经验: {}\n", self.reward.exp));
            }
            if self.reward.money > 0 {
                result.push_str(&format!("  金币: {}\n", self.reward.money));
            }
            if self.reward.potential > 0 {
                result.push_str(&format!("  潜能: {}\n", self.reward.potential));
            }
            for (item_id, count) in &self.reward.items {
                result.push_str(&format!("  {} x{}\n", item_id, count));
            }
        }

        result
    }
}

/// 玩家任务数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerQuestData {
    /// 当前进行中的任务 (任务ID -> 任务)
    pub active_quests: HashMap<String, Quest>,
    /// 已完成的任务ID列表
    pub completed_quests: Vec<String>,
    /// 可接受的任务ID列表
    pub available_quests: Vec<String>,
    /// 日常任务完成次数 (任务ID -> 次数)
    pub daily_completions: HashMap<String, u32>,
    /// 上次重置日常任务的时间
    pub last_daily_reset: i64,
}

impl Default for PlayerQuestData {
    fn default() -> Self {
        Self {
            active_quests: HashMap::new(),
            completed_quests: Vec::new(),
            available_quests: Vec::new(),
            daily_completions: HashMap::new(),
            last_daily_reset: chrono::Utc::now().timestamp(),
        }
    }
}

impl PlayerQuestData {
    /// 添加活动任务
    pub fn add_active_quest(&mut self, quest: Quest) -> Result<()> {
        let quest_id = quest.id.clone();
        if self.active_quests.contains_key(&quest_id) {
            return Err(MudError::InvalidOperation("任务已在进行中".to_string()));
        }
        self.active_quests.insert(quest_id, quest);
        Ok(())
    }

    /// 获取任务
    pub fn get_quest(&self, quest_id: &str) -> Option<&Quest> {
        self.active_quests.get(quest_id)
    }

    /// 获取可变任务
    pub fn get_quest_mut(&mut self, quest_id: &str) -> Option<&mut Quest> {
        self.active_quests.get_mut(quest_id)
    }

    /// 移除任务
    pub fn remove_quest(&mut self, quest_id: &str) -> Option<Quest> {
        self.active_quests.remove(quest_id)
    }

    /// 提交任务
    pub fn turn_in_quest(&mut self, quest_id: &str) -> Result<QuestReward> {
        let quest = self.active_quests.get(quest_id)
            .ok_or_else(|| MudError::NotFound("任务不存在".to_string()))?;

        if quest.status != QuestStatus::Completed {
            return Err(MudError::InvalidOperation("任务未完成".to_string()));
        }

        let reward = quest.reward.clone();

        // 移除活动任务，添加到已完成列表
        self.active_quests.remove(quest_id);
        self.completed_quests.push(quest_id.to_string());

        // 更新日常完成次数
        if let Some(count) = self.daily_completions.get_mut(quest_id) {
            *count += 1;
        } else {
            self.daily_completions.insert(quest_id.to_string(), 1);
        }

        Ok(reward)
    }

    /// 检查并重置日常任务
    pub fn check_reset_daily(&mut self) {
        use chrono::{TimeZone, Datelike};

        let now = chrono::Utc::now();
        let today = (now.year(), now.month(), now.day());

        let last_dt = chrono::Utc.timestamp_opt(self.last_daily_reset, 0);
        if let Some(last_dt) = last_dt.single() {
            let last_day = (last_dt.year(), last_dt.month(), last_dt.day());
            if today != last_day {
                // 重置日常完成次数
                self.daily_completions.clear();
                self.last_daily_reset = now.timestamp();
            }
        }
    }

    /// 渲染任务列表
    pub fn render_quest_list(&self) -> String {
        let mut result = String::from("=== 任务列表 ===\n");

        if self.active_quests.is_empty() {
            result.push_str("当前没有进行中的任务\n");
        } else {
            for (quest_id, quest) in &self.active_quests {
                let status = match quest.status {
                    QuestStatus::InProgress => {
                        if quest.is_all_objectives_completed() {
                            "§g[可提交]§r"
                        } else {
                            "§c[进行中]§r"
                        }
                    }
                    QuestStatus::Completed => "§g[可提交]§r",
                    QuestStatus::Failed => "§r[已失败]§r",
                    _ => "",
                };
                result.push_str(&format!("{} {} {}\n", quest_id, quest.name_cn, status));
            }
        }

        result
    }
}

/// 任务管理器
pub struct QuestManager {
    /// 任务模板
    quests: HashMap<String, Quest>,
}

impl QuestManager {
    pub fn new() -> Self {
        Self {
            quests: HashMap::new(),
        }
    }

    /// 注册任务模板
    pub fn register_quest(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }

    /// 获取任务模板
    pub fn get_quest_template(&self, quest_id: &str) -> Option<&Quest> {
        self.quests.get(quest_id)
    }

    /// 创建任务实例
    pub fn create_quest(&self, quest_id: &str) -> Result<Quest> {
        if let Some(template) = self.quests.get(quest_id) {
            let mut quest = template.clone();
            // 重置状态
            quest.status = QuestStatus::NotStarted;
            quest.accepted_time = None;
            quest.completed_time = None;
            for obj in &mut quest.objectives {
                obj.current_count = 0;
                obj.completed = false;
            }
            Ok(quest)
        } else {
            Err(MudError::NotFound(format!("任务不存在: {}", quest_id)))
        }
    }
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局任务管理器
pub static QUESTD: once_cell::sync::Lazy<std::sync::Mutex<QuestManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(QuestManager::default()));

/// 创建预设任务列表
pub fn create_preset_quests() -> Vec<Quest> {
    vec![
        // 新手任务：击杀10只史莱姆
        Quest::new(
            "quest_kill_slimes".to_string(),
            "史莱姆清理".to_string(),
            QuestType::Main,
        )
        .with_description("新手村的史莱姆太多了，请帮助清理10只。".to_string())
        .with_objective(QuestObjective::new(
            QuestObjectiveType::KillMonster,
            "slime".to_string(),
            10,
        ))
        .with_reward(QuestReward {
            exp: 100,
            money: 50,
            potential: 10,
            ..Default::default()
        }),

        // 收集任务：收集5个草药
        Quest::new(
            "quest_collect_herbs".to_string(),
            "草药采集".to_string(),
            QuestType::Side,
        )
        .with_description("药店老板需要一些草药来制作药品。".to_string())
        .with_objective(QuestObjective::new(
            QuestObjectiveType::CollectItem,
            "herb".to_string(),
            5,
        ))
        .with_reward(QuestReward {
            exp: 50,
            money: 30,
            items: vec![("medicine_small".to_string(), 3)],
            ..Default::default()
        }),

        // 日常任务：完成3次战斗
        Quest::new(
            "quest_daily_battle".to_string(),
            "日常战斗".to_string(),
            QuestType::Daily,
        )
        .with_description("每天完成3次战斗来保持战斗状态。".to_string())
        .with_objective(QuestObjective::new(
            QuestObjectiveType::KillMonster,
            "any".to_string(),
            3,
        ))
        .with_reward(QuestReward {
            exp: 200,
            money: 100,
            ..Default::default()
        }),
    ]
}
