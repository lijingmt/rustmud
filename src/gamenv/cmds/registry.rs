// gamenv/cmds/registry.rs - 命令注册表
// 对应 txpike9 的命令文件名自动映射机制

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::gamenv::core::command::*;

/// 命令注册表（线程安全单例）
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn CommandHandler>>,
    aliases: HashMap<String, String>, // alias -> canonical name
    categories: HashMap<CommandCategory, Vec<String>>,
}

impl CommandRegistry {
    /// 创建新的命令注册表
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// 注册命令
    pub fn register(&mut self, handler: Arc<dyn CommandHandler>) {
        let metadata = handler.metadata();
        let name = metadata.name.clone();

        // 注册别名
        for alias in &metadata.aliases {
            self.aliases.insert(alias.clone(), name.clone());
        }

        // 按类别索引
        self.categories
            .entry(metadata.category)
            .or_insert_with(Vec::new)
            .push(name.clone());

        self.commands.insert(name, handler);
    }

    /// 批量注册命令
    pub fn register_all(&mut self, handlers: Vec<Arc<dyn CommandHandler>>) {
        for handler in handlers {
            self.register(handler);
        }
    }

    /// 分发命令到对应的处理器
    pub async fn dispatch(&self, cmd_str: &str, ctx: CommandContext) -> CommandResult {
        // 解析命令名和参数
        let parts: Vec<&str> = cmd_str.trim().split_whitespace().collect();
        if parts.is_empty() {
            return CommandResult::from("请输入命令。");
        }

        let cmd_name = self.resolve_alias(parts[0]);

        // 获取命令处理器
        let handler = match self.commands.get(&cmd_name) {
            Some(h) => h,
            None => return CommandResult::from(format!("未知命令: {}", parts[0])),
        };

        // 验证参数数量
        let metadata = handler.metadata();
        if let Err(e) = metadata.validate_args(parts.len() - 1) {
            return CommandResult::from(format!("命令错误: {}", e));
        }

        // 执行命令
        handler.handle(ctx).await
    }

    /// 解析别名到标准命令名
    pub fn resolve_alias(&self, name: &str) -> String {
        self.aliases
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }

    /// 获取命令元数据
    pub fn get_metadata(&self, name: &str) -> Option<CommandMetadata> {
        let resolved = self.resolve_alias(name);
        self.commands.get(&resolved)
            .map(|h| h.metadata().clone())
    }

    /// 列出所有命令
    pub fn list_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// 按类别列出命令
    pub fn list_by_category(&self, category: CommandCategory) -> Vec<String> {
        self.categories
            .get(&category)
            .cloned()
            .unwrap_or_default()
    }
}

/// 全局命令注册表单例（Lazy 初始化）
static REGISTRY: once_cell::sync::Lazy<Arc<RwLock<CommandRegistry>>> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(RwLock::new(CommandRegistry::new()))
    });

/// 获取全局命令注册表
pub fn get_registry() -> Arc<RwLock<CommandRegistry>> {
    REGISTRY.clone()
}

/// 初始化命令注册表（在启动时调用）
pub async fn init_registry() -> Arc<RwLock<CommandRegistry>> {
    let registry = get_registry();

    // 注册所有命令
    let mut reg = registry.write().await;

    // 导入各个命令模块并注册
    reg.register_all(vec![
        crate::gamenv::cmds::look::get_command(),
        crate::gamenv::cmds::inventory::get_command(),
        crate::gamenv::cmds::skills::get_command(),
        crate::gamenv::cmds::skills::get_cast_command(),
        crate::gamenv::cmds::pk::get_command(),
        crate::gamenv::cmds::move_dir::get_command(),
    ]);

    registry.clone()
}

/// 辅助宏：创建带元数据的命令
///
/// 使用 async_trait 和 Lazy 初始化，简化命令定义
#[macro_export]
macro_rules! command {
    (
        name: $name:expr,
        aliases: [$($alias:expr),*],
        description: $desc:expr,
        category: $category:expr,
        min_args: $min:expr,
        max_args: $max:expr,
        handler: |$ctx:ident| $body:expr
    ) => {
        {
            struct CommandImpl;

            #[async_trait::async_trait]
            impl $crate::gamenv::core::command::CommandHandler for CommandImpl {
                async fn handle(
                    &self,
                    $ctx: $crate::gamenv::core::command::CommandContext,
                ) -> $crate::gamenv::core::command::CommandResult {
                    $body
                }

                fn metadata(&self) -> &$crate::gamenv::core::command::CommandMetadata {
                    use once_cell::sync::Lazy;
                    static META: Lazy<$crate::gamenv::core::command::CommandMetadata> = Lazy::new(|| {
                        $crate::gamenv::core::command::CommandMetadata::new($name, $desc, $category)
                            .with_aliases(&[$($alias),*])
                            .with_args($min, $max)
                    });
                    &META
                }
            }

            std::sync::Arc::new(CommandImpl) as std::sync::Arc<dyn $crate::gamenv::core::command::CommandHandler>
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_basic() {
        let mut registry = CommandRegistry::new();

        // 添加测试命令
        let cmd = command! {
            name: "test",
            aliases: ["t"],
            description: "Test command",
            category: CommandCategory::System,
            min_args: 0,
            max_args: None,
            handler: |_ctx| CommandResult::from("OK")
        };

        registry.register(cmd);

        // 测试解析
        assert_eq!(registry.resolve_alias("test"), "test");
        assert_eq!(registry.resolve_alias("t"), "test");
    }
}
