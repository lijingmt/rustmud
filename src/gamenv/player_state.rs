// gamenv/player_state.rs - 玩家状态管理
// 对应 txpike9 中的用户状态存储

use crate::gamenv::world::ItemTemplate;
use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 玩家状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    /// 用户ID
    pub userid: String,
    /// 名称
    pub name: String,
    /// 等级
    pub level: u32,
    /// 经验值
    pub exp: u64,
    /// 下一级所需经验
    pub exp_needed: u64,
    /// 当前HP
    pub hp: u32,
    /// 最大HP
    pub hp_max: u32,
    /// 当前MP
    pub mp: u32,
    /// 最大MP
    pub mp_max: u32,
    /// 攻击力
    pub attack: u32,
    /// 防御力
    pub defense: u32,
    /// 金币
    pub gold: u64,
    /// 当前房间ID
    pub current_room: String,
    /// 背包物品 (item_id -> (template_id, count, stackable))
    /// 简化为使用元组，避免循环依赖
    #[serde(default)]
    pub inventory: HashMap<String, (String, i32, bool)>,
    /// 背包容量
    pub inventory_capacity: usize,
    /// 已接任务
    #[serde(default)]
    pub active_quests: Vec<QuestProgress>,
    /// 已完成任务
    #[serde(default)]
    pub completed_quests: Vec<String>,
    /// 上次保存时间
    pub last_save: i64,
    /// 在线状态
    pub online: bool,
}

/// 任务进度
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestProgress {
    /// 任务ID
    pub quest_id: String,
    /// 目标
    pub target: String,
    /// 当前进度
    pub current: i32,
    /// 目标数量
    pub target_count: i32,
    /// 奖励经验
    pub reward_exp: u32,
    /// 奖励金币
    pub reward_gold: u32,
}

