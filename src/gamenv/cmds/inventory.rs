// gamenv/cmds/inventory.rs - 背包命令
// 对应 txpike9/wapmud2/cmds/ 目录中的背包相关命令

use crate::gamenv::player_state::PlayerState;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

/// 查看背包命令
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
