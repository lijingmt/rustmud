// gamenv/cmds/learn.rs - 学习技能和拜师命令
// 对应 txpike9/wapmud2/cmds/ 目录中的学习相关命令

use std::sync::Arc;
use async_trait::async_trait;
use crate::gamenv::core::command::*;
use crate::gamenv::single::skills::{get_enhanced_skilld, PlayerStats};
use crate::gamenv::single::masters::{get_masterd, StatRequirements as MasterStatReq};

/// Learn命令 - 学习技能
pub struct LearnCommand;

#[async_trait]
impl CommandHandler for LearnCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        if ctx.args.is_empty() {
            return CommandResult::from(show_learn_help());
        }

        let skill_id = &ctx.args[0];

        // 获取技能管理器
        let skill_mgr = get_enhanced_skilld();
        let mut skill_mgr = skill_mgr.write().await;

        // TODO: 从玩家对象获取实际属性
        let player_stats = PlayerStats {
            gen: 20,
            str: 25,
            con: 20,
            dex: 15,
            int: 15,
        };

        match skill_mgr.learn_skill(ctx.player_id.clone(), skill_id.clone(), &player_stats) {
            Ok(msg) => CommandResult::from(format!("§g{}§N\n", msg)),
            Err(e) => CommandResult::from(format!("§c{}§N\n", e)),
        }
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "learn",
                    "学习技能",
                    CommandCategory::Interaction
                )
                .with_aliases(&["学", "xue"])
                .with_args(1, Some(1))
            });
        &META
    }
}

/// Baishi命令 - 拜师
pub struct BaishiCommand;

#[async_trait]
impl CommandHandler for BaishiCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        if ctx.args.is_empty() {
            return CommandResult::from(show_baishi_help());
        }

        let master_id = &ctx.args[0];

        // 获取师父管理器
        let master_mgr = get_masterd();
        let mut master_mgr = master_mgr.write().await;

        // TODO: 从玩家对象获取实际属性和等级
        let player_stats = MasterStatReq {
            gen: Some(20),
            str: Some(25),
            con: Some(20),
            dex: Some(15),
            int: Some(15),
        };
        let player_level = 10;

        match master_mgr.apprentice_to(&ctx.player_id, master_id, &player_stats, player_level) {
            Ok(msg) => CommandResult::from(format!("§g{}§N\n{}", msg, show_next_steps())),
            Err(e) => CommandResult::from(format!("§c{}§N\n", e)),
        }
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "baishi",
                    "拜师",
                    CommandCategory::Interaction
                )
                .with_aliases(&["拜师", "apprentice"])
                .with_args(1, Some(1))
            });
        &META
    }
}

/// Enable命令 - 启用武功映射
pub struct EnableCommand;

#[async_trait]
impl CommandHandler for EnableCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        if ctx.args.len() < 2 {
            return CommandResult::from("§c用法: enable <基础技能> <特殊武功>§N\n例如: enable unarmed xionghuquan\n");
        }

        let basic_skill = &ctx.args[0];
        let special_skill = &ctx.args[1];

        // 获取技能管理器
        let skill_mgr = get_enhanced_skilld();
        let mut skill_mgr = skill_mgr.write().await;

        match skill_mgr.enable_skill(&ctx.player_id, basic_skill, special_skill) {
            Ok(msg) => CommandResult::from(format!("§g{}§N\n", msg)),
            Err(e) => CommandResult::from(format!("§c{}§N\n", e)),
        }
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "enable",
                    "启用武功",
                    CommandCategory::Interaction
                )
                .with_aliases(&["启用", "特殊武功"])
                .with_args(2, Some(2))
            });
        &META
    }
}

/// Menpai命令 - 查看门派信息
pub struct MenpaiCommand;

#[async_trait]
impl CommandHandler for MenpaiCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        let master_mgr = get_masterd();
        let master_mgr = master_mgr.read().await;

        // 检查玩家是否已拜师
        let output = if let Some(rel) = master_mgr.get_apprenticeship(&ctx.player_id) {
            format!(
                "§Y【你的门派】§N\n\
                门派: §g{}§N\n\
                师父: §g{}§N\n\
                职位: §g{}§N\n\
                贡献: §g{}§N\n\
                拜师时间: {}",
                rel.school_id,
                rel.master_id,
                rel.rank.cn_name(),
                rel.contribution,
                rel.since
            )
        } else {
            show_menpai_list(&master_mgr)
        };

        CommandResult::from(output)
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "menpai",
                    "门派信息",
                    CommandCategory::Info
                )
                .with_aliases(&["门派", "school", "gang"])
            });
        &META
    }
}

