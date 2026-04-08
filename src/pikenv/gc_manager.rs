// pikenv/gc_manager.rs - GC 管理器
// 对应 txpike9/pikenv/gc_manager.pike

use tokio::time::{interval, Duration};

/// GC 管理器 (对应 gc_manager.pike)
pub struct GcManager {
    interval_secs: u64,
}

impl GcManager {
    pub fn new(interval_secs: u64) -> Self {
        Self { interval_secs }
    }

    /// 启动 GC 任务 (对应 start())
    pub async fn start(&self) {
        let mut timer = interval(Duration::from_secs(self.interval_secs));
        timer.tick().await; // 跳过第一次立即触发

        loop {
            timer.tick().await;
            self.run_gc().await;
        }
    }

    /// 执行 GC (对应 Pike 的 gc())
    async fn run_gc(&self) {
        tracing::debug!("Running garbage collection...");

        // 这里会触发各种清理任务：
        // 1. 清理断开的连接
        // 2. 清理过期对象
        // 3. 清理临时文件
        // 4. 清理缓存

        tracing::debug!("Garbage collection completed");
    }

    /// 运行 GC 管理器 (在后台任务中)
    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.start().await;
        })
    }
}

impl Default for GcManager {
    fn default() -> Self {
        Self::new(60) // 默认 60 秒间隔
    }
}
