// gamenv/single/daemons/spawn_d.rs - NPC刷新守护进程
// 对应 txpike9/gamenv/single/daemons/boss_npcd.pike, dnpchotd.pike
//
// 负责NPC的刷新和重生:
// - NPC死亡后从房间移除
// - 30秒后在原位置刷新

use crate::core::*;
use crate::gamenv::world::*;
use crate::gamenv::efuns::{destruct, register_object, ObjectType};
use crate::gamenv::clone::npc::create_npc_from_template;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tokio::time::{interval, Duration, sleep};

/// 刷新区域
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SpawnZone {
    /// 江湖小镇
    Jhxz,
    /// 大理
    Dali,
    /// 北京
    Beijing,
    /// 恶人峡
    Erenxia,
    /// 雪山
    Xueshan,
    /// 神龙
    Shenlong,
    /// 广幕洞
    Gmd,
    /// 丝绸之路
    Sichou,
}

impl SpawnZone {
    pub fn as_str(&self) -> &str {
        match self {
            SpawnZone::Jhxz => "jhxz",
            SpawnZone::Dali => "dali",
            SpawnZone::Beijing => "beijing",
            SpawnZone::Erenxia => "erenxia",
            SpawnZone::Xueshan => "xueshan",
            SpawnZone::Shenlong => "shenlong",
            SpawnZone::Gmd => "gmd",
            SpawnZone::Sichou => "sichou",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "jhxz" => Some(SpawnZone::Jhxz),
            "dali" => Some(SpawnZone::Dali),
            "beijing" => Some(SpawnZone::Beijing),
            "erenxia" => Some(SpawnZone::Erenxia),
            "xueshan" => Some(SpawnZone::Xueshan),
            "shenlong" => Some(SpawnZone::Shenlong),
            "gmd" => Some(SpawnZone::Gmd),
            "sichou" => Some(SpawnZone::Sichou),
            _ => None,
        }
    }
}

/// 普通NPC刷新点配置 (对应非Boss的野怪)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSpawnPoint {
    /// 模板ID
    pub template_id: String,
    /// 刷新房间ID
    pub room_id: String,
    /// 最大同时存在数量
    pub max_count: usize,
    /// 刷新间隔(秒)
    pub respawn_time: u64,
    /// 当前活跃的NPC列表
    pub active_npcs: Vec<String>,
    /// 待刷新的NPC (death_time, template_id, room_id)
    pub pending_respawns: Vec<(i64, String, String)>,
}

impl NpcSpawnPoint {
    pub fn new(template_id: String, room_id: String) -> Self {
        Self {
            template_id,
            room_id,
            max_count: 1,
            respawn_time: 30, // 默认30秒刷新
            active_npcs: Vec::new(),
            pending_respawns: Vec::new(),
        }
    }

    pub fn with_max_count(mut self, count: usize) -> Self {
        self.max_count = count;
        self
    }

    pub fn with_respawn_time(mut self, seconds: u64) -> Self {
        self.respawn_time = seconds;
        self
    }

    /// 检查是否可以刷新
    pub fn can_spawn(&self) -> bool {
        self.active_npcs.len() < self.max_count
    }

    /// 移除死亡的NPC
    pub fn remove_npc(&mut self, npc_id: &str) {
        self.active_npcs.retain(|id| id != npc_id);
    }

    /// 添加待刷新NPC
    pub fn add_pending_respawn(&mut self, death_time: i64, template_id: String, room_id: String) {
        self.pending_respawns.push((death_time, template_id, room_id));
    }
}

/// 刷新的NPC信息 (Boss用)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnedNpc {
    /// NPC ID
    pub npc_id: String,
    /// 房间 ID
    pub room_id: String,
    /// 区域
    pub zone: SpawnZone,
    /// 刷新时间
    pub spawned_at: i64,
    /// 过期时间
    pub expires_at: i64,
    /// 是否是Boss
    pub is_boss: bool,
    /// 是否存活
    pub is_alive: bool,
}