/// 显示拜师帮助
fn show_baishi_help() -> String {
    let mut output = String::from("§Y【拜师系统】§N\n\n");
    output.push_str("§H可拜的门派师父§N\n");
    output.push_str("─────────────────\n");
    output.push_str("[拜武堂堂主:baishi wutang] - 需要臂力20，等级10\n");
    output.push_str("[拜武当掌门:baishi wudang] - 需要根骨25，等级10\n");
    output.push_str("[拜少林方丈:baishi shaolin] - 需要臂力25体质15，等级10\n");
    output.push_str("[拜华山掌门:baishi huashan] - 需要悟性30灵巧25，等级20\n\n");
    output.push_str("§c提示: 拜师后才能学习门派武功！§N\n");
    output
}

/// 显示门派列表
fn show_menpai_list(master_mgr: &crate::gamenv::single::masters::MasterDaemon) -> String {
    let mut output = String::from("§Y【江湖门派】§N\n\n");
    output.push_str("§H各大门派§N\n");
    output.push_str("─────────────────\n\n");

    output.push_str("§g【武堂】§N - 特殊门派，武功化繁为简\n");
    output.push_str("可学: 猛虎拳\n");
    output.push_str("[拜师:baishi wutang]\n\n");

    output.push_str("§g【武当】§N - 内家武学，以柔克刚\n");
    output.push_str("可学: 太极剑\n");
    output.push_str("[拜师:baishi wudang]\n\n");

    output.push_str("§g【少林】§N - 少林七十二绝技\n");
    output.push_str("可学: 罗汉拳\n");
    output.push_str("[拜师:baishi shaolin]\n\n");

    output.push_str("§g【华山】§N - 剑法第一\n");
    output.push_str("可学: 独孤九剑\n");
    output.push_str("[拜师:baishi huashan]\n\n");

    output.push_str("§c提示: 选择一个门派拜师后，才能学习该门派的武功！§N\n");
    output
}

/// 显示学习帮助
fn show_learn_help() -> String {
    let mut output = String::from("§Y【技能学习】§N\n\n");
    output.push_str("§H基础技能§N\n");
    output.push_str("[学习基础拳脚:learn unarmed_basic]\n\n");

    output.push_str("§H门派武功§N (需要先拜师)\n");
    output.push_str("[学习猛虎拳:learn xionghuquan] - 武堂独门拳法\n");
    output.push_str("[学习太极剑:learn taiji] - 武当镇派剑法\n");
    output.push_str("[学习罗汉拳:learn luohanquan] - 少林七十二绝技\n");
    output.push_str("[学习独孤九剑:learn dugujiujian] - 华山派镇派绝学\n\n");

    output.push_str("§H启用武功§N\n");
    output.push_str("用法: enable <基础技能> <特殊武功>\n");
    output.push_str("例如: [enable unarmed xionghuquan]\n");
    output.push_str("启用后，有效等级 = 基础技能/2 + 特殊武功\n\n");

    output.push_str("§c提示: 先用 [baishi 门派] 拜师，再学习门派武功！§N\n");
    output
}

/// 显示下一步操作提示
fn show_next_steps() -> String {
    "§Y【下一步】§N\n\
    [学习武功:learn <技能名>]\n\
    [查看门派:menpai]\n\
    [启用武功:enable <基础> <特殊>]\n".to_string()
}

/// 获取Learn命令实例
pub fn get_learn_command() -> Arc<dyn CommandHandler> {
    Arc::new(LearnCommand) as Arc<dyn CommandHandler>
}

/// 获取Baishi命令实例
pub fn get_baishi_command() -> Arc<dyn CommandHandler> {
    Arc::new(BaishiCommand) as Arc<dyn CommandHandler>
}

/// 获取Enable命令实例
pub fn get_enable_command() -> Arc<dyn CommandHandler> {
    Arc::new(EnableCommand) as Arc<dyn CommandHandler>
}

/// 获取Menpai命令实例
pub fn get_menpai_command() -> Arc<dyn CommandHandler> {
    Arc::new(MenpaiCommand) as Arc<dyn CommandHandler>
}
