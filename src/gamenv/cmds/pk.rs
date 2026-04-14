// gamenv/cmds/pk.rs - PK战斗命令
// 对应 txpike9/wapmud2/cmds/ 目录中的PK相关命令

use std::sync::Arc;
use async_trait::async_trait;

use crate::gamenv::core::command::*;

// 注意：完整的PK命令处理需要访问世界数据来查找目标
// 这些命令应该在HTTP API层处理，因为需要访问world和player_state
// 这个文件作为命令处理的接口定义

/// PK命令接口（旧接口）
pub async fn pk_command(_userid: &str, _target: &str) -> String {
    // 实际实现在 HTTP API 模块中
    "PK命令处理\n".to_string()
}

/// PK继续命令接口（旧接口）
pub async fn pk_continue_command(_userid: &str) -> String {
    // 实际实现在 HTTP API 模块中
    "PK继续\n".to_string()
}

/// 逃跑命令接口（旧接口）
pub async fn escape_command(_userid: &str) -> String {
    "逃跑\n".to_string()
}

/// 投降命令接口（旧接口）
pub async fn surrender_command(_userid: &str) -> String {
    "投降\n".to_string()
}

/// PK 命令处理器（新命令系统）
pub struct PkCommand;

#[async_trait]
impl CommandHandler for PkCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        if ctx.args.is_empty() {
            return CommandResult::from("请指定PK目标。");
        }

        let target = ctx.joined_args();
        let output = pk_command(&ctx.player_id, &target).await;
        CommandResult::from(output)
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "pk",
                    "与指定目标进行PK",
                    CommandCategory::Combat
                )
                .with_args(1, None)
                .requires_target()
            });
        &META
    }
}

/// 获取 PK 命令实例
pub fn get_command() -> Arc<dyn CommandHandler> {
    Arc::new(PkCommand) as Arc<dyn CommandHandler>
}
