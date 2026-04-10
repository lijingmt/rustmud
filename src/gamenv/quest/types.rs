// gamenv/quest/types.rs - 任务系统数据类型
// 对应 txpike9/gamenv/single/daemons/questd.pike 和 gamenv/inherit/questnpc.pike

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 任务类型 - 对应txpike9的4种任务类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestType {
    /// 找物品任务 (find)
    Find,
    /// 送物品任务 (give)
    Give,
    /// 击杀NPC任务 (kill)
    Kill,
    /// 询问NPC任务 (ask)
    Ask,
}

impl QuestType {
    /// 从字符串解析任务类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "find" => Some(QuestType::Find),
            "give" => Some(QuestType::Give),
            "kill" => Some(QuestType::Kill),
            "ask" => Some(QuestType::Ask),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            QuestType::Find => "find",
            QuestType::Give => "give",
            QuestType::Kill => "kill",
            QuestType::Ask => "ask",
        }
    }

    /// 获取任务类型的中文名称
    pub fn cn_name(&self) -> &'static str {
        match self {
            QuestType::Find => "寻找",
            QuestType::Give => "送达",
            QuestType::Kill => "击杀",
            QuestType::Ask => "打听",
        }
    }
}

/// 奖励类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RewardType {
    /// 增加属性
    Add { attr: String, amount: i32 },
    /// 给予金钱
    Money { currency: Currency, amount: i32 },
    /// 给予物品
    GiveItem { item_id: String, amount: i32 },
}

/// 货币类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Currency {
    Gold,
    Silver,
    Coin,
}

impl Currency {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gold" => Some(Currency::Gold),
            "silver" => Some(Currency::Silver),
            "coin" => Some(Currency::Coin),
            _ => None,
        }
    }

    pub fn cn_name(&self) -> &'static str {
        match self {
            Currency::Gold => "金币",
            Currency::Silver => "银两",
            Currency::Coin => "铜钱",
        }
    }
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuestStatus {
    /// 未开始
    NotStarted,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
}

/// 任务数据 - 对应txpike9的quest数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    /// 任务唯一ID
    pub id: String,
    /// 任务类型
    pub quest_type: QuestType,
    /// 目标NPC名称
    pub target_npc: String,
    /// 目标物品/事情名称
    pub target_object: String,
    /// 需要的数量
    pub target_amount: i32,
    /// 当前进度
    pub current_amount: i32,
    /// 任务描述（给任务时说的话）
    pub quest_talk: String,
    /// 完成描述（完成任务时说的话）
    pub finish_talk: String,
    /// NPC对话（任务完成后的对话）
    pub npc_talk: String,
    /// 奖励
    pub reward: RewardType,
    /// 任务名称
    pub quest_name: String,
    /// 所需道行等级（用于任务匹配）
    pub required_level: i32,
    /// 是否为主线任务（道行<0表示主线任务）
    pub is_main_quest: bool,
    /// 主线任务所属线名称
    pub main_line_name: Option<String>,
    /// 完成后删除的任务记录
    pub delete_name: Option<String>,
}

impl Quest {
    /// 创建新任务
    pub fn new(
        id: String,
        quest_type: QuestType,
        target_npc: String,
        target_object: String,
        target_amount: i32,
        required_level: i32,
    ) -> Self {
        Self {
            id,
            quest_type,
            target_npc,
            target_object,
            target_amount,
            current_amount: 0,
            quest_talk: String::new(),
            finish_talk: String::new(),
            npc_talk: String::new(),
            reward: RewardType::Add { attr: "exp".to_string(), amount: 100 },
            quest_name: String::new(),
            required_level,
            is_main_quest: required_level < 0,
            main_line_name: None,
            delete_name: None,
        }
    }

    /// 设置对话
    pub fn with_talks(mut self, quest_talk: &str, finish_talk: &str, npc_talk: &str) -> Self {
        self.quest_talk = quest_talk.to_string();
        self.finish_talk = finish_talk.to_string();
        self.npc_talk = npc_talk.to_string();
        self
    }

    /// 设置奖励
    pub fn with_reward(mut self, reward: RewardType) -> Self {
        self.reward = reward;
        self
    }

    /// 设置为主线任务
    pub fn with_main_quest(mut self, quest_name: &str, main_line: &str, delete_name: Option<String>) -> Self {
        self.is_main_quest = true;
        self.quest_name = quest_name.to_string();
        self.main_line_name = Some(main_line.to_string());
        self.delete_name = delete_name;
        self
    }

    /// 检查任务是否完成
    pub fn is_completed(&self) -> bool {
        self.current_amount >= self.target_amount
    }

    /// 增加进度
    pub fn add_progress(&mut self, amount: i32) -> bool {
        self.current_amount = (self.current_amount + amount).min(self.target_amount);
        self.is_completed()
    }

