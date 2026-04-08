// gamenv/cmds.rs - 命令系统
// 对应 txpike9/gamenv/cmds/ 目录

use crate::core::*;
use crate::pikenv::efuns::EFUNSD;
use async_trait::async_trait;

/// 命令 Trait
#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, args: &str, user: &GObject) -> Result<Value>;
}

/// 命令管理器
pub struct CommandManager {
    commands: std::collections::HashMap<String, std::sync::Arc<dyn Command>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: std::collections::HashMap::new(),
        }
    }

    /// 注册命令
    pub fn register(&mut self, cmd: std::sync::Arc<dyn Command>) {
        self.commands.insert(cmd.name().to_string(), cmd);
    }

    /// 执行命令
    pub async fn execute(&self, cmd_str: &str, user: &GObject) -> Result<Value> {
        let parts: Vec<&str> = cmd_str.splitn(2, ' ').collect();
        let cmd_name = parts[0];
        let args = parts.get(1).unwrap_or(&"");

        if let Some(cmd) = self.commands.get(cmd_name) {
            cmd.execute(args, user).await
        } else {
            Err(MudError::CommandNotFound(cmd_name.to_string()))
        }
    }
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new()
    }
}

// ========== 基础命令实现 ==========

/// Look 命令 (对应 look)
pub struct LookCommand;

#[async_trait]
impl Command for LookCommand {
    fn name(&self) -> &str {
        "look"
    }

    async fn execute(&self, _args: &str, _user: &GObject) -> Result<Value> {
        Ok(Value::String("你环顾四周...".to_string()))
    }
}

/// Help 命令 (对应 help)
pub struct HelpCommand;

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    async fn execute(&self, _args: &str, _user: &GObject) -> Result<Value> {
        Ok(Value::String("可用命令: look, help, say, inventory...".to_string()))
    }
}

/// Say 命令 (对应 say)
pub struct SayCommand;

#[async_trait]
impl Command for SayCommand {
    fn name(&self) -> &str {
        "say"
    }

    async fn execute(&self, args: &str, _user: &GObject) -> Result<Value> {
        let message = format!("你说: {}\n", args);
        Ok(Value::String(message))
    }
}
