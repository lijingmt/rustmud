// gamenv/single/daemons/userd_v2.rs - 用户管理守护进程（新版本）
// 对应 txpike9/gamenv/single/daemons/userd.pike
// 实现统一的 Daemon trait 接口

use crate::core::{MudError, GObject, GObjectExt};
use crate::gamenv::traits::daemon::*;
use crate::gamenv::user::User;
use std::collections::HashMap;
use std::sync::Arc;
use std::result::Result;
use tokio::sync::RwLock;

/// 用户管理守护进程（新版本）
///
/// 实现了 Daemon trait，支持统一的生命周期管理
pub struct UserDaemonV2 {
    /// 在线用户列表 (userid -> User)
    users: HashMap<String, Arc<RwLock<User>>>,
    /// 最大在线人数
    max_users: usize,
    /// 守护进程状态
    state: DaemonState,
    /// 统计信息
    stats: DaemonStats,
}

impl UserDaemonV2 {
    /// 创建新的用户守护进程
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            max_users: 1000,
            state: DaemonState::Uninitialized,
            stats: DaemonStats::default(),
        }
    }

    /// 设置最大在线人数
    pub fn set_max_users(&mut self, max: usize) {
        self.max_users = max;
    }

    /// 用户登录
    pub async fn login(&mut self, userid: String) -> Result<Arc<RwLock<User>>, MudError> {
        // 检查是否已在线
        if let Some(user) = self.users.get(&userid) {
            return Ok(user.clone());
        }

        // 检查人数限制
        if self.users.len() >= self.max_users {
            return Err(MudError::RuntimeError("服务器人数已满".to_string()));
        }

        // 创建新用户
        let user_id = userid.clone();
        let user = Arc::new(RwLock::new(User::new(userid.clone())));
        self.users.insert(userid.clone(), user.clone());
        self.stats.requests_processed += 1;

        tracing::info!("User logged in: {}, total users: {}", user_id, self.users.len());

        Ok(user)
    }

    /// 用户登出
    pub async fn logout(&mut self, userid: &str) -> Result<(), MudError> {
        if let Some(user) = self.users.remove(userid) {
            tracing::info!("User logged out: {}, remaining users: {}",
                userid, self.users.len());

            // 保存用户数据
            let u = user.read().await;
            // TODO: 实现保存到数据库/文件
            drop(u);
        }
        Ok(())
    }

    /// 获取用户
    pub async fn get_user(&self, userid: &str) -> Option<Arc<RwLock<User>>> {
        self.users.get(userid).cloned()
    }

    /// 获取所有在线用户
    pub async fn get_online_users(&self) -> Vec<String> {
        self.users.keys().cloned().collect()
    }

    /// 获取在线人数
    pub fn get_online_count(&self) -> usize {
        self.users.len()
    }

    /// 广播消息给所有用户
    pub async fn broadcast(&self, message: &str) {
        for user in self.users.values() {
            let u = user.read().await;
            // TODO: 发送消息给用户
            drop(u);
        }
    }

    /// 广播消息给指定区域的所有用户
    pub async fn broadcast_to_zone(&self, zone: &str, message: &str) {
        for user in self.users.values() {
            let u = user.read().await;
            if let Some(ref room_id) = u.room_id {
                // 简化处理：检查房间ID是否包含zone名称
                if room_id.contains(zone) {
                    // TODO: 发送消息给用户
                }
            }
            drop(u);
        }
    }

    /// 根据名字查找用户
    pub async fn find_user_by_name(&self, name: &str) -> Option<Arc<RwLock<User>>> {
        for user in self.users.values() {
            let u = user.read().await;
            if u.name == name {
                drop(u);
                return Some(user.clone());
            }
            drop(u);
        }
        None
    }

    /// 检查用户是否在线
    pub fn is_online(&self, userid: &str) -> bool {
        self.users.contains_key(userid)
    }

    /// 踢出用户
    pub async fn kick(&mut self, userid: &str) -> Result<(), MudError> {
        if self.users.remove(userid).is_some() {
            tracing::info!("User kicked: {}", userid);
            Ok(())
        } else {
            Err(MudError::NotFound(format!("用户 {} 不在线", userid)))
        }
    }

    /// 保存所有用户数据
    pub async fn save_all(&self) -> Result<(), MudError> {
        for (userid, user) in &self.users {
            let u = user.read().await;
            tracing::info!("Saving user: {}", userid);
            // TODO: 实现保存到数据库/文件
            drop(u);
        }
        Ok(())
    }

    /// 定时清理离线用户
    pub async fn cleanup_idle_users(&mut self, idle_seconds: i64) {
        let now = chrono::Utc::now().timestamp();
        let mut to_remove = Vec::new();

        for (userid, user) in &self.users {
            let u = user.read().await;
            if let Some(login_time) = u.login_time {
                if now - login_time > idle_seconds {
                    to_remove.push(userid.clone());
                }
            }
            drop(u);
        }

        for userid in to_remove {
            let _ = self.logout(&userid).await;
        }
    }

    /// 更新统计信息
    fn update_stats(&mut self) {
        self.stats.add_metric("online_users".to_string(), self.users.len().to_string());
        self.stats.add_metric("max_users".to_string(), self.max_users.to_string());
    }
}