    /// 渲染任务描述
    pub fn render(&self) -> String {
        let status = if self.is_completed() {
            "§g[可完成]§r"
        } else {
            &format!("§Y进度: {}/{}§r", self.current_amount, self.target_amount)
        };

        format!(
            "{} - {}{} {}",
            self.quest_name,
            self.quest_type.cn_name(),
            self.target_object,
            status
        )
    }
}

/// 玩家任务数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerQuestData {
    /// 进行中的任务 (NPC名称 -> 任务)
    pub pending_quests: HashMap<String, Quest>,
    /// 主线任务记录 (主线名 -> 已完成的任务路径)
    pub main_quests: HashMap<String, String>,
    /// 任务缓存
    pub quest_cache: HashMap<String, HashMap<String, i32>>,
}

impl Default for PlayerQuestData {
    fn default() -> Self {
        Self {
            pending_quests: HashMap::new(),
            main_quests: HashMap::new(),
            quest_cache: HashMap::new(),
        }
    }
}

impl PlayerQuestData {
    /// 添加任务
    pub fn add_quest(&mut self, npc_name: &str, quest: Quest) -> Result<(), String> {
        if self.pending_quests.contains_key(npc_name) {
            return Err(format!("已有{}的任务", npc_name));
        }
        if self.pending_quests.len() >= 3 {
            return Err("已有3个任务，无法接受更多任务".to_string());
        }
        self.pending_quests.insert(npc_name.to_string(), quest);
        Ok(())
    }

    /// 获取NPC的任务
    pub fn get_quest(&self, npc_name: &str) -> Option<&Quest> {
        self.pending_quests.get(npc_name)
    }

    /// 获取NPC的可变任务
    pub fn get_quest_mut(&mut self, npc_name: &str) -> Option<&mut Quest> {
        self.pending_quests.get_mut(npc_name)
    }

    /// 移除任务
    pub fn remove_quest(&mut self, npc_name: &str) -> Option<Quest> {
        self.pending_quests.remove(npc_name)
    }

    /// 检查主线任务是否完成
    pub fn is_main_quest_completed(&self, main_line: &str, quest_name: &str) -> bool {
        if let Some(path) = self.main_quests.get(main_line) {
            path.contains(&format!("/{}", quest_name))
        } else {
            false
        }
    }

    /// 添加主线任务完成记录
    pub fn add_main_quest_completion(&mut self, main_line: &str, quest_name: &str) {
        let entry = self.main_quests.entry(main_line.to_string()).or_insert_with(String::new);
        entry.push_str(&format!("/{}", quest_name));
    }

    /// 删除主线任务记录
    pub fn remove_main_quest_entry(&mut self, main_line: &str, quest_name: &str) {
        if let Some(path) = self.main_quests.get_mut(main_line) {
            *path = path.replace(&format!("/{}", quest_name), "");
        }
    }
}

/// 任务模板 - 从CSV加载
#[derive(Debug, Clone)]
pub struct QuestTemplate {
    pub npc_name: String,
    pub required_level: i32,
    pub quest_type: QuestType,
    pub target_npc: String,
    pub target_object: String,
    pub target_amount: i32,
    pub quest_talk: String,
    pub finish_talk: String,
    pub npc_talk: String,
    pub reward_code: String,
    pub quest_name: String,
    pub base_quest: String,
    pub delete_name: String,
}

/// 任务守护进程数据
#[derive(Debug, Clone)]
pub struct QuestDaemonData {
    /// 所有任务模板 (NPC名 -> 道行 -> 任务模板)
    pub quest_templates: HashMap<String, Vec<QuestTemplate>>,
    /// 奖励配置
    pub rewards: HashMap<String, Vec<RewardType>>,
}

impl Default for QuestDaemonData {
    fn default() -> Self {
        Self {
            quest_templates: HashMap::new(),
            rewards: HashMap::new(),
        }
    }
}

/// CSV任务数据行格式
#[derive(Debug, Deserialize)]
pub struct QuestCsvRow {
    #[serde(rename = "0")]
    pub npc_name: String,
    #[serde(rename = "1")]
    pub required_level: String,
    #[serde(rename = "2")]
    pub quest_type: String,
    #[serde(rename = "3")]
    pub target_npc: String,
    #[serde(rename = "4")]
    pub target_object: String,
    #[serde(rename = "5")]
    pub target_amount: String,
    #[serde(rename = "6")]
    pub quest_talk_part1: String,
    #[serde(rename = "7")]
    pub quest_talk_part2: String,
    #[serde(rename = "8")]
    pub finish_talk: String,
    #[serde(rename = "9")]
    pub npc_talk: String,
    #[serde(rename = "10")]
    pub reward_code: String,
    #[serde(rename = "11")]
    pub quest_name: String,
    #[serde(rename = "12")]
    pub base_quest: String,
    #[serde(rename = "13")]
    pub delete_name: String,
}
