// gamenv/single/daemons/userd.rs - 用户管理守护进程
// 对应 txpike9/gamenv/single/daemons/userd.pike

use crate::core::*;
use crate::gamenv::user::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 用户管理守护进程
///
/// 负责管理所有在线用户的信息
pub struct UserDaemon {
    /// 在线用户列表 (userid -> User)
    users: HashMap<String, Arc<RwLock<User>>>,
    /// 最大在线人数
    max_users: usize,
}

impl UserDaemon {
    /// 创建新的用户守护进程
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            max_users: 1000,
        }
    }

    /// 设置最大在线人数
    pub fn set_max_users(&mut self, max: usize) {
        self.max_users = max;
    }

    /// 用户登录
    pub async fn login(&mut self, userid: String) -> Result<Arc<RwLock<User>>> {
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

        tracing::info!("User logged in: {}, total users: {}", user_id, self.users.len());

        Ok(user)
    }

    /// 用户登出
    pub async fn logout(&mut self, userid: &str) -> Result<()> {
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
    pub async fn kick(&mut self, userid: &str) -> Result<()> {
        if self.users.remove(userid).is_some() {
            tracing::info!("User kicked: {}", userid);
            Ok(())
        } else {
            Err(MudError::NotFound(format!("用户 {} 不在线", userid)))
        }
    }

    /// 保存所有用户数据
    pub async fn save_all(&self) -> Result<()> {
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
}

impl Default for UserDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局用户守护进程
pub static USERD: std::sync::OnceLock<RwLock<UserDaemon>> = std::sync::OnceLock::new();

/// 获取用户守护进程
pub fn get_userd() -> &'static RwLock<UserDaemon> {
    USERD.get_or_init(|| RwLock::new(UserDaemon::new()))
}
