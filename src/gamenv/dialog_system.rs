// gamenv/dialog_system.rs - NPC对话系统
// 对应 txpike9 的 NPC 对话功能

use crate::gamenv::world::{Npc, DialogNode, DialogOption, DialogAction};
use crate::gamenv::player_state::PlayerState;
use crate::gamenv::world::ItemTemplate;
use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 对话会话
#[derive(Clone, Debug)]
pub struct DialogSession {
    /// NPC ID
    pub npc_id: String,
    /// 当前对话节点ID
    pub current_node: String,
    /// 对话历史
    pub history: Vec<String>,
    /// 会话开始时间
    pub start_time: i64,
}

impl DialogSession {
    /// 创建新的对话会话
    pub fn new(npc_id: String, start_node: String) -> Self {
        Self {
            npc_id,
            current_node: start_node,
            history: Vec::new(),
            start_time: chrono::Utc::now().timestamp(),
        }
    }

    /// 添加到历史记录
    pub fn add_history(&mut self, text: String) {
        self.history.push(text);
    }

    /// 格式化对话历史
    pub fn format_history(&self) -> String {
        self.history.join("\n")
    }
}

/// 对话系统
pub struct DialogSystem;

impl DialogSystem {
    /// 开始对话
    pub fn start_dialog(npc: &Npc) -> Result<DialogSession> {
        if npc.dialogs.is_empty() {
            return Err(MudError::NotFound("该NPC没有对话".to_string()));
        }

        // 从第一个对话节点开始
        let start_node = npc.dialogs.first()
            .map(|d| d.id.clone())
            .unwrap_or_else(|| "greeting".to_string());

        Ok(DialogSession::new(npc.id.clone(), start_node))
    }