impl PlayerState {
    /// 创建新玩家
    pub fn new(userid: String, name: String) -> Self {
        Self {
            userid,
            name,
            level: 1,
            exp: 0,
            exp_needed: 100,
            hp: 100,
            hp_max: 100,
            mp: 50,
            mp_max: 50,
            attack: 15,
            defense: 10,
            gold: 100,
            current_room: "newbie_square".to_string(),
            inventory: HashMap::new(),
            inventory_capacity: 20,
            active_quests: Vec::new(),
            completed_quests: Vec::new(),
            last_save: chrono::Utc::now().timestamp(),
            online: true,
        }
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// 是否死亡
    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    /// 受到伤害
    pub fn take_damage(&mut self, damage: u32) -> u32 {
        let actual_damage = damage.min(self.hp);
        self.hp = self.hp.saturating_sub(damage);
        actual_damage
    }

    /// 治疗
    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    /// 恢复MP
    pub fn restore_mp(&mut self, amount: u32) {
        self.mp = (self.mp + amount).min(self.mp_max);
    }

    /// 添加经验
    pub fn add_exp(&mut self, amount: u64) -> bool {
        self.exp += amount;
        let mut leveled_up = false;
        while self.exp >= self.exp_needed {
            self.exp -= self.exp_needed;
            self.level_up();
            leveled_up = true;
        }
        leveled_up
    }

    /// 升级
    fn level_up(&mut self) {
        self.level += 1;
        self.exp_needed = self.calculate_exp_needed();
        self.hp_max = 100 + (self.level * 10) as u32;
        self.hp = self.hp_max;
        self.mp_max = 50 + (self.level * 5) as u32;
        self.mp = self.mp_max;
        self.attack += 3;
        self.defense += 2;
    }

    /// 计算升级所需经验
    fn calculate_exp_needed(&self) -> u64 {
        100 * (self.level as u64)
    }

    /// 添加物品到背包
    pub fn add_item(&mut self, item: &ItemTemplate, count: i32) -> Result<()> {
        if self.inventory.len() >= self.inventory_capacity && !self.inventory.contains_key(&item.id) {
            return Err(MudError::RuntimeError("背包已满".to_string()));
        }

        let entry = self.inventory.entry(item.id.clone()).or_insert_with(|| {
            (item.id.clone(), 0, item.stackable)
        });

        entry.1 += count;
        Ok(())
    }

    /// 从背包移除物品
    pub fn remove_item(&mut self, item_id: &str, count: i32) -> Result<()> {
        if let Some((_, item_count, _)) = self.inventory.get_mut(item_id) {
            if *item_count < count {
                return Err(MudError::RuntimeError("物品数量不足".to_string()));
            }
            *item_count -= count;
            if *item_count <= 0 {
                self.inventory.remove(item_id);
            }
            Ok(())
        } else {
            Err(MudError::NotFound("背包中没有这个物品".to_string()))
        }
    }

    /// 获取物品数量
    pub fn get_item_count(&self, item_id: &str) -> i32 {
        self.inventory.get(item_id).map(|(_, c, _)| *c).unwrap_or(0)
    }

    /// 检查是否有物品
    pub fn has_item(&self, item_id: &str, count: i32) -> bool {
        self.get_item_count(item_id) >= count
    }

    /// 花费金币
    pub fn spend_gold(&mut self, amount: u64) -> Result<()> {
        if self.gold < amount {
            return Err(MudError::RuntimeError("金币不足".to_string()));
        }
        self.gold -= amount;
        Ok(())
    }

    /// 添加金币
    pub fn add_gold(&mut self, amount: u64) {
        self.gold += amount;
    }

    /// 移动到房间
    pub fn move_to(&mut self, room_id: String) {
        self.current_room = room_id;
    }

    /// 添加任务
    pub fn add_quest(&mut self, quest: QuestProgress) {
        if !self.active_quests.iter().any(|q| q.quest_id == quest.quest_id) {
            self.active_quests.push(quest);
        }
    }

    /// 更新任务进度
    pub fn update_quest_progress(&mut self, target: &str, count: i32) -> Option<QuestProgress> {
        for quest in &mut self.active_quests {
            if quest.target == target {
                quest.current = (quest.current + count).min(quest.target_count);
                if quest.current >= quest.target_count {
                    return Some(quest.clone());
                }
            }
        }
        None
    }

    /// 完成任务
    pub fn complete_quest(&mut self, quest_id: &str) -> Option<QuestProgress> {
        if let Some(pos) = self.active_quests.iter().position(|q| q.quest_id == quest_id) {
            let quest = self.active_quests.remove(pos);
            self.completed_quests.push(quest_id.to_string());
            self.add_exp(quest.reward_exp as u64);
            self.add_gold(quest.reward_gold as u64);
            return Some(quest);
        }
        None
    }

    /// 格式化状态显示
    pub fn format_score(&self) -> String {
        format!(
            "玩家: {}\n\
             等级: {}\n\
             经验: {}/{}\n\
             HP: {}/{}\n\
             MP: {}/{}\n\
             攻击: {}\n\
             防御: {}\n\
             金币: {}",
            self.name,
            self.level,
            self.exp,
            self.exp_needed,
            self.hp,
            self.hp_max,
            self.mp,
            self.mp_max,
            self.attack,
            self.defense,
            self.gold
        )
    }

    /// 计算HP百分比
    pub fn hp_percent(&self) -> u32 {
        if self.hp_max == 0 {
            return 0;
        }
        (self.hp * 100 / self.hp_max).min(100)
    }

    /// 计算MP百分比
    pub fn mp_percent(&self) -> u32 {
        if self.mp_max == 0 {
            return 0;
        }
        (self.mp * 100 / self.mp_max).min(100)
    }

    /// 计算经验百分比
    pub fn exp_percent(&self) -> u32 {
        if self.exp_needed == 0 {
            return 0;
        }
        ((self.exp * 100) / self.exp_needed).min(100) as u32
    }
}

/// 玩家状态管理器
pub struct PlayerStateManager {
    players: HashMap<String, Arc<RwLock<PlayerState>>>,
}

impl PlayerStateManager {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    /// 获取或创建玩家状态
    pub async fn get_or_create(&mut self, userid: String) -> Arc<RwLock<PlayerState>> {
        if !self.players.contains_key(&userid) {
            let state = PlayerState::new(userid.clone(), userid.clone());
            self.players.insert(userid.clone(), Arc::new(RwLock::new(state)));
        }
        self.players.get(&userid).unwrap().clone()
    }

    /// 获取玩家状态
    pub async fn get(&self, userid: &str) -> Option<Arc<RwLock<PlayerState>>> {
        self.players.get(userid).cloned()
    }

    /// 移除玩家状态
    pub async fn remove(&mut self, userid: &str) {
        self.players.remove(userid);
    }

    /// 保存所有玩家
    pub async fn save_all(&self) -> Result<()> {
        // TODO: 实现保存到数据库或文件
        for (userid, state) in &self.players {
            let s = state.read().await;
            tracing::info!("Saving player state for: {}", userid);
        }
        Ok(())
    }

    /// 获取在线玩家列表
    pub async fn get_online_players(&self) -> Vec<String> {
        let mut players = Vec::new();
        for (userid, state) in &self.players {
            let s = state.read().await;
            if s.online {
                players.push(format!("{} ({}级)", userid, s.level));
            }
        }
        players
    }

    /// 获取在线玩家数量
    pub async fn get_online_count(&self) -> usize {
        let mut count = 0;
        for state in self.players.values() {
            let s = state.read().await;
            if s.online {
                count += 1;
            }
        }
        count
    }
}

impl Default for PlayerStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局玩家状态管理器
pub static PLAYER_STATE_MANAGER: std::sync::OnceLock<RwLock<PlayerStateManager>> =
    std::sync::OnceLock::new();

/// 获取玩家状态管理器
pub fn get_player_manager() -> &'static RwLock<PlayerStateManager> {
    PLAYER_STATE_MANAGER.get_or_init(|| RwLock::new(PlayerStateManager::new()))
}
