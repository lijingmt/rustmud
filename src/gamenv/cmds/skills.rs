// gamenv/cmds/skills.rs - 技能命令
// 对应 txpike9/wapmud2/cmds/ 目录中的技能相关命令

use std::sync::Arc;
use async_trait::async_trait;

use crate::gamenv::single::daemons::pkd::PKD;
use crate::gamenv::core::command::*;
use crate::gamenv::combat::skill::{SKILLD, LearnedSkill};

/// 技能命令模式
enum SkillsMode {
    /// 查看所有技能
    All,
    /// 查看战斗技能
    Battle,
    /// 查看已学技能详情
    Detail(String),
}

/// 查看技能列表命令
pub async fn skills_command(userid: &str, args: &[&str]) -> String {
    // 如果在战斗中，显示战斗技能
    if let Some(battle) = PKD.get_player_battle(userid).await {
        return battle.generate_skills_list(userid);
    }

    // 不在战斗中，显示所有已学技能
    let mode = if args.is_empty() {
        SkillsMode::All
    } else if args.len() == 1 {
        SkillsMode::Detail(args[0].to_string())
    } else {
        return "§c用法: skills [技能ID]§N\n".to_string();
    };

    match mode {
        SkillsMode::All => show_all_skills(),
        SkillsMode::Detail(skill_id) => show_skill_detail(&skill_id),
        SkillsMode::Battle => "§c你不在战斗中！§N\n".to_string(),
    }
}

/// 显示所有可用技能
fn show_all_skills() -> String {
    let skill_mgr = SKILLD.lock().unwrap();
    let mut output = String::from("§Y【武功技能】§N\n\n");
    output.push_str("§H基础技能§N\n");
    output.push_str("─────────────────\n");

    let basic_skills = [
        ("skill_basic_attack", "基础攻击", "1", "0"),
        ("skill_xionghuquan", "猛虎拳", "10", "15"),
        ("skill_taiji", "太极剑", "10", "10"),
        ("skill_luohanquan", "罗汉拳", "10", "15"),
        ("skill_dugujiujian", "独孤九剑", "20", "30"),
        ("skill_power_strike", "强力一击", "5", "20"),
        ("skill_quick_strike", "快速连击", "3", "15"),
    ];

    for (skill_id, name, level, cost) in basic_skills {
        if let Some(skill) = skill_mgr.get_skill(skill_id) {
            let cd_text = if skill.current_cooldown > 0 {
                format!(" §c(冷却:{})§N", skill.current_cooldown)
            } else {
                String::new()
            };
            output.push_str(&format!(
                "§g[{}:cast {}]§N {} - Lv.{} 消耗内力:{}{}\n",
                name, skill_id, name, level, cost, cd_text
            ));
        }
    }

    output.push_str("\n§H辅助技能§N\n");
    output.push_str("─────────────────\n");

    let support_skills = [
        ("skill_heal", "治疗术", "10", "30"),
        ("skill_defense", "防御姿态", "3", "10"),
    ];

    for (skill_id, name, level, cost) in support_skills {
        if let Some(skill) = skill_mgr.get_skill(skill_id) {
            let cd_text = if skill.current_cooldown > 0 {
                format!(" §c(冷却:{})§N", skill.current_cooldown)
            } else {
                String::new()
            };
            output.push_str(&format!(
                "§g[{}:cast {}]§N {} - Lv.{} 消耗内力:{}{}\n",
                name, skill_id, name, level, cost, cd_text
            ));
        }
    }

    output.push_str("\n§c提示: 在战斗中使用 'cast <技能ID>' 释放技能§N\n");
    output.push_str("[查看技能详情:skills <技能ID>]\n");

    output
}

/// 显示技能详情
fn show_skill_detail(skill_id: &str) -> String {
    let skill_mgr = SKILLD.lock().unwrap();

    if let Some(skill) = skill_mgr.get_skill(skill_id) {
        let mut output = format!("§Y【{}】§N\n\n", skill.name_cn);
        output.push_str(&skill.render_info());
        output.push_str("\n[返回:skills]\n");
        output
    } else {
        format!("§c找不到技能: {}§N\n[返回:skills]\n", skill_id)
    }
}

/// 学习技能命令
pub async fn learn_skill_command(userid: &str, skill_id: &str) -> String {
    // 检查技能是否存在
    let (skill_exists, skill_name) = {
        let skill_mgr = SKILLD.lock().unwrap();
        if let Some(skill) = skill_mgr.get_skill(skill_id) {
            (true, skill.name_cn.clone())
        } else {
            (false, String::new())
        }
    };

    if !skill_exists {
        return format!("§c找不到技能: {}§N\n", skill_id);
    }

    // TODO: 检查玩家是否满足学习条件
    // TODO: 添加到玩家的已学技能列表

    format!("§g你学习了 {}！§N\n[继续:skills]\n", skill_name)
}

/// Skills 命令处理器（新命令系统）
pub struct SkillsCommand;

#[async_trait]
impl CommandHandler for SkillsCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        let args: Vec<&str> = ctx.args.iter().map(|s| s.as_str()).collect();
        let output = skills_command(&ctx.player_id, &args).await;
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
                .with_aliases(&["skill", "cha", "武功"])
                .with_args(0, Some(1))
            });
        &META
    }
}

/// Cast 命令处理器 - 在战斗中使用技能
pub struct CastCommand;

#[async_trait]
impl CommandHandler for CastCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        if ctx.args.is_empty() {
            return CommandResult::from("§c用法: cast <技能ID>§N\n");
        }

        let skill_id = &ctx.args[0];
        let userid = ctx.player_id.clone();

        // 检查玩家是否在战斗中
        if PKD.get_player_battle(&userid).await.is_none() {
            return CommandResult::from("§c你不在战斗中！§N\n");
        }

        // 选择技能
        match PKD.select_skill(&userid, skill_id).await {
            Ok(msg) => {
                // 显示成功消息，战斗会通过heartbeat自动继续
                CommandResult::from(msg)
            }
            Err(e) => CommandResult::from(format!("§c{}§N\n", e)),
        }
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "cast",
                    "施展技能",
                    CommandCategory::Combat
                )
                .with_args(1, Some(1))
            });
        &META
    }
}

/// 获取 Skills 命令实例
pub fn get_command() -> Arc<dyn CommandHandler> {
    Arc::new(SkillsCommand) as Arc<dyn CommandHandler>
}

/// 获取 Cast 命令实例
pub fn get_cast_command() -> Arc<dyn CommandHandler> {
    Arc::new(CastCommand) as Arc<dyn CommandHandler>
}
