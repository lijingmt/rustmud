// gamenv/hidden_cmd.rs - 命令隐藏系统
// 对应 txpike9 的 hidden.pike 功能，将命令转换为数字索引防止外部黑客攻击

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 隐藏命令数组大小 - 每个用户可用的最大命令索引数
/// 足够大的值确保单个页面不会出现重复的命令索引
const HIDDEN_SIZE: usize = 100000;

/// 用户的隐藏命令状态
#[derive(Debug)]
struct UserHiddenCommands {
    /// 命令数组
    commands: Vec<Option<String>>,
    /// 当前位置
    position: usize,
}

impl UserHiddenCommands {
    fn new() -> Self {
        Self {
            commands: vec![None; HIDDEN_SIZE],
            position: 0,
        }
    }

    /// 隐藏命令，返回索引
    fn hide(&mut self, cmd: String) -> String {
        let pos = self.position;
        if pos >= HIDDEN_SIZE {
            self.position = 0;
        } else {
            self.position = pos + 1;
        }

        self.commands[pos] = Some(cmd);
        pos.to_string()
    }

    /// 取消隐藏命令，从索引获取实际命令
    fn unhide(&self, index_str: &str) -> Option<String> {
        let index: usize = index_str.trim().parse().ok()?;
        if index >= HIDDEN_SIZE {
            return None;
        }
        self.commands.get(index)?.clone()
    }
}

/// 全局命令隐藏管理器
pub struct HiddenCommandManager {
    /// 每个用户的隐藏命令映射
    users: HashMap<String, UserHiddenCommands>,
}

impl HiddenCommandManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// 隐藏命令：将明文命令存储，返回数字索引
    pub fn hide_command(&mut self, userid: &str, cmd: &str) -> String {
        if userid.is_empty() || cmd.is_empty() {
            return "0".to_string();
        }

        let user_commands = self.users
            .entry(userid.to_string())
            .or_insert_with(UserHiddenCommands::new);

        user_commands.hide(cmd.to_string())
    }

    /// 解码命令：将数字索引转换为实际命令
    /// 如果索引无效，返回默认命令 "look"
    pub fn unhide_command(&self, userid: &str, index_str: &str) -> String {
        if userid.is_empty() || index_str.is_empty() {
            return "look".to_string();
        }

        // 如果不是纯数字，直接返回原字符串
        if !index_str.trim().chars().all(|c| c.is_ascii_digit()) {
            return index_str.trim().to_string();
        }

        if let Some(user_commands) = self.users.get(userid) {
            if let Some(cmd) = user_commands.unhide(index_str) {
                return cmd;
            }
        }

        // 默认返回 look 命令
        "look".to_string()
    }

    /// 清理用户的隐藏命令缓存（可选，用于释放内存）
    pub fn clear_user(&mut self, userid: &str) {
        self.users.remove(userid);
    }

    /// 获取当前用户数量
    pub fn user_count(&self) -> usize {
        self.users.len()
    }
}

impl Default for HiddenCommandManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局隐藏命令管理器
static GLOBAL_HIDDEN: std::sync::OnceLock<Arc<RwLock<HiddenCommandManager>>> =
    std::sync::OnceLock::new();

/// 获取全局隐藏命令管理器
pub fn get_hidden_manager() -> Arc<RwLock<HiddenCommandManager>> {
    GLOBAL_HIDDEN.get_or_init(|| {
        Arc::new(RwLock::new(HiddenCommandManager::new()))
    }).clone()
}

/// 便捷函数：隐藏命令
pub async fn hide_command(userid: &str, cmd: &str) -> String {
    let manager = get_hidden_manager();
    let mut mgr = manager.write().await;
    mgr.hide_command(userid, cmd)
}

/// 便捷函数：解码命令
pub async fn unhide_command(userid: &str, index_str: &str) -> String {
    let manager = get_hidden_manager();
    let mgr = manager.read().await;
    mgr.unhide_command(userid, index_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hide_unhide() {
        let mut mgr = HiddenCommandManager::new();

        // 测试隐藏和恢复命令
        let index1 = mgr.hide_command("user1", "look");
        let index2 = mgr.hide_command("user1", "north");
        let index3 = mgr.hide_command("user1", "south");

        assert_eq!(mgr.unhide_command("user1", &index1), "look");
        assert_eq!(mgr.unhide_command("user1", &index2), "north");
        assert_eq!(mgr.unhide_command("user1", &index3), "south");
    }

    #[tokio::test]
    async fn test_invalid_index_returns_look() {
        let mgr = HiddenCommandManager::new();
        assert_eq!(mgr.unhide_command("user1", "99999"), "look");
        assert_eq!(mgr.unhide_command("user1", "-1"), "look");
    }

    #[tokio::test]
    async fn test_non_numeric_returns_input() {
        let mgr = HiddenCommandManager::new();
        assert_eq!(mgr.unhide_command("user1", "look"), "look");
        assert_eq!(mgr.unhide_command("user1", "north"), "north");
    }

    #[tokio::test]
    async fn test_wrap_around() {
        let mut mgr = HiddenCommandManager::new();

        // 填充整个数组
        for i in 0..HIDDEN_SIZE {
            mgr.hide_command("user1", &format!("cmd{}", i));
        }

        // 下一个应该回到位置0
        let index = mgr.hide_command("user1", "wrapped");
        assert_eq!(index, "0");
        assert_eq!(mgr.unhide_command("user1", &index), "wrapped");
    }
}
