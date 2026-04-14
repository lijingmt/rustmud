// gamenv/core/command.rs - 命令抽象
// 所有命令处理的基础接口

use async_trait::async_trait;

/// 命令执行上下文
#[derive(Clone, Debug)]
pub struct CommandContext {
    pub player_id: String,
    pub room_id: String,
    pub args: Vec<String>,
    pub raw_command: String,
}

impl CommandContext {
    pub fn new(player_id: &str, room_id: &str, raw_command: &str) -> Self {
        let parts: Vec<String> = raw_command
            .trim()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Self {
            player_id: player_id.to_string(),
            room_id: room_id.to_string(),
            args: parts.get(1..).unwrap_or(&[]).to_vec(),
            raw_command: raw_command.to_string(),
        }
    }

    pub fn arg(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(|s| s.as_str())
    }

    pub fn arg_or(&self, index: usize, default: &str) -> String {
        self.arg(index).unwrap_or(default).to_string()
    }

    pub fn joined_args(&self) -> String {
        self.args.join(" ")
    }
}

/// 命令执行结果
#[derive(Clone, Debug)]
pub struct CommandResult {
    pub output: String,
    pub should_update_room: bool,
    pub events: Vec<GameEvent>,
}

impl CommandResult {
    pub fn simple(output: &str) -> Self {
        Self {
            output: output.to_string(),
            should_update_room: false,
            events: vec![],
        }
    }

    pub fn with_room_update(output: &str) -> Self {
        Self {
            output: output.to_string(),
            should_update_room: true,
            events: vec![],
        }
    }
}

impl From<String> for CommandResult {
    fn from(output: String) -> Self {
        Self {
            output,
            should_update_room: false,
            events: vec![],
        }
    }
}

impl From<&str> for CommandResult {
    fn from(output: &str) -> Self {
        Self {
            output: output.to_string(),
            should_update_room: false,
            events: vec![],
        }
    }
}

/// 游戏事件
#[derive(Clone, Debug)]
pub enum GameEvent {
    CombatStart { player_id: String, target_id: String },
    CombatEnd { winner: String, loser: String },
    ItemPickup { player_id: String, item_id: String },
    ItemDrop { player_id: String, item_id: String },
    LevelUp { player_id: String, new_level: i32 },
    Death { entity_id: String },
    Chat { sender: String, message: String, channel: ChatChannel },
}

#[derive(Clone, Debug)]
pub enum ChatChannel {
    Say,
    Shout,
    Tell,
    System,
}

/// 命令处理器trait - 使用 async_trait 支持 async fn
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// 处理命令（async 方法）
    async fn handle(&self, ctx: CommandContext) -> CommandResult;

    /// 获取命令元数据
    fn metadata(&self) -> &CommandMetadata;
}

/// 命令元数据
#[derive(Clone, Debug)]
pub struct CommandMetadata {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub category: CommandCategory,
    pub min_args: usize,
    pub max_args: Option<usize>,
    pub requires_target: bool,
}

impl CommandMetadata {
    pub fn new(name: &str, description: &str, category: CommandCategory) -> Self {
        Self {
            name: name.to_string(),
            aliases: vec![],
            description: description.to_string(),
            category,
            min_args: 0,
            max_args: None,
            requires_target: false,
        }
    }

    pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_args(mut self, min: usize, max: Option<usize>) -> Self {
        self.min_args = min;
        self.max_args = max;
        self
    }

    pub fn requires_target(mut self) -> Self {
        self.requires_target = true;
        self
    }

    /// 检查参数数量是否有效
    pub fn validate_args(&self, arg_count: usize) -> Result<(), CommandError> {
        if arg_count < self.min_args {
            return Err(CommandError::TooFewArguments {
                expected: self.min_args,
                got: arg_count,
            });
        }

        if let Some(max) = self.max_args {
            if arg_count > max {
                return Err(CommandError::TooManyArguments {
                    expected: max,
                    got: arg_count,
                });
            }
        }

        Ok(())
    }
}

/// 命令类别
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Movement,
    Interaction,
    Combat,
    Social,
    System,
    Info,
    Admin,
}

/// 命令错误
#[derive(Clone, Debug)]
pub enum CommandError {
    UnknownCommand(String),
    TooFewArguments { expected: usize, got: usize },
    TooManyArguments { expected: usize, got: usize },
    InvalidTarget(String),
    PermissionDenied,
    Cooldown,
    NotImplemented,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::UnknownCommand(cmd) => write!(f, "未知命令: {}", cmd),
            CommandError::TooFewArguments { expected, got } => {
                write!(f, "参数不足：需要 {} 个，得到 {} 个", expected, got)
            }
            CommandError::TooManyArguments { expected, got } => {
                write!(f, "参数过多：最多 {} 个，得到 {} 个", expected, got)
            }
            CommandError::InvalidTarget(target) => write!(f, "无效的目标: {}", target),
            CommandError::PermissionDenied => write!(f, "权限不足"),
            CommandError::Cooldown => write!(f, "命令冷却中"),
            CommandError::NotImplemented => write!(f, "功能尚未实现"),
        }
    }
}

/// 简单命令处理器宏 - 用于快速实现简单命令
#[macro_export]
macro_rules! simple_command {
    ($name:expr, $desc:expr, $category:expr, $handler:expr) => {{
        struct SimpleCommand;

        #[async_trait::async_trait]
        impl $crate::gamenv::core::command::CommandHandler for SimpleCommand {
            async fn handle(
                &self,
                ctx: $crate::gamenv::core::command::CommandContext,
            ) -> $crate::gamenv::core::command::CommandResult {
                $handler(ctx).await
            }

            fn metadata(&self) -> &$crate::gamenv::core::command::CommandMetadata {
                static META: $crate::gamenv::core::command::CommandMetadata =
                    $crate::gamenv::core::command::CommandMetadata::new($name, $desc, $category);
                &META
            }
        }

        std::sync::Arc::new(SimpleCommand) as std::sync::Arc<dyn CommandHandler>
    }};
}
