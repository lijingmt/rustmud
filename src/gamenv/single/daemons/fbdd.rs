// gamenv/single/daemons/fbdd.rs - 副本系统守护进程
// 对应 txpike9/gamenv/single/daemons/fbdd.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 副本类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DungeonType {
    /// 单人副本
    Solo,
    /// 组队副本
    Team,
    /// 帮派副本
    Guild,
    /// 世界BOSS
    WorldBoss,
    /// 无限塔
    Tower,
}

/// 副本状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DungeonStatus {
    /// 未开启
    Closed,
    /// 等待中
    Waiting,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
}

/// 副本信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dungeon {
    /// 副本ID
    pub id: String,
    /// 副本名称
    pub name: String,
    /// 副本类型
    pub dungeon_type: DungeonType,
    /// 最小等级
    pub min_level: i32,
    /// 最大等级
    pub max_level: i32,
    /// 最小人数
    pub min_players: i32,
    /// 最大人数
    pub max_players: i32,
    /// 时间限制（秒）
    pub time_limit: i32,
    /// 状态
    pub status: DungeonStatus,
    /// 入口房间ID
    pub entrance_room: String,
    /// 副本房间列表
    pub rooms: Vec<String>,
    /// BOSS NPC ID
    pub boss_npc: String,
    /// 完成奖励
    pub rewards: DungeonRewards,
}

/// 副本奖励
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonRewards {
    /// 经验奖励
    pub exp: u64,
    /// 金币奖励
    pub gold: u64,
    /// 物品奖励
    pub items: Vec<(String, i32)>,
}

/// 副本实例
#[derive(Clone, Debug)]
pub struct DungeonInstance {
    /// 实例ID
    pub id: String,
    /// 副本ID
    pub dungeon_id: String,
    /// 玩家列表
    pub players: Vec<String>,
    /// 创建时间
    pub created_at: i64,
    /// 开始时间
    pub started_at: Option<i64>,
    /// 完成时间
    pub completed_at: Option<i64>,
    /// 当前房间索引
    pub current_room_index: usize,
    /// 击杀的怪物
    pub killed_monsters: Vec<String>,
    /// 状态
    pub status: DungeonStatus,
}

impl DungeonInstance {
    /// 创建新实例
    pub fn new(dungeon_id: String, players: Vec<String>) -> Self {
        Self {
            id: format!("inst_{}_{}", dungeon_id, chrono::Utc::now().timestamp()),
            dungeon_id,
            players,
            created_at: chrono::Utc::now().timestamp(),
            started_at: None,
            completed_at: None,
            current_room_index: 0,
            killed_monsters: Vec::new(),
            status: DungeonStatus::Waiting,
        }
    }

    /// 开始副本
    pub fn start(&mut self) {
        self.started_at = Some(chrono::Utc::now().timestamp());
        self.status = DungeonStatus::InProgress;
    }

    /// 完成副本
    pub fn complete(&mut self) {
        self.completed_at = Some(chrono::Utc::now().timestamp());
        self.status = DungeonStatus::Completed;
    }

    /// 失败副本
    pub fn fail(&mut self) {
        self.completed_at = Some(chrono::Utc::now().timestamp());
        self.status = DungeonStatus::Failed;
    }

    /// 获取已用时间
    pub fn get_elapsed_time(&self) -> i64 {
        let start = self.started_at.unwrap_or(self.created_at);
        let end = self.completed_at.unwrap_or_else(|| chrono::Utc::now().timestamp());
        end - start
    }
}

/// 副本守护进程
pub struct FbdDaemon {
    /// 所有副本模板
    dungeons: HashMap<String, Dungeon>,
    /// 活跃的副本实例
    instances: HashMap<String, DungeonInstance>,
    /// 玩家到实例的映射 (userid -> instance_id)
    player_instances: HashMap<String, String>,
}

impl FbdDaemon {
    /// 创建新的副本守护进程
    pub fn new() -> Self {
        Self {
            dungeons: HashMap::new(),
            instances: HashMap::new(),
            player_instances: HashMap::new(),
        }
    }

    /// 添加副本
    pub fn add_dungeon(&mut self, dungeon: Dungeon) {
        self.dungeons.insert(dungeon.id.clone(), dungeon);
    }

    /// 获取副本
    pub fn get_dungeon(&self, dungeon_id: &str) -> Option<&Dungeon> {
        self.dungeons.get(dungeon_id)
    }

    /// 获取所有副本
    pub fn get_all_dungeons(&self) -> Vec<&Dungeon> {
        self.dungeons.values().collect()
    }

