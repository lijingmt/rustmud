// gamenv/traits/daemon.rs - 守护进程系统
// 对应 txpike9/gamenv/single/daemons/ 单例模式
// 提供统一的守护进程接口和管理器

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

/// 守护进程状态
#[derive(Clone, Debug, PartialEq)]
pub enum DaemonState {
    /// 未初始化
    Uninitialized,
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 关闭中
    ShuttingDown,
    /// 已关闭
    Shutdown,
}

/// 守护进程 trait - 所有守护进程都需要实现
///
/// 对应 txpike9 中的各种 *_d.pike 守护进程文件
#[async_trait::async_trait]
pub trait Daemon: Send + Sync + Any + fmt::Debug {
    /// 守护进程名称（唯一标识符）
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// 守护进程描述
    fn description(&self) -> &str {
        "守护进程"
    }

    /// 初始化守护进程
    ///
    /// 在服务器启动时调用，用于加载数据、建立连接等
    async fn initialize(&mut self) -> Result<(), DaemonError>;

    /// 启动守护进程
    ///
    /// 开始处理任务、定时器等
    async fn start(&mut self) -> Result<(), DaemonError>;

    /// 停止守护进程
    ///
    /// 优雅关闭，保存数据等
    async fn stop(&mut self) -> Result<(), DaemonError>;

    /// 获取守护进程状态
    fn state(&self) -> DaemonState;

    /// 心跳 - 定期调用的健康检查
    ///
    /// 返回 true 表示健康，false 表示需要重启
    async fn heartbeat(&mut self) -> bool {
        true // 默认健康
    }

    /// 获取统计信息
    fn stats(&self) -> DaemonStats {
        DaemonStats::default()
    }

    /// 转换为 Any 以支持 downcasting
    fn as_any(&self) -> &dyn Any;

    /// 转换为 Any (mutable)
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// 守护进程错误
#[derive(Clone, Debug)]
pub enum DaemonError {
    /// 守护进程未初始化
    Uninitialized(String),
    /// 守护进程已运行
    AlreadyRunning(String),
    /// 守护进程未运行
    NotRunning(String),
    /// 初始化失败
    InitializationFailed(String),
    /// 运行时错误
    RuntimeError(String),
    /// 关闭失败
    ShutdownFailed(String),
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonError::Uninitialized(name) => write!(f, "守护进程 {} 未初始化", name),
            DaemonError::AlreadyRunning(name) => write!(f, "守护进程 {} 已在运行", name),
            DaemonError::NotRunning(name) => write!(f, "守护进程 {} 未运行", name),
            DaemonError::InitializationFailed(msg) => write!(f, "初始化失败: {}", msg),
            DaemonError::RuntimeError(msg) => write!(f, "运行时错误: {}", msg),
            DaemonError::ShutdownFailed(msg) => write!(f, "关闭失败: {}", msg),
        }
    }
}

impl std::error::Error for DaemonError {}

/// 守护进程统计信息
#[derive(Clone, Debug, Default)]
pub struct DaemonStats {
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 处理的请求数
    pub requests_processed: u64,
    /// 错误次数
    pub error_count: u64,
    /// 最后心跳时间
    pub last_heartbeat: i64,
    /// 自定义统计数据
    pub custom_metrics: HashMap<String, String>,
}

impl DaemonStats {
    /// 添加自定义指标
    pub fn add_metric(&mut self, key: String, value: String) {
        self.custom_metrics.insert(key, value);
    }

    /// 获取指标
    pub fn get_metric(&self, key: &str) -> Option<&String> {
        self.custom_metrics.get(key)
    }
}

