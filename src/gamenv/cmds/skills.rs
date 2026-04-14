// gamenv/cmds/skills.rs - 技能命令
// 对应 txpike9/wapmud2/cmds/ 目录中的技能相关命令

use std::sync::Arc;
use async_trait::async_trait;

use crate::gamenv::single::daemons::pkd::PKD;
use crate::gamenv::core::command::*;

/// 查看技能列表命令（旧接口）
pub async fn skills_command(userid: &str) -> String {
    match PKD.get_player_battle(userid).await {
        Some(battle) => battle.generate_skills_list(userid),
        None => "你不在战斗中！\n[返回:look]".to_string(),
    }
}

/// Skills 命令处理器（新命令系统）
pub struct SkillsCommand;

#[async_trait]
impl CommandHandler for SkillsCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        let output = skills_command(&ctx.player_id).await;
        CommandResult::from(output)
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "skills",
                    "查看技能列表",
                    CommandCategory::Info
                )
                .with_aliases(&["skill", "cha"])
                .with_args(0, None)
            });
        &META
    }
}

/// 获取 Skills 命令实例
pub fn get_command() -> Arc<dyn CommandHandler> {
    Arc::new(SkillsCommand) as Arc<dyn CommandHandler>
}
