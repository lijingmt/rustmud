// gamenv/http_api/thread_manager.rs - 线程管理器
// 对应 txpike9/gamenv/single/daemons/http_api/thread_manager.pike

use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore, OwnedSemaphorePermit};

/// 线程/任务管理器
pub struct ThreadManager {
    /// 任务信号量 (限制并发数)
    semaphore: Arc<Semaphore>,
    /// 活跃任务数
    active_tasks: Arc<RwLock<usize>>,
    /// 最大并发数
    max_concurrent: usize,
}

impl ThreadManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_tasks: Arc::new(RwLock::new(0)),
            max_concurrent,
        }
    }

    /// 获取执行许可 (对应 acquire_thread)
    pub async fn acquire(&self) -> ThreadPermit {
        // 等待可用线程 - 使用 acquire_owned 获取 owned permit
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        let mut active = self.active_tasks.write().await;
        *active += 1;

        ThreadPermit {
            _permit: Some(permit),
            active_tasks: self.active_tasks.clone(),
        }
    }

    /// 获取当前活跃任务数
    pub async fn active_count(&self) -> usize {
        *self.active_tasks.read().await
    }

    /// 获取可用线程数
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }
}

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new(100) // 默认100个并发
    }
}

/// 线程许可 (自动释放)
pub struct ThreadPermit {
    _permit: Option<OwnedSemaphorePermit>,
    active_tasks: Arc<RwLock<usize>>,
}

impl Drop for ThreadPermit {
    fn drop(&mut self) {
        let active = self.active_tasks.clone();
        tokio::spawn(async move {
            let mut count = active.write().await;
            *count = count.saturating_sub(1);
        });
    }
}

/// 全局线程管理器
pub static THREAD_MANAGER: std::sync::OnceLock<ThreadManager> =
    std::sync::OnceLock::new();

/// 获取全局线程管理器
pub fn get_thread_manager() -> &'static ThreadManager {
    THREAD_MANAGER.get_or_init(|| ThreadManager::default())
}
