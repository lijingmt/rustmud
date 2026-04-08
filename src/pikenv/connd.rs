// pikenv/connd.rs - 连接管理器
// 对应 txpike9/pikenv/connd.pike

use crate::core::{GObject, ObjectId};
use crate::pikenv::conn::Connection;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 连接管理器 (对应 CONND)
pub struct ConnectionManager {
    /// 用户到连接的映射
    connections: DashMap<ObjectId, Arc<Connection>>,
    /// 当前玩家 (对应 this_player)
    this_player: Arc<RwLock<Option<ObjectId>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
            this_player: Arc::new(RwLock::new(None)),
        }
    }

    /// 设置连接 (对应 set_conn())
    pub fn set_conn(&self, user_id: ObjectId, conn: Arc<Connection>) {
        self.connections.insert(user_id, conn);
    }

    /// 获取连接 (对应 query_conn())
    pub fn get_conn(&self, user_id: ObjectId) -> Option<Arc<Connection>> {
        self.connections.get(&user_id).map(|v| v.clone())
    }

    /// 删除连接 (对应 erase_conn())
    pub fn erase_conn(&self, user_id: ObjectId) {
        self.connections.remove(&user_id);
    }

    /// 删除用户 (对应 erase_user())
    pub fn erase_user(&self, user_id: ObjectId) {
        self.erase_conn(user_id);
    }

    /// 设置当前玩家 (对应 set_this_player())
    pub async fn set_this_player(&self, user_id: ObjectId) {
        let mut tp = self.this_player.write().await;
        *tp = Some(user_id);
    }

    /// 获取当前玩家 (对应 this_player() / EFUNSD->query_this_player())
    pub async fn get_this_player(&self) -> Option<ObjectId> {
        let tp = self.this_player.read().await;
        *tp
    }

    /// 清除当前玩家
    pub async fn clear_this_player(&self) {
        let mut tp = self.this_player.write().await;
        *tp = None;
    }

    /// 获取所有用户 (对应 query_users())
    pub fn get_all_users(&self) -> Vec<ObjectId> {
        self.connections.iter().map(|v| *v.key()).collect()
    }

    /// 获取在线用户数
    pub fn user_count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// 全局连接管理器实例
pub static CONND: once_cell::sync::Lazy<ConnectionManager> =
    once_cell::sync::Lazy::new(ConnectionManager::new);
