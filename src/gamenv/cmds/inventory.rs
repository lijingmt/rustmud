// gamenv/cmds/inventory.rs - 背包命令
// 对应 txpike9/wapmud2/cmds/ 目录中的背包相关命令

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock as TokioRwLock;

use crate::gamenv::player_state::PlayerState;
use crate::gamenv::core::command::*;
use crate::gamenv::player_state;

/// 查看背包命令（旧接口，保持兼容）
pub async fn inventory_command(state: &Arc<TokioRwLock<PlayerState>>, args: &[&str]) -> String {
    let player = state.read().await;

    // 如果有参数，查看具体物品
    if !args.is_empty() {
        let item_type = args[0];
        match item_type {
            "yao" => "你的药囊里有：\n  §W金创药§N - 恢复100点生命\n".to_string(),
            "wu" => "你的武器：\n  §W木剑§N - 攻击+5\n".to_string(),
            _ => format!("不支持的背包类型: {}\n", item_type),
        }
    } else {
        // 显示背包分类
        "你的背包：\n§H分类§N\n[查看药品:inventory yao]\n[查看武器:inventory wu]\n[查看防具:inventory fang]\n".to_string()
    }
}

/// Inventory 命令处理器（新命令系统）
pub struct InventoryCommand;

#[async_trait]
impl CommandHandler for InventoryCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        let player_mgr = player_state::get_player_manager();
        let mgr = player_mgr.read().await;

        if let Some(player) = mgr.get(&ctx.player_id).await {
            // 将 Vec<String> 转换为 Vec<&str>
            let args: Vec<&str> = ctx.args.iter().map(|s| s.as_str()).collect();
            let output = inventory_command(&player, &args).await;
            CommandResult::from(output)
        } else {
            CommandResult::from("找不到玩家状态。")
        }
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "inventory",
                    "查看背包物品",
                    CommandCategory::Info
                )
                .with_aliases(&["i", "inv", "bag"])
                .with_args(0, None)
            });
        &META
    }
}

/// 获取 Inventory 命令实例
pub fn get_command() -> Arc<dyn CommandHandler> {
    Arc::new(InventoryCommand) as Arc<dyn CommandHandler>
}
