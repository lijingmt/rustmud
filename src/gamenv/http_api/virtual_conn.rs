// gamenv/http_api/virtual_conn.rs - 虚拟连接池
// 对应 txpike9/gamenv/single/daemons/http_api/virtual_conn.pike

use crate::core::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 虚拟连接数据
#[derive(Clone)]
pub struct VirtualConnection {
    /// 用户 ID
    pub userid: String,
    /// 输出缓冲区
    pub buffer: Arc<RwLock<String>>,
    /// 最后使用时间
    pub last_used: Arc<RwLock<i64>>,
    /// 关联的玩家对象
    pub player: Option<GObject>,
}

impl VirtualConnection {
    /// 创建新的虚拟连接
    pub fn new(userid: String) -> Self {
        Self {
            userid,
            buffer: Arc::new(RwLock::new(String::new())),
            last_used: Arc::new(RwLock::new(chrono::Utc::now().timestamp())),
            player: None,
        }
    }

    /// 写入数据到缓冲区 (对应 BufferConnection->write())
    pub async fn write(&self, data: &str) {
        let mut buffer = self.buffer.write().await;
        buffer.push_str(data);
    }

    /// 获取缓冲区内容 (对应 BufferConnection->get_output())
    pub async fn get_output(&self) -> String {
        let buffer = self.buffer.read().await;
        buffer.clone()
    }

    /// 清空缓冲区 (对应 BufferConnection->clear())
    pub async fn clear(&self) {
        let mut buffer = self.buffer.write().await;
        buffer.clear();
    }

    /// 更新使用时间
    pub async fn update_time(&self) {
        let mut last = self.last_used.write().await;
        *last = chrono::Utc::now().timestamp();
    }

    /// 设置关联的玩家对象
    pub fn set_player(&mut self, player: GObject) {
        self.player = Some(player);
    }
}

/// 虚拟连接池 (对应 vconnections mapping)
pub struct VirtualConnectionPool {
    /// userid -> VirtualConnection
    connections: HashMap<String, VirtualConnection>,
    /// 连接超时时间 (秒)
    timeout_secs: i64,
}

impl VirtualConnectionPool {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            timeout_secs: 1800, // 30 分钟
        }
    }

    /// Get or create virtual connection
    pub async fn get_or_create(&mut self, userid: &str) -> std::result::Result<VirtualConnection, String> {
        // 清理过期连接
        self.cleanup_expired().await;

        // 查找现有连接
        if let Some(conn) = self.connections.get_mut(userid) {
            conn.update_time().await;
            return Ok(conn.clone());
        }

        // 创建新连接
        let conn = VirtualConnection::new(userid.to_string());
        self.connections.insert(userid.to_string(), conn.clone());
        Ok(conn)
    }

    /// 设置虚拟连接 (对应 set_virtual_connection)
    pub fn set(&mut self, userid: String, conn: VirtualConnection) {
        self.connections.insert(userid, conn);
    }

    /// 删除虚拟连接
    pub fn remove(&mut self, userid: &str) -> Option<VirtualConnection> {
        self.connections.remove(userid)
    }

    /// 更新连接使用时间 (对应 update_connection_time)
    pub async fn update_time(&mut self, userid: &str) {
        if let Some(conn) = self.connections.get_mut(userid) {
            conn.update_time().await;
        }
    }

    /// 清理过期连接
    async fn cleanup_expired(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.connections.retain(|userid, conn| {
            let last = *tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(conn.last_used.read())
            });
            let expired = now - last > self.timeout_secs;
            if expired {
                tracing::debug!("Cleaning up expired virtual connection: {}", userid);
            }
            !expired
        });
    }

    /// 获取连接数量
    pub fn count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for VirtualConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局虚拟连接池
pub static GLOBAL_VCONN_POOL: std::sync::OnceLock<std::sync::Mutex<VirtualConnectionPool>> =
    std::sync::OnceLock::new();

/// 获取全局虚拟连接池
pub fn get_global_pool() -> &'static std::sync::Mutex<VirtualConnectionPool> {
    GLOBAL_VCONN_POOL.get_or_init(|| std::sync::Mutex::new(VirtualConnectionPool::new()))
}
