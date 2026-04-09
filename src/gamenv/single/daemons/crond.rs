// gamenv/single/daemons/crond.rs - 定时任务守护进程
// 对应 txpike9/gamenv/single/daemons/crond.pike

use crate::core::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// 定时任务
pub type CronJob = Arc<dyn CronJobTrait>;

/// 定时任务Trait
pub trait CronJobTrait: Send + Sync {
    /// 获取任务名称
    fn name(&self) -> &str;

    /// 获取Cron表达式
    fn schedule(&self) -> &str;

    /// 执行任务
    fn execute(&self) -> Result<()>;
}

/// 定时任务信息
#[derive(Clone, Debug)]
pub struct CronTask {
    /// 任务名称
    pub name: String,
    /// Cron表达式
    pub schedule: String,
    /// 下次执行时间
    pub next_run: i64,
    /// 任务是否启用
    pub enabled: bool,
}

/// 定时守护进程
pub struct CronDaemon {
    /// 注册的任务
    tasks: HashMap<String, CronTask>,
    /// 任务处理器
    handlers: HashMap<String, CronJob>,
}

impl CronDaemon {
    /// 创建新的定时守护进程
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// 注册定时任务
    pub fn register(&mut self, name: String, schedule: String, handler: CronJob) {
        let task = CronTask {
            name: name.clone(),
            schedule,
            next_run: Self::calculate_next_run("* * * * *"), // 默认每分钟
            enabled: true,
        };
        self.tasks.insert(name.clone(), task);
        self.handlers.insert(name, handler);
    }

    /// 启动定时守护进程
    pub async fn start(&self) {
        let tasks = self.tasks.clone();
        let handlers = self.handlers.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;

                let now = chrono::Utc::now().timestamp();
                for (name, task) in &tasks {
                    if !task.enabled {
                        continue;
                    }

                    if now >= task.next_run {
                        if let Some(handler) = handlers.get(name) {
                            tracing::info!("Executing cron job: {}", name);
                            if let Err(e) = handler.execute() {
                                tracing::error!("Cron job {} failed: {:?}", name, e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// 计算下次执行时间（简化版）
    fn calculate_next_run(cron_expr: &str) -> i64 {
        // TODO: 实现完整的cron表达式解析
        // 简化版：默认每分钟执行
        chrono::Utc::now().timestamp() + 60
    }

    /// 手动触发任务
    pub fn trigger(&self, name: &str) -> Result<()> {
        if let Some(handler) = self.handlers.get(name) {
            handler.execute()
        } else {
            Err(MudError::NotFound(format!("任务 {} 不存在", name)))
        }
    }
}

impl Default for CronDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局定时守护进程
pub static CROND: std::sync::OnceLock<RwLock<CronDaemon>> = std::sync::OnceLock::new();

/// 获取定时守护进程
pub fn get_crond() -> &'static RwLock<CronDaemon> {
    CROND.get_or_init(|| RwLock::new(CronDaemon::new()))
}

/// 示例定时任务：清理离线用户
pub struct CleanupIdleUsersJob;

impl CronJobTrait for CleanupIdleUsersJob {
    fn name(&self) -> &str {
        "cleanup_idle_users"
    }

    fn schedule(&self) -> &str {
        "*/5 * * * *" // 每5分钟
    }

    fn execute(&self) -> Result<()> {
        tracing::info!("Running cleanup idle users job");
        // TODO: 清理离线用户
        Ok(())
    }
}

/// 示例定时任务：保存所有数据
pub struct SaveAllDataJob;

impl CronJobTrait for SaveAllDataJob {
    fn name(&self) -> &str {
        "save_all_data"
    }

    fn schedule(&self) -> &str {
        "*/30 * * * *" // 每30分钟
    }

    fn execute(&self) -> Result<()> {
        tracing::info!("Running save all data job");
        // TODO: 保存所有数据
        Ok(())
    }
}

/// 示例定时任务：排行榜更新
pub struct UpdateRankingsJob;

impl CronJobTrait for UpdateRankingsJob {
    fn name(&self) -> &str {
        "update_rankings"
    }

    fn schedule(&self) -> &str {
        "0 * * * *" // 每小时
    }

    fn execute(&self) -> Result<()> {
        tracing::info!("Running update rankings job");
        // TODO: 更新排行榜
        Ok(())
    }
}