impl Default for UserDaemonV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for UserDaemonV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserDaemonV2")
            .field("users_count", &self.users.len())
            .field("max_users", &self.max_users)
            .field("state", &self.state)
            .finish()
    }
}

/// 实现 Daemon trait
#[async_trait::async_trait]
impl Daemon for UserDaemonV2 {
    fn name(&self) -> &str {
        "userd"
    }

    fn description(&self) -> &str {
        "用户管理守护进程 - 管理所有在线用户"
    }

    async fn initialize(&mut self) -> Result<(), DaemonError> {
        self.state = DaemonState::Initializing;
        tracing::info!("UserDaemon 初始化中...");

        // 初始化逻辑：从数据库加载在线用户状态等
        // TODO: 从持久化存储恢复用户状态

        self.state = DaemonState::Running;
        self.update_stats();
        tracing::info!("UserDaemon 初始化完成");
        Ok(())
    }

    async fn start(&mut self) -> Result<(), DaemonError> {
        if self.state == DaemonState::Running {
            return Err(DaemonError::AlreadyRunning("userd".to_string()));
        }

        tracing::info!("UserDaemon 启动...");
        self.state = DaemonState::Running;

        // 启动定时清理任务
        // let daemon_weak = Arc::downgrade(&Arc::new(RwLock::new(UserDaemonV2::new())));
        // TODO: 启动后台清理任务

        tracing::info!("UserDaemon 启动完成");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), DaemonError> {
        self.state = DaemonState::ShuttingDown;
        tracing::info!("UserDaemon 关闭中...");

        // 保存所有用户数据
        if let Err(e) = self.save_all().await {
            tracing::warn!("保存用户数据失败: {}", e);
        }

        self.state = DaemonState::Shutdown;
        tracing::info!("UserDaemon 已关闭");
        Ok(())
    }

    fn state(&self) -> DaemonState {
        self.state.clone()
    }

    async fn heartbeat(&mut self) -> bool {
        // 清理长时间不活动的用户
        self.cleanup_idle_users(3600).await; // 1小时
        self.update_stats();
        true // 始终健康
    }

    fn stats(&self) -> DaemonStats {
        let mut stats = self.stats.clone();
        stats.add_metric("online_users".to_string(), self.users.len().to_string());
        stats
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// 全局用户守护进程（新版本）
pub static USERD_V2: std::sync::OnceLock<Arc<tokio::sync::RwLock<UserDaemonV2>>> =
    std::sync::OnceLock::new();

/// 获取用户守护进程（新版本）
pub fn get_userd_v2() -> Arc<tokio::sync::RwLock<UserDaemonV2>> {
    USERD_V2.get_or_init(|| {
        Arc::new(tokio::sync::RwLock::new(UserDaemonV2::new()))
    }).clone()
}

/// 辅助函数：登录（兼容旧接口）
pub async fn login_v2(userid: String) -> Result<Arc<RwLock<User>>, MudError> {
    let userd = get_userd_v2();
    let mut userd = userd.write().await;
    userd.login(userid).await
}

/// 辅助函数：登出（兼容旧接口）
pub async fn logout_v2(userid: &str) -> Result<(), MudError> {
    let userd = get_userd_v2();
    let mut userd = userd.write().await;
    userd.logout(userid).await.map_err(|e| match e {
        MudError::NotFound(msg) => MudError::NotFound(msg),
        _ => MudError::RuntimeError(format!("登出失败: {}", e)),
    })
}

/// 辅助函数：获取用户（兼容旧接口）
pub async fn get_user_v2(userid: &str) -> Option<Arc<RwLock<User>>> {
    let userd = get_userd_v2();
    let userd = userd.read().await;
    userd.get_user(userid).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_userd_v2_basic() {
        let mut userd = UserDaemonV2::new();

        // 测试初始化
        assert!(userd.initialize().await.is_ok());
        assert_eq!(userd.state(), DaemonState::Running);

        // 测试登录
        let user = userd.login("test_user".to_string()).await;
        assert!(user.is_ok());
        assert_eq!(userd.get_online_count(), 1);

        // 测试获取用户
        let found = userd.get_user("test_user").await;
        assert!(found.is_some());

        // 测试登出
        assert!(userd.logout("test_user").await.is_ok());
        assert_eq!(userd.get_online_count(), 0);
    }

    #[tokio::test]
    async fn test_userd_v2_limits() {
        let mut userd = UserDaemonV2::new();
        userd.set_max_users(2);

        userd.initialize().await.ok();

        // 可以登录2个用户
        assert!(userd.login("user1".to_string()).await.is_ok());
        assert!(userd.login("user2".to_string()).await.is_ok());

        // 第3个用户被拒绝
        assert!(userd.login("user3".to_string()).await.is_err());
    }
}
