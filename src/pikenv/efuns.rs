// pikenv/efuns.rs - 内置函数系统
// 对应 txpike9/pikenv/efuns.pike

use crate::core::*;
use crate::pikenv::connd::CONND;
use crate::pikenv::config::CONFIG;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 内置函数管理器 (对应 EFUNSD / efuns.pike)
pub struct EfunManager {
    /// ROOT 目录 (对应 efuns->ROOT)
    pub root: String,
    /// 端口号 (对应 efuns->port)
    pub port: u16,
    /// 日志文件后缀
    pub logfile_postfix: String,
    /// 游戏区号
    pub game_area: String,
    /// 心跳管理器
    heart_beats: Arc<RwLock<HeartBeatManager>>,
}

/// 心跳管理器 (对应 heart_beats mapping)
pub struct HeartBeatManager {
    // TODO: 实现心跳系统
}

impl EfunManager {
    /// 获取单例实例
    pub fn instance() -> Arc<Self> {
        static INSTANCE: once_cell::sync::OnceCell<Arc<EfunManager>> =
            once_cell::sync::OnceCell::new();

        INSTANCE.get_or_init(|| {
            Arc::new(Self::new())
        }).clone()
    }

    pub fn new() -> Self {
        let config = &*CONFIG;
        Self {
            root: config.root.clone(),
            port: config.port,
            logfile_postfix: config.log_prefix.clone(),
            game_area: config.game_area.clone(),
            heart_beats: Arc::new(RwLock::new(HeartBeatManager {})),
        }
    }

    // ========== 玩家相关 efuns ==========

    /// 获取所有玩家 (对应 users())
    pub async fn users(&self, all: bool) -> Vec<ObjectId> {
        let connd = &*CONND;
        if all {
            connd.get_all_users()
        } else {
            // 只返回活跃玩家
            connd.get_all_users()
        }
    }

    /// 获取当前玩家 (对应 this_player())
    pub async fn this_player(&self) -> Option<ObjectId> {
        CONND.get_this_player().await
    }

    /// 设置当前玩家 (对应 set_this_player())
    pub async fn set_this_player(&self, user_id: ObjectId) {
        CONND.set_this_player(user_id).await;
    }

    /// 广播消息给所有玩家 (对应 shout())
    pub async fn shout(&self, message: &str) {
        let connd = &*CONND;
        for user_id in connd.get_all_users() {
            if let Some(conn) = connd.get_conn(user_id) {
                // TODO: 发送消息
            }
        }
    }

    /// 向指定对象发送消息 (对应 tell_object())
    pub async fn tell_object(&self, target: ObjectId, message: &str) {
        let connd = &*CONND;
        if let Some(conn) = connd.get_conn(target) {
            // TODO: conn.write(message).await;
        }
    }

    // ========== 路径相关 efuns ==========

    /// 路径转换 (对应 pikenv_path())
    pub fn pikenv_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            // 绝对路径
            let full_path = format!("{}{}", self.root, path);
            if std::path::Path::new(&full_path).exists() {
                return full_path;
            }
        } else {
            // 相对路径
            let full_path = format!("{}/{}", self.root, path);
            if std::path::Path::new(&full_path).exists() {
                return full_path;
            }
        }
        path.to_string()
    }

    // ========== 对象相关 efuns ==========

    /// 加载对象 (对应 load_object())
    pub async fn load_object(&self, path: &str) -> Result<GObject> {
        let full_path = self.pikenv_path(path);
        // TODO: 实现对象加载
        Err(MudError::ObjectNotFound(full_path))
    }

    /// 保存对象 (对应 save_object())
    pub async fn save_object(&self, obj: &GObject, path: &str) -> Result<()> {
        obj.save_object(path.to_string()).await
    }

    /// 恢复对象 (对应 restore_object())
    pub async fn restore_object(&self, obj: &GObject, path: &str) -> Result<()> {
        obj.restore_object(path.to_string()).await
    }

    // ========== 命令相关 efuns ==========

    /// 执行命令 (对应 EFUNSD->command())
    pub async fn command(&self, cmd: &str, user: &GObject) -> Result<Value> {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command_name = parts[0];
        let args = parts.get(1).unwrap_or(&"");

        // TODO: 从 cmds/ 目录加载命令并执行
        tracing::debug!("Executing command: {} with args: {}", command_name, args);

        Ok(Value::Void)
    }

    // ========== 时间相关 efuns ==========

    /// 获取当前时间 (对应 time())
    pub fn time(&self) -> i64 {
        chrono::Utc::now().timestamp()
    }

    /// 获取当前时间字符串 (对应 ctime())
    pub fn ctime(&self, timestamp: i64) -> String {
        use chrono::TimeZone;
        let datetime = chrono::Local.timestamp_opt(timestamp, 0)
            .single()
            .unwrap_or_default();
        datetime.format("%a %b %d %H:%M:%S %Y").to_string()
    }

    // ========== 工具函数 ==========

    /// 获取游戏区号 (对应 query_game_area())
    pub fn query_game_area(&self) -> &str {
        &self.game_area
    }

    /// 设置游戏区号 (对应 set_game_area())
    pub fn set_game_area(&mut self, area: String) {
        self.game_area = area;
    }
}

// 全局 EFUNSD 实例
pub static EFUNSD: once_cell::sync::Lazy<Arc<EfunManager>> =
    once_cell::sync::Lazy::new(EfunManager::instance);

// ========== 工具函数 efuns ==========

/// 对应 a_delete() - 删除数组元素
pub fn a_delete<T>(arr: &mut Vec<T>, index: usize) -> Option<T> {
    if index < arr.len() {
        Some(arr.remove(index))
    } else {
        None
    }
}

/// 对应 a_insert() - 插入数组元素
pub fn a_insert<T>(arr: &mut Vec<T>, index: usize, value: T) {
    if index <= arr.len() {
        arr.insert(index, value);
    }
}

/// 对应 m_delete() - 删除映射元素
pub fn m_delete<K, V>(map: &mut std::collections::HashMap<K, V>, key: &K) -> Option<V>
where
    K: std::hash::Hash + Eq + Clone,
{
    map.remove(key)
}

/// 对应 mkmapping() - 创建映射
pub fn mkmapping<K, V>(pairs: Vec<(K, V)>) -> std::collections::HashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    pairs.into_iter().collect()
}

/// 对应 sizeof() - 获取大小
pub fn sizeof<T: ?Sized>(value: &T) -> usize {
    use std::mem::size_of_val;
    size_of_val(value)
}

/// 对应 sprintf() - 格式化字符串
pub fn sprintf(fmt: &str, args: &[Value]) -> String {
    let mut result = fmt.to_string();
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i); // {0}, {1}, etc.
        if result.contains(&placeholder) {
            result = result.replace(&placeholder, &format!("{:?}", arg));
        }
    }
    result
}

/// 对应 explode() - 分割字符串
pub fn explode(s: &str, delimiter: &str) -> Vec<String> {
    s.split(delimiter).map(|s| s.to_string()).collect()
}

/// 对应 implode() - 连接字符串数组
pub fn implode(arr: &[String], delimiter: &str) -> String {
    arr.join(delimiter)
}