    /// 创建副本实例
    pub fn create_instance(&mut self, dungeon_id: &str, players: Vec<String>) -> Result<String> {
        // 检查副本是否存在
        let dungeon = self.dungeons.get(dungeon_id)
            .ok_or_else(|| MudError::NotFound(format!("副本 {} 不存在", dungeon_id)))?;

        // 检查玩家是否已在副本中
        for player in &players {
            if self.player_instances.contains_key(player) {
                return Err(MudError::RuntimeError(format!("玩家 {} 已在副本中", player)));
            }
        }

        // 检查人数
        let player_count = players.len() as i32;
        if player_count < dungeon.min_players || player_count > dungeon.max_players {
            return Err(MudError::RuntimeError(format!(
                "人数不符合要求，需要 {}-{} 人",
                dungeon.min_players, dungeon.max_players
            )));
        }

        // 创建实例
        let instance = DungeonInstance::new(dungeon_id.to_string(), players);
        let instance_id = instance.id.clone();

        // 注册玩家映射
        for player in &instance.players {
            self.player_instances.insert(player.clone(), instance_id.clone());
        }

        self.instances.insert(instance_id.clone(), instance);

        Ok(instance_id)
    }

    /// 加入副本实例
    pub fn join_instance(&mut self, instance_id: &str, player: String) -> Result<()> {
        if let Some(instance) = self.instances.get_mut(instance_id) {
            if instance.players.len() >= 5 {
                return Err(MudError::RuntimeError("副本已满".to_string()));
            }
            instance.players.push(player.clone());
            self.player_instances.insert(player, instance_id.to_string());
            Ok(())
        } else {
            Err(MudError::NotFound("副本实例不存在".to_string()))
        }
    }

    /// 离开副本实例
    pub fn leave_instance(&mut self, player: &str) -> Result<()> {
        if let Some(instance_id) = self.player_instances.remove(player) {
            if let Some(instance) = self.instances.get_mut(&instance_id) {
                instance.players.retain(|p| p != player);

                // 如果没有玩家了，删除实例
                if instance.players.is_empty() {
                    self.instances.remove(&instance_id);
                }
            }
            Ok(())
        } else {
            Err(MudError::NotFound("玩家不在副本中".to_string()))
        }
    }

    /// 获取玩家所在的副本实例
    pub fn get_player_instance(&self, player: &str) -> Option<&DungeonInstance> {
        if let Some(instance_id) = self.player_instances.get(player) {
            self.instances.get(instance_id)
        } else {
            None
        }
    }

    /// 格式化副本列表
    pub fn format_dungeon_list(&self) -> String {
        let mut output = String::from("§H=== 副本列表 ===§N\n");

        for dungeon in self.dungeons.values() {
            let type_name = match dungeon.dungeon_type {
                DungeonType::Solo => "单人",
                DungeonType::Team => "组队",
                DungeonType::Guild => "帮派",
                DungeonType::WorldBoss => "世界BOSS",
                DungeonType::Tower => "无限塔",
            };

            let status = match dungeon.status {
                DungeonStatus::Closed => "§R关闭§N",
                DungeonStatus::Waiting => "§G等待中§N",
                DungeonStatus::InProgress => "§Y进行中§N",
                DungeonStatus::Completed => "§C已完成§N",
                DungeonStatus::Failed => "§R已失败§N",
            };

            output.push_str(&format!(
                "§Y[{}]§N {} - {}-{}人 - Lv.{}-{}\n  状态: {}\n",
                dungeon.name,
                type_name,
                dungeon.min_players,
                dungeon.max_players,
                dungeon.min_level,
                dungeon.max_level,
                status
            ));
        }

        output
    }

    /// 初始化默认副本
    pub fn init_default_dungeons(&mut self) {
        // 新手副本
        let newbie_dungeon = Dungeon {
            id: "newbie_cave".to_string(),
            name: "野猪洞穴".to_string(),
            dungeon_type: DungeonType::Solo,
            min_level: 1,
            max_level: 10,
            min_players: 1,
            max_players: 1,
            time_limit: 1800, // 30分钟
            status: DungeonStatus::Waiting,
            entrance_room: "newbie_cave_entrance".to_string(),
            rooms: vec![
                "newbie_cave_1".to_string(),
                "newbie_cave_2".to_string(),
                "newbie_cave_boss".to_string(),
            ],
            boss_npc: "boar_king".to_string(),
            rewards: DungeonRewards {
                exp: 500,
                gold: 200,
                items: vec![("newbie_sword".to_string(), 1)],
            },
        };

        self.add_dungeon(newbie_dungeon);

        tracing::info!("Initialized default dungeons");
    }
}

impl Default for FbdDaemon {
    fn default() -> Self {
        let mut daemon = Self::new();
        daemon.init_default_dungeons();
        daemon
    }
}

/// 全局副本守护进程
pub static FBDD: std::sync::OnceLock<RwLock<FbdDaemon>> = std::sync::OnceLock::new();

/// 获取副本守护进程
pub fn get_fbdd() -> &'static RwLock<FbdDaemon> {
    FBDD.get_or_init(|| RwLock::new(FbdDaemon::default()))
}