/// 守护进程包装器 - 统一管理守护进程的生命周期
pub struct DaemonWrapper {
    /// 守护进程实例
    daemon: TokioRwLock<Box<dyn Daemon>>,
    /// 启动时间
    start_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl DaemonWrapper {
    /// 创建新的守护进程包装器
    pub fn new(daemon: Box<dyn Daemon>) -> Self {
        Self {
            daemon: TokioRwLock::new(daemon),
            start_time: None,
        }
    }

    /// 初始化守护进程
    pub async fn initialize(&self) -> Result<(), DaemonError> {
        let mut daemon = self.daemon.write().await;
        daemon.initialize().await?;
        Ok(())
    }

    /// 启动守护进程
    pub async fn start(&mut self) -> Result<(), DaemonError> {
        {
            let daemon = self.daemon.read().await;
            if daemon.state() == DaemonState::Running {
                return Err(DaemonError::AlreadyRunning(daemon.name().to_string()));
            }
        }

        {
            let mut daemon = self.daemon.write().await;
            daemon.start().await?;
            self.start_time = Some(chrono::Utc::now());
        }
        Ok(())
    }

    /// 停止守护进程
    pub async fn stop(&mut self) -> Result<(), DaemonError> {
        let mut daemon = self.daemon.write().await;
        daemon.stop().await?;
        self.start_time = None;
        Ok(())
    }

    /// 心跳检查
    pub async fn heartbeat(&self) -> bool {
        let mut daemon = self.daemon.write().await;
        daemon.heartbeat().await
    }

    /// 获取守护进程状态
    pub async fn state(&self) -> DaemonState {
        let daemon = self.daemon.read().await;
        daemon.state()
    }

    /// 获取统计信息
    pub async fn stats(&self) -> DaemonStats {
        let daemon = self.daemon.read().await;
        let mut stats = daemon.stats();
        if let Some(start_time) = self.start_time {
            stats.uptime_seconds = (chrono::Utc::now() - start_time).num_seconds() as u64;
        }
        stats.last_heartbeat = chrono::Utc::now().timestamp();
        stats
    }

    /// 获取守护进程名称
    pub async fn name(&self) -> String {
        let daemon = self.daemon.read().await;
        daemon.name().to_string()
    }

    /// 获取守护进程描述
    pub async fn description(&self) -> String {
        let daemon = self.daemon.read().await;
        daemon.description().to_string()
    }

    /// 检查守护进程是否为指定类型
    pub async fn is_type<D: Daemon + 'static>(&self) -> bool {
        let daemon = self.daemon.read().await;
        daemon.as_any().is::<D>()
    }
}

/// 守护进程管理器 - 全局单例
///
/// 对应 txpike9 中的各个守护进程单例（USERD, PKD, SCHOOLD 等）
pub struct DaemonManager {
    /// 注册的守护进程
    daemons: TokioRwLock<HashMap<String, DaemonWrapper>>,
    /// 管理器状态
    state: TokioRwLock<DaemonState>,
}

impl DaemonManager {
    /// 创建新的守护进程管理器
    pub fn new() -> Self {
        Self {
            daemons: TokioRwLock::new(HashMap::new()),
            state: TokioRwLock::new(DaemonState::Uninitialized),
        }
    }

    /// 注册守护进程
    pub async fn register(&self, daemon: Box<dyn Daemon>) -> Result<(), DaemonError> {
        let name = {
            let d = daemon.as_ref();
            d.name().to_string()
        };

        let mut daemons = self.daemons.write().await;
        if daemons.contains_key(&name) {
            return Err(DaemonError::AlreadyRunning(format!("守护进程 {} 已注册", name)));
        }

        daemons.insert(name.clone(), DaemonWrapper::new(daemon));
        tracing::info!("守护进程 {} 已注册", name);
        Ok(())
    }

    /// 初始化所有守护进程
    pub async fn initialize_all(&self) -> Result<(), DaemonError> {
        {
            let mut state = self.state.write().await;
            *state = DaemonState::Initializing;
        }

        let daemons = self.daemons.read().await;
        for (name, wrapper) in daemons.iter() {
            if let Err(e) = wrapper.initialize().await {
                tracing::error!("守护进程 {} 初始化失败: {}", name, e);
                return Err(DaemonError::InitializationFailed(format!("{}: {}", name, e)));
            }
            tracing::info!("守护进程 {} 初始化成功", name);
        }

        {
            let mut state = self.state.write().await;
            *state = DaemonState::Running;
        }

        Ok(())
    }