impl SpawnedNpc {
    pub fn new(npc_id: String, room_id: String, zone: SpawnZone, lifetime_seconds: i64, is_boss: bool) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            npc_id,
            room_id,
            zone,
            spawned_at: now,
            expires_at: now + lifetime_seconds,
            is_boss,
            is_alive: true,
        }
    }

    /// 是否已过期
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }
}

/// Boss配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BossConfig {
    /// Boss NPC ID
    pub npc_id: String,
    /// Boss 名称
    pub name: String,
    /// 刷新区域
    pub zones: Vec<SpawnZone>,
    /// 刷新间隔（秒）
    pub respawn_interval: i64,
    /// 存在时间（秒）
    pub lifetime: i64,
    /// 最小在线玩家数
    pub min_online_players: usize,
    /// 广播消息
    pub broadcast_message: String,
}

/// 刷新守护进程
pub struct SpawnDaemon {
    /// 已刷新的NPC (Boss)
    spawned_npcs: HashMap<String, SpawnedNpc>,
    /// Boss配置
    boss_configs: Vec<BossConfig>,
    /// 上次刷新时间
    last_spawn_times: HashMap<String, i64>,
    /// 当前在线玩家数
    online_players: usize,
    /// 普通NPC刷新点: room_id -> Vec<NpcSpawnPoint>
    npc_spawn_points: HashMap<String, Vec<NpcSpawnPoint>>,
}

