// gamenv/cmds/skills.rs - 技能命令
// 对应 txpike9/wapmud2/cmds/ 目录中的技能相关命令

use crate::gamenv::single::daemons::pkd::PKD;

/// 查看技能列表命令
pub async fn skills_command(userid: &str) -> String {
    match PKD.get_player_battle(userid).await {
        Some(battle) => battle.generate_skills_list(userid),
        None => "你不在战斗中！\n[返回:look]".to_string(),
    }
}
