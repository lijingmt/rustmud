// gamenv/single/daemons/taskd.rs - 任务系统守护进程
// 对应 txpike9/gamenv/single/daemons/taskd.pike

use crate::core::*;
use crate::gamenv::player_state::{PlayerState, QuestProgress};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 任务类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    /// 击杀任务
    Kill,
    /// 收集任务
    Collect,
    /// 护送任务
    Escort,
    /// 探索任务
    Explore,
    /// 对话任务
    Talk,
    /// 副本任务
    Dungeon,
}

/// 任务状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// 未接取
    NotStarted,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
}

/// 任务
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 目标ID (怪物ID、物品ID等)
    pub target_id: String,
    /// 目标数量
    pub target_count: i32,
    /// 最小等级
    pub min_level: i32,
    /// 最大等级
    pub max_level: i32,
    /// 奖励经验
    pub reward_exp: u32,
    /// 奖励金币
    pub reward_gold: u32,
    /// 奖励物品
    pub reward_items: Vec<(String, i32)>,
    /// 前置任务
    pub prerequisite: Option<String>,
    /// 自动接取
    pub auto_accept: bool,
}

/// 任务模板
#[derive(Clone, Debug)]
pub struct TaskTemplate {
    pub task: Task,
    pub on_accept: Option<String>,
    pub on_complete: Option<String>,
    pub on_fail: Option<String>,
}

/// 任务守护进程
pub struct TaskDaemon {
    /// 所有任务模板
    tasks: HashMap<String, TaskTemplate>,
    /// 玩家任务状态 (userid -> task_id -> status)
    player_tasks: HashMap<String, HashMap<String, TaskStatus>>,
}

impl TaskDaemon {
    /// 创建新的任务守护进程
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            player_tasks: HashMap::new(),
        }
    }

    /// 添加任务
    pub fn add_task(&mut self, task: TaskTemplate) {
        self.tasks.insert(task.task.id.clone(), task);
    }

    /// 获取任务
    pub fn get_task(&self, task_id: &str) -> Option<&TaskTemplate> {
        self.tasks.get(task_id)
    }

    /// 获取可接任务列表
    pub fn get_available_tasks(&self, player: &PlayerState) -> Vec<&Task> {
        let level = player.level as i32;

        self.tasks.values()
            .filter(|template| {
                let task = &template.task;
                // 检查等级
                if level < task.min_level || level > task.max_level {
                    return false;
                }
                // 检查前置任务
                if let Some(ref prereq) = task.prerequisite {
                    if !player.completed_quests.contains(prereq) {
                        return false;
                    }
                }
                // 检查是否已完成
                if player.completed_quests.contains(&task.id) {
                    return false;
                }
                true
            })
            .map(|t| &t.task)
            .collect()
    }

    /// 检查任务进度
    pub fn check_progress(&mut self, userid: &str, target_id: &str, count: i32) -> Vec<Task> {
        let mut completed = Vec::new();

        if let Some(tasks) = self.player_tasks.get_mut(userid) {
            let to_remove: Vec<String> = tasks.iter()
                .filter(|(_, status)| **status == TaskStatus::InProgress)
                .map(|(id, _)| id.clone())
                .collect();

            for task_id in to_remove {
                if let Some(template) = self.tasks.get(&task_id) {
                    if template.task.target_id == target_id {
                        // 检查是否完成
                        // TODO: 实现任务进度检查
                        completed.push(template.task.clone());
                    }
                }
            }
        }

        completed
    }

    /// 格式化任务列表
    pub fn format_task_list(&self, tasks: &[&Task]) -> String {
        let mut output = String::from("§H=== 可接任务 ===§N\n");

        if tasks.is_empty() {
            output.push_str("暂时没有可接任务。\n");
        } else {
            for task in tasks {
                output.push_str(&format!(
                    "§Y[{}]§N {} (Lv.{}-{})\n",
                    Self::format_task_type(&task.task_type),
                    task.name,
                    task.min_level,
                    task.max_level
                ));
                output.push_str(&format!("  {}\n", task.description));
                output.push_str(&format!("  奖励: {}经验, {}金币\n",
                    task.reward_exp, task.reward_gold));
                output.push('\n');
            }
        }

        output
    }

    /// 格式化任务类型
    fn format_task_type(task_type: &TaskType) -> &str {
        match task_type {
            TaskType::Kill => "击杀",
            TaskType::Collect => "收集",
            TaskType::Escort => "护送",
            TaskType::Explore => "探索",
            TaskType::Talk => "对话",
            TaskType::Dungeon => "副本",
        }
    }

    /// 初始化默认任务
    pub fn init_default_tasks(&mut self) {
        tracing::info!("Initializing default tasks...");

        // 新手任务：击杀野猪
        let kill_boars = TaskTemplate {
            task: Task {
                id: "quest_kill_boars".to_string(),
                name: "野猪威胁".to_string(),
                description: "新手村外的野猪最近很猖獗，去解决5只野猪吧！".to_string(),
                task_type: TaskType::Kill,
                target_id: "boar".to_string(),
                target_count: 5,
                min_level: 1,
                max_level: 10,
                reward_exp: 100,
                reward_gold: 50,
                reward_items: vec![],
                prerequisite: None,
                auto_accept: true,
            },
            on_accept: None,
            on_complete: Some("give_newbie_weapon".to_string()),
            on_fail: None,
        };

        self.add_task(kill_boars);
    }
}

impl Default for TaskDaemon {
    fn default() -> Self {
        let mut daemon = Self::new();
        daemon.init_default_tasks();
        daemon
    }
}

/// 全局任务守护进程
pub static TASKD: std::sync::OnceLock<RwLock<TaskDaemon>> = std::sync::OnceLock::new();

/// 获取任务守护进程
pub fn get_taskd() -> &'static RwLock<TaskDaemon> {
    TASKD.get_or_init(|| RwLock::new(TaskDaemon::default()))
}