impl SpawnDaemon {
    /// 创建新的刷新守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            spawned_npcs: HashMap::new(),
            boss_configs: Vec::new(),
            last_spawn_times: HashMap::new(),
            online_players: 0,
            npc_spawn_points: HashMap::new(),
        };

        daemon.init_boss_configs();
        daemon.init_npc_spawn_points();
        daemon
    }

    /// 初始化普通NPC刷新点
    fn init_npc_spawn_points(&mut self) {
        // 新手村野狼
        self.add_npc_spawn_point(NpcSpawnPoint::new(
            "mob/wolf".to_string(),
            "xinshoucun/changetang".to_string(),
        ).with_max_count(3).with_respawn_time(30));

        // 新手村村民
        self.add_npc_spawn_point(NpcSpawnPoint::new(
            "npc/villager".to_string(),
            "xinshoucun/changetang".to_string(),
        ).with_max_count(5).with_respawn_time(60));
    }

    /// 添加NPC刷新点
    pub fn add_npc_spawn_point(&mut self, spawn: NpcSpawnPoint) {
        let room_id = spawn.room_id.clone();
        self.npc_spawn_points
            .entry(room_id)
            .or_insert_with(Vec::new)
            .push(spawn);
    }

    /// 初始化Boss配置
    fn init_boss_configs(&mut self) {
        // 飘风剑使者
        let miaofengshi = BossConfig {
            npc_id: "boss_miaofengshi".to_string(),
            name: "飘风剑使者".to_string(),
            zones: vec![
                SpawnZone::Jhxz,
                SpawnZone::Dali,
                SpawnZone::Erenxia,
                SpawnZone::Sichou,
                SpawnZone::Xueshan,
            ],
            respawn_interval: 3600, // 1小时
            lifetime: 3600,         // 1小时存在时间
            min_online_players: 3,
            broadcast_message: "出现了$ZONE$NAME，此人行踪诡异定有蹊跷！".to_string(),
        };

        // 三阶剑气光龙
        let guanglong = BossConfig {
            npc_id: "boss_guanglong".to_string(),
            name: "三阶剑气光龙".to_string(),
            zones: vec![
                SpawnZone::Jhxz,
                SpawnZone::Dali,
                SpawnZone::Beijing,
                SpawnZone::Erenxia,
                SpawnZone::Xueshan,
                SpawnZone::Shenlong,
                SpawnZone::Gmd,
            ],
            respawn_interval: 7200, // 2小时
            lifetime: 1200,         // 20分钟存在时间
            min_online_players: 3,
            broadcast_message: "$ZONE突然出现 $NAME！".to_string(),
        };

        // 三阶剑气光龙（彩）
        let guanglong_cai = BossConfig {
            npc_id: "boss_guanglong_cai".to_string(),
            name: "三阶剑气彩光龙".to_string(),
            zones: vec![
                SpawnZone::Jhxz,
                SpawnZone::Dali,
                SpawnZone::Beijing,
                SpawnZone::Erenxia,
                SpawnZone::Xueshan,
                SpawnZone::Shenlong,
                SpawnZone::Gmd,
            ],
            respawn_interval: 14400, // 4小时
            lifetime: 1200,          // 20分钟存在时间
            min_online_players: 3,
            broadcast_message: "$ZONE突然出现 §H$NAME§N！".to_string(),
        };

        // 三阶剑气光龙（彩2）
        let guanglong_cai2 = BossConfig {
            npc_id: "boss_guanglong_cai2".to_string(),
            name: "三阶剑气彩光龙(二阶)".to_string(),
            zones: vec![
                SpawnZone::Jhxz,
                SpawnZone::Dali,
                SpawnZone::Beijing,
                SpawnZone::Erenxia,
                SpawnZone::Xueshan,
                SpawnZone::Shenlong,
                SpawnZone::Gmd,
            ],
            respawn_interval: 14400, // 4小时
            lifetime: 1200,          // 20分钟存在时间
            min_online_players: 3,
            broadcast_message: "$ZONE突然出现 §Y$NAME§N！".to_string(),
        };

        // 三阶剑气光龙（彩3）
        let guanglong_cai3 = BossConfig {
            npc_id: "boss_guanglong_cai3".to_string(),
            name: "三阶剑气彩光龙(三阶)".to_string(),
            zones: vec![
                SpawnZone::Jhxz,
                SpawnZone::Dali,
                SpawnZone::Beijing,
                SpawnZone::Erenxia,
                SpawnZone::Xueshan,
                SpawnZone::Shenlong,
                SpawnZone::Gmd,
            ],
            respawn_interval: 14400, // 4小时
            lifetime: 1200,          // 20分钟存在时间
            min_online_players: 3,
            broadcast_message: "$OR突然出现 §R$NAME§N！".to_string(),
        };

        self.boss_configs.push(miaofengshi);
        self.boss_configs.push(guanglong);
        self.boss_configs.push(guanglong_cai);
        self.boss_configs.push(guanglong_cai2);
        self.boss_configs.push(guanglong_cai3);
    }

    /// 更新在线玩家数
    pub fn update_online_players(&mut self, count: usize) {
        self.online_players = count;
    }

    /// 检查是否可以刷新
    pub fn can_spawn(&self, config: &BossConfig) -> bool {
        // 检查在线玩家数
        if self.online_players < config.min_online_players {
            return false;
        }

        // 检查刷新间隔
        if let Some(&last_time) = self.last_spawn_times.get(&config.npc_id) {
            let now = chrono::Utc::now().timestamp();
            if now - last_time < config.respawn_interval {
                return false;
            }
        }

        // 检查是否已存在
        if self.spawned_npcs.values().any(|npc| {
            npc.npc_id == config.npc_id && npc.is_alive && !npc.is_expired()
        }) {
            return false;
        }

        true
    }

    /// 刷新Boss
    pub fn spawn_boss(&mut self, config: &BossConfig, world: &GameWorld) -> Option<SpawnedNpc> {
        if !self.can_spawn(config) {
            return None;
        }

        // 随机选择区域
        let zone = config.zones[rand::random::<usize>() % config.zones.len()].clone();

        // 获取该区域的房间列表
        let zone_rooms: Vec<_> = world.rooms.values()
            .filter(|room| room.zone == zone.as_str())
            .collect();

        if zone_rooms.is_empty() {
            return None;
        }

        // 尝试找一个非和平房间
        let mut attempts = 0;
        let selected_room = loop {
            let room = zone_rooms[rand::random::<usize>() % zone_rooms.len()];
            if room.danger_level > 0 {
                break room;
            }
            attempts += 1;
            if attempts > 50 {
                break zone_rooms[0]; // 找不到就选第一个
            }
        };

        let spawn_id = format!("spawn_{}_{}", config.npc_id, chrono::Utc::now().timestamp());
        let spawned_npc = SpawnedNpc::new(
            config.npc_id.clone(),
            selected_room.id.clone(),
            zone,
            config.lifetime,
            true,
        );

        self.spawned_npcs.insert(spawn_id.clone(), spawned_npc.clone());
        self.last_spawn_times.insert(config.npc_id.clone(), chrono::Utc::now().timestamp());

        Some(spawned_npc)
    }

    /// 清理过期的刷新NPC
    pub fn cleanup_expired(&mut self) -> usize {
        let before = self.spawned_npcs.len();
        self.spawned_npcs.retain(|_, npc| !npc.is_expired() || npc.is_alive);
        before - self.spawned_npcs.len()
    }

    // ==================== 普通NPC死亡和刷新处理 ====================

    /// 处理NPC死亡 (对外接口)
    pub async fn on_npc_died(&mut self, npc_id: &str, template_id: &str, room_id: &str) {
        tracing::info!("NPC died: {} (template: {}, room: {})", npc_id, template_id, room_id);

        // 1. 从刷新点移除
        self.remove_npc_from_spawn_point(room_id, template_id, npc_id);

        // 2. 从世界中销毁NPC
        destruct(npc_id).await;

        // 3. 安排刷新
        let now = chrono::Utc::now().timestamp();
        self.add_pending_respawn(room_id, template_id, now);
    }

    /// 从刷新点移除NPC
    fn remove_npc_from_spawn_point(&mut self, room_id: &str, template_id: &str, npc_id: &str) {
        if let Some(spawns) = self.npc_spawn_points.get_mut(room_id) {
            for spawn in spawns.iter_mut() {
                if spawn.template_id == template_id {
                    spawn.remove_npc(npc_id);
                }
            }
        }
    }

    /// 添加待刷新NPC
    fn add_pending_respawn(&mut self, room_id: &str, template_id: &str, death_time: i64) {
        if let Some(spawns) = self.npc_spawn_points.get_mut(room_id) {
            for spawn in spawns.iter_mut() {
                if spawn.template_id == template_id {
                    spawn.add_pending_respawn(death_time, template_id.to_string(), room_id.to_string());
                }
            }
        }
    }

    /// 处理刷新 (每秒调用一次)
    pub async fn process_respawns(&mut self) {
        let now = chrono::Utc::now().timestamp();

        for (_room_id, spawns) in self.npc_spawn_points.iter_mut() {
            for spawn in spawns.iter_mut() {
                let respawn_time = spawn.respawn_time as i64;
                let can_spawn = spawn.can_spawn();
                let mut still_pending = Vec::new();

                // 先把待刷新列表取出来，避免借用问题
                let pending: Vec<_> = spawn.pending_respawns.drain(..).collect();

                for (death_time, template_id, room_id) in pending {
                    let elapsed = now - death_time;

                    if elapsed >= respawn_time {
                        if can_spawn {
                            // 执行刷新
                            if let Some(npc) = create_npc_from_template(&template_id).await {
                                let new_npc_id = npc.character.id.clone();

                                // 注册到世界
                                register_object(
                                    new_npc_id.clone(),
                                    ObjectType::Npc,
                                    Some(room_id.clone()),
                                ).await;

                                // 添加到活跃列表
                                spawn.active_npcs.push(new_npc_id.clone());

                                tracing::info!(
                                    "NPC respawned: {} (template: {}, room: {})",
                                    new_npc_id,
                                    template_id,
                                    room_id
                                );
                            } else {
                                // 刷新失败，继续等待
                                still_pending.push((death_time, template_id, room_id));
                            }
                        } else {
                            // 已达最大数量，继续等待
                            still_pending.push((death_time, template_id, room_id));
                        }
                    } else {
                        // 还没到刷新时间
                        still_pending.push((death_time, template_id, room_id));
                    }
                }

                spawn.pending_respawns = still_pending;
            }
        }
    }

    /// NPC被击杀
    pub fn on_npc_killed(&mut self, npc_id: &str, killer_name: &str) -> String {
        let mut msg = String::new();

        for (_, spawned) in self.spawned_npcs.iter_mut() {
            if spawned.npc_id == npc_id && spawned.is_alive {
                spawned.is_alive = false;
                msg = format!("{} 被 {} 击败了！", npc_id, killer_name);
                break;
            }
        }

        msg
    }

    /// 获取活跃的Boss
    pub fn get_active_bosses(&self) -> Vec<&SpawnedNpc> {
        self.spawned_npcs.values()
            .filter(|npc| npc.is_boss && npc.is_alive && !npc.is_expired())
            .collect()
    }

    /// 获取Boss状态
    pub fn get_boss_status(&self, npc_id: &str) -> Option<String> {
        for spawned in self.spawned_npcs.values() {
            if spawned.npc_id == npc_id && spawned.is_alive {
                return Some(format!(
                    "Boss: {} - 所在区域: {} - 刷新时间: {} - 过期时间: {}",
                    spawned.npc_id,
                    spawned.zone.as_str(),
                    spawned.spawned_at,
                    spawned.expires_at
                ));
            }
        }
        None
    }

    /// 格式化广播消息
    pub fn format_broadcast(&self, config: &BossConfig, zone: &SpawnZone) -> String {
        let zone_name = match zone {
            SpawnZone::Jhxz => "江湖小镇",
            SpawnZone::Dali => "大理",
            SpawnZone::Beijing => "北京",
            SpawnZone::Erenxia => "恶人峡",
            SpawnZone::Xueshan => "雪山",
            SpawnZone::Shenlong => "神龙",
            SpawnZone::Gmd => "广幕洞",
            SpawnZone::Sichou => "丝绸之路",
        };

        let mut msg = config.broadcast_message.clone();
        msg = msg.replace("$ZONE", zone_name);
        msg = msg.replace("$NAME", &config.name);
        msg = msg.replace("$OR", zone_name); // For special formatting

        format!("§Y[游戏公告]§N {}", msg)
    }

    /// 启动刷新任务
    pub async fn start_spawn_task(&mut self, world: Arc<TokioRwLock<GameWorld>>) {
        let daemon = Arc::new(TokioRwLock::new(self.clone_ref()));

        // 每秒检查一次NPC刷新，每分钟检查Boss
        let mut interval = interval(Duration::from_secs(1));
        let mut boss_counter = 0;

        loop {
            interval.tick().await;
            boss_counter += 1;

            let mut daemon_guard = daemon.write().await;

            // 每秒都处理NPC刷新
            daemon_guard.process_respawns().await;

            // 每分钟检查一次Boss刷新
            if boss_counter >= 60 {
                boss_counter = 0;
                let world_guard = world.read().await;

                // 清理过期NPC
                daemon_guard.cleanup_expired();

                // 检查每个Boss配置
                for config in daemon_guard.boss_configs.clone() {
                    if daemon_guard.can_spawn(&config) {
                        if let Some(spawned) = daemon_guard.spawn_boss(&config, &world_guard) {
                            let broadcast = daemon_guard.format_broadcast(&config, &spawned.zone);
                            // 这里应该广播给所有在线玩家
                            log::info!("Boss spawned: {} in {}", config.name, spawned.zone.as_str());
                            log::info!("Broadcast: {}", broadcast);
                        }
                    }
                }
            }
        }
    }

    /// 克隆引用（用于Arc包装）
    fn clone_ref(&self) -> Self {
        Self {
            spawned_npcs: self.spawned_npcs.clone(),
            boss_configs: self.boss_configs.clone(),
            last_spawn_times: self.last_spawn_times.clone(),
            online_players: self.online_players,
            npc_spawn_points: self.npc_spawn_points.clone(),
        }
    }
}

impl Default for SpawnDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局刷新守护进程
pub static SPAWND: std::sync::OnceLock<TokioRwLock<SpawnDaemon>> = std::sync::OnceLock::new();

/// 获取刷新守护进程
pub fn get_spawnd() -> &'static TokioRwLock<SpawnDaemon> {
    SPAWND.get_or_init(|| TokioRwLock::new(SpawnDaemon::default()))
}

/// NPC死亡 (全局接口，供战斗系统调用)
///
/// 当NPC被杀死时调用此函数，NPC会从房间消失，30秒后刷新
pub async fn npc_died(npc_id: &str, template_id: &str, room_id: &str) {
    let daemon = get_spawnd();
    let mut daemon = daemon.write().await;
    daemon.on_npc_died(npc_id, template_id, room_id).await;
}