    /// 启动所有守护进程
    pub async fn start_all(&self) -> Result<(), DaemonError> {
        let mut daemons = self.daemons.write().await;
        for (name, wrapper) in daemons.iter_mut() {
            if let Err(e) = wrapper.start().await {
                tracing::error!("守护进程 {} 启动失败: {}", name, e);
                return Err(DaemonError::RuntimeError(format!("{}: {}", name, e)));
            }
            tracing::info!("守护进程 {} 启动成功", name);
        }
        Ok(())
    }

    /// 停止所有守护进程
    pub async fn stop_all(&self) -> Result<(), DaemonError> {
        {
            let mut state = self.state.write().await;
            *state = DaemonState::ShuttingDown;
        }

        let mut daemons = self.daemons.write().await;
        for (name, wrapper) in daemons.iter_mut() {
            if let Err(e) = wrapper.stop().await {
                tracing::warn!("守护进程 {} 停止失败: {}", name, e);
            } else {
                tracing::info!("守护进程 {} 已停止", name);
            }
        }

        {
            let mut state = self.state.write().await;
            *state = DaemonState::Shutdown;
        }

        Ok(())
    }

    /// 获取守护进程
    pub async fn get(&self, name: &str) -> Option<DaemonWrapper> {
        let daemons = self.daemons.read().await;
        // Note: This is a simplified version - in reality we'd need
        // to think about how to return a reference to the wrapper
        // For now, we'll clone the Arc reference if we wrap it
        None // TODO: Implement proper reference handling
    }

    /// 列出所有守护进程
    pub async fn list(&self) -> Vec<String> {
        let daemons = self.daemons.read().await;
        daemons.keys().cloned().collect()
    }

    /// 获取所有守护进程状态
    pub async fn get_all_stats(&self) -> HashMap<String, (DaemonState, DaemonStats)> {
        let daemons = self.daemons.read().await;
        let mut result = HashMap::new();

        for (name, wrapper) in daemons.iter() {
            let state = wrapper.state().await;
            let stats = wrapper.stats().await;
            result.insert(name.clone(), (state, stats));
        }

        result
    }

    /// 全局心跳检查
    pub async fn heartbeat_all(&self) -> Result<(), DaemonError> {
        let daemons = self.daemons.read().await;

        for (name, wrapper) in daemons.iter() {
            tokio::spawn({
                let name = name.clone();
                async move {
                    // 实际的心跳逻辑会在这里
                    // 目前只是示例
                }
            });
        }

        Ok(())
    }

    /// 获取管理器状态
    pub async fn state(&self) -> DaemonState {
        let state = self.state.read().await;
        state.clone()
    }
}

impl Default for DaemonManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局守护进程管理器
static DAEMON_MANAGER: once_cell::sync::Lazy<Arc<DaemonManager>> =
    once_cell::sync::Lazy::new(|| Arc::new(DaemonManager::new()));

/// 获取全局守护进程管理器
pub fn get_daemon_manager() -> Arc<DaemonManager> {
    DAEMON_MANAGER.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用守护进程
    #[derive(Debug)]
    struct TestDaemon {
        state: DaemonState,
    }

    #[async_trait::async_trait]
    impl Daemon for TestDaemon {
        fn name(&self) -> &str {
            "test_daemon"
        }

        fn description(&self) -> &str {
            "测试守护进程"
        }

        async fn initialize(&mut self) -> Result<(), DaemonError> {
            self.state = DaemonState::Running;
            Ok(())
        }

        async fn start(&mut self) -> Result<(), DaemonError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), DaemonError> {
            self.state = DaemonState::Shutdown;
            Ok(())
        }

        fn state(&self) -> DaemonState {
            self.state.clone()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_daemon_manager() {
        let manager = DaemonManager::new();
        let daemon = Box::new(TestDaemon {
            state: DaemonState::Uninitialized,
        });

        // 注册守护进程
        assert!(manager.register(daemon).await.is_ok());

        // 列出守护进程
        let list = manager.list().await;
        assert_eq!(list.len(), 1);
        assert!(list.contains(&"test_daemon".to_string()));
    }
}