    /// 获取当前对话节点
    pub fn get_current_node<'a>(
        npc: &'a Npc,
        session: &DialogSession,
    ) -> Option<&'a DialogNode> {
        npc.get_dialog(&session.current_node)
    }

    /// 执行对话选项
    pub fn select_option(
        npc: &Npc,
        session: &mut DialogSession,
        option_index: usize,
        player: &mut PlayerState,
        world_items: &HashMap<String, ItemTemplate>,
    ) -> Result<DialogResult> {
        let current_node = Self::get_current_node(npc, session)
            .ok_or_else(|| MudError::NotFound("对话节点不存在".to_string()))?;

        if option_index >= current_node.options.len() {
            return Err(MudError::InvalidOperation("无效的选项".to_string()));
        }

        let option = &current_node.options[option_index];

        // 记录玩家选择
        session.add_history(format!("你选择: {}", option.text));

        // 执行动作
        let action_result = if option.next == "end" {
            DialogResult::Ended
        } else {
            // 切换到下一个节点
            session.current_node = option.next.clone();

            // 获取新节点并执行动作
            if let Some(new_node) = npc.get_dialog(&option.next) {
                if let Some(ref action) = new_node.action {
                    Self::execute_action(action, player, npc, world_items)?
                } else {
                    DialogResult::Continue
                }
            } else {
                DialogResult::Continue
            }
        };

        Ok(action_result)
    }

    /// 执行对话动作
    fn execute_action(
        action: &DialogAction,
        player: &mut PlayerState,
        npc: &Npc,
        world_items: &HashMap<String, ItemTemplate>,
    ) -> Result<DialogResult> {
        match action {
            DialogAction::GiveQuest { quest_id, target, count, reward_exp, reward_gold } => {
                use crate::gamenv::player_state::QuestProgress;
                let quest = QuestProgress {
                    quest_id: quest_id.clone(),
                    target: target.clone(),
                    current: 0,
                    target_count: *count,
                    reward_exp: *reward_exp as u32,
                    reward_gold: *reward_gold as u32,
                };
                player.add_quest(quest);
                Ok(DialogResult::QuestAccepted)
            }
            DialogAction::CompleteQuest { quest_id } => {
                if let Some(quest) = player.complete_quest(quest_id) {
                    Ok(DialogResult::QuestCompleted(quest))
                } else {
                    Ok(DialogResult::Continue)
                }
            }
            DialogAction::OpenShop { shop_id } => {
                Ok(DialogResult::ShopOpened(shop_id.clone()))
            }
            DialogAction::Teleport { room_id } => {
                player.move_to(room_id.clone());
                Ok(DialogResult::Teleported(room_id.clone()))
            }
            DialogAction::GiveItem { item_id, count } => {
                if let Some(item) = world_items.get(item_id) {
                    let _ = player.add_item(item, *count);
                    Ok(DialogResult::ItemReceived(item_id.clone(), *count))
                } else {
                    Ok(DialogResult::Continue)
                }
            }
            DialogAction::TeachSkill { skill_id } => {
                // TODO: 实现技能学习
                Ok(DialogResult::SkillLearned(skill_id.clone()))
            }
            DialogAction::Heal => {
                player.heal(player.hp_max);
                player.restore_mp(player.mp_max);
                Ok(DialogResult::Healed)
            }
        }
    }

    /// 格式化对话显示
    pub fn format_dialog(npc: &Npc, node: &DialogNode) -> String {
        let mut output = format!("§C{}§N说: \"{}\"\n\n", npc.name, node.text);

        if !node.options.is_empty() {
            output.push_str("§H请选择:§N\n");
            for (idx, option) in node.options.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", idx + 1, option.text));
            }
        }

        output
    }

    /// 处理对话命令
    pub fn handle_dialog_command(
        cmd: &str,
        npc: &Npc,
        session: &mut DialogSession,
        player: &mut PlayerState,
        world_items: &HashMap<String, ItemTemplate>,
    ) -> Result<String> {
        // 检查是否输入数字选择
        if let Ok(num) = cmd.parse::<usize>() {
            if num > 0 {
                let result = Self::select_option(npc, session, num - 1, player, world_items)?;

                // 格式化结果
                if let Some(node) = Self::get_current_node(npc, session) {
                    let mut output = Self::format_dialog(npc, node);

                    // 添加动作结果
                    match result {
                        DialogResult::QuestAccepted => {
                            output.insert_str(0, "§G[任务已接受]§N\n\n");
                        }
                        DialogResult::QuestCompleted(quest) => {
                            output = format!("§G[任务完成]§N\n获得 {} 经验和 {} 金币！\n\n",
                                quest.reward_exp, quest.reward_gold);
                        }
                        DialogResult::ShopOpened(shop_id) => {
                            output = format!("§C[商店已打开]§N\n商店ID: {}\n\n", shop_id);
                        }
                        DialogResult::Teleported(room_id) => {
                            output = format!("§Y[传送]§N\n你被传送到了 {}\n\n", room_id);
                        }
                        DialogResult::ItemReceived(item, count) => {
                            output.insert_str(0, &format!("§Y[获得物品]§N {} x{}\n\n", item, count));
                        }
                        DialogResult::Healed => {
                            output.insert_str(0, "§G[已完全治愈]§N\n\n");
                        }
                        DialogResult::Ended => {
                            output = "§H对话结束。§N\n".to_string();
                        }
                        _ => {}
                    }

                    return Ok(output);
                }
            }
        }

        // 显示当前对话
        if let Some(node) = Self::get_current_node(npc, session) {
            Ok(Self::format_dialog(npc, node))
        } else {
            Ok("对话已结束。".to_string())
        }
    }
}

/// 对话结果
#[derive(Clone, Debug)]
pub enum DialogResult {
    /// 继续对话
    Continue,
    /// 对话结束
    Ended,
    /// 任务已接受
    QuestAccepted,
    /// 任务已完成
    QuestCompleted(crate::gamenv::player_state::QuestProgress),
    /// 商店已打开
    ShopOpened(String),
    /// 已传送
    Teleported(String),
    /// 获得物品
    ItemReceived(String, i32),
    /// 学会技能
    SkillLearned(String),
    /// 已治疗
    Healed,
}

/// 对话管理器
pub struct DialogManager {
    /// 活跃的对话会话 (userid -> DialogSession)
    sessions: HashMap<String, DialogSession>,
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// 开始对话
    pub fn start(&mut self, userid: String, npc: &Npc) -> Result<DialogSession> {
        let session = DialogSystem::start_dialog(npc)?;
        self.sessions.insert(userid, session.clone());
        Ok(session)
    }

    /// 获取会话
    pub fn get_session(&self, userid: &str) -> Option<&DialogSession> {
        self.sessions.get(userid)
    }

    /// 获取可变会话
    pub fn get_session_mut(&mut self, userid: &str) -> Option<&mut DialogSession> {
        self.sessions.get_mut(userid)
    }

    /// 结束对话
    pub fn end(&mut self, userid: &str) -> bool {
        self.sessions.remove(userid).is_some()
    }

    /// 是否在对话中
    pub fn is_in_dialog(&self, userid: &str) -> bool {
        self.sessions.contains_key(userid)
    }
}

impl Default for DialogManager {
    fn default() -> Self {
        Self::new()
    }
}
