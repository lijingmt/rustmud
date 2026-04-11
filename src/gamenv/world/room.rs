// gamenv/world/room.rs - 房间系统
// 对应 txpike9 的 room.pike
// 包含房间的心跳系统 (heart_beat) 用于NPC刷新

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NPC刷新配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcSpawnConfig {
    /// NPC模板ID
    pub template_id: String,
    /// 最大数量
    pub max_count: usize,
    /// 刷新间隔(秒)
    pub respawn_time: u64,
}

impl NpcSpawnConfig {
    pub fn new(template_id: String, max_count: usize, respawn_time: u64) -> Self {
        Self {
            template_id,
            max_count,
            respawn_time,
    }
}
}

/// NPC死亡记录
#[derive(Clone, Debug)]
pub struct KilledNpc {
    /// NPC模板ID
    pub template_id: String,
    /// 死亡时间戳
    pub death_time: i64,
}

/// 房间
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    /// 房间ID
    pub id: String,
    /// 房间名称
    pub name: String,
    /// 短描述
    pub short: String,
    /// 长描述
    pub long: String,
    /// 出口方向 -> 房间ID
    pub exits: HashMap<String, String>,
    /// NPC列表
    #[serde(default)]
    pub npcs: Vec<String>,
    /// 怪物列表
    #[serde(default)]
    pub monsters: Vec<String>,
    /// 地面物品
    #[serde(default)]
    pub items: Vec<String>,
    /// 所属区域
    pub zone: String,
    /// 亮度 0-100
    pub light: i32,
    /// 危险等级
    #[serde(default)]
    pub danger_level: i32,
    /// 房间类型 (normal, store, dungeon等)
    #[serde(default)]
    pub room_type: String,
    /// 交互链接
    #[serde(default)]
    pub links: String,
    /// 是否为安全区（不可PK）
    #[serde(default)]
    pub is_peaceful: bool,
    /// 是否为卧室（可休息睡觉）
    #[serde(default)]
    pub is_bedroom: bool,
    /// NPC刷新配置 (reset使用)
    #[serde(skip)]
    pub spawn_configs: Vec<NpcSpawnConfig>,
    /// 已死亡的NPC列表 (等待刷新)
    #[serde(skip, default)]
    pub killed_npcs: Vec<KilledNpc>,
    /// 上次重置时间 (try_reset使用)
    #[serde(skip)]
    pub last_reset: i64,
    /// 重置间隔(秒) (对应txpike9的reset_interval)
    #[serde(skip)]
    pub reset_interval: i64,
}

impl Room {
    /// 添加NPC刷新配置
    pub fn add_spawn_config(&mut self, template_id: String, max_count: usize, respawn_time: u64) {
        self.spawn_configs.push(NpcSpawnConfig::new(template_id, max_count, respawn_time));
    }

    /// 记录NPC死亡
    pub fn on_npc_killed(&mut self, template_id: String) {
        let now = chrono::Utc::now().timestamp();
        tracing::info!("Room {}: NPC {} killed, will respawn on next reset",
            self.id, template_id);
        self.killed_npcs.push(KilledNpc {
            template_id,
            death_time: now,
        });
    }

    /// 尝试重置房间 - 检查是否需要刷新NPC
    ///
    /// 对应 txpike9 room.pike 中的 try_reset() 函数
    /// 当玩家进入房间时调用此方法
    pub fn try_reset(&mut self) {
        let now = chrono::Utc::now().timestamp();

        // 检查是否需要重置
        if now - self.last_reset > self.reset_interval {
            tracing::debug!("Room {}: Resetting (last reset was {}s ago)",
                self.id, now - self.last_reset);
            self.last_reset = now;
            self.reset();
        }
    }

    /// 重置房间 - 刷新NPC
    ///
    /// 对应 txpike9 room.pike 中的 reset_items() 函数
    fn reset(&mut self) {
        let now = chrono::Utc::now().timestamp();

        // 收集需要刷新的NPC模板ID
        let mut to_spawn: Vec<String> = Vec::new();

        for spawn_config in &self.spawn_configs {
            let template_id = &spawn_config.template_id;

            // 计算当前该模板的NPC数量
            let current_count = self.npcs.iter()
                .filter(|npc_id| npc_id.contains(template_id))
                .count();

            // 检查是否有需要刷新的死亡NPC
            let (can_spawn, needs_respawn) = self.killed_npcs.iter()
                .filter(|k| &k.template_id == template_id)
                .fold((false, false), |(can, needs), killed| {
                    let elapsed = now - killed.death_time;
                    if elapsed >= spawn_config.respawn_time as i64 {
                        (true, true)
                    } else {
                        (can, needs)
                    }
                });

            if can_spawn && current_count < spawn_config.max_count {
                if needs_respawn {
                    to_spawn.push(template_id.clone());
                }
            }
        }

        // 执行刷新
        for template_id in to_spawn {
            self.spawn_npc(template_id);
        }

        // 清理已刷新的死亡记录
        self.killed_npcs.retain(|killed| {
            let needs_respawn = self.spawn_configs.iter()
                .any(|config| &config.template_id == &killed.template_id);
            if !needs_respawn {
                return false;
            }
            let elapsed = now - killed.death_time;
            let respawn_time = self.spawn_configs.iter()
                .find(|c| &c.template_id == &killed.template_id)
                .map(|c| c.respawn_time as i64)
                .unwrap_or(30);
            elapsed < respawn_time
        });
    }

    /// 刷新NPC
    fn spawn_npc(&mut self, template_id: String) {
        let room_id = self.id.clone();

        // 生成新的NPC实例ID
        let npc_id = format!("{}/{}_{:?}",
            room_id,
            template_id.replace('/', "_"),
            chrono::Utc::now().timestamp_millis()
        );

        tracing::info!("Room {}: Spawning NPC {} (template: {})",
            room_id, npc_id, template_id);

        // 添加到房间NPC列表
        self.npcs.push(npc_id.clone());

        // 从killed_npcs中移除一条该模板的记录
        if let Some(pos) = self.killed_npcs.iter()
            .position(|k| k.template_id == template_id) {
            self.killed_npcs.remove(pos);
        }
    }

    /// 获取出口列表
    pub fn get_exits(&self) -> Vec<(String, String)> {
        self.exits.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// 检查是否有某个方向的出口
    pub fn has_exit(&self, direction: &str) -> bool {
        self.exits.contains_key(direction)
    }

    /// 获取出口方向
    pub fn get_exit(&self, direction: &str) -> Option<&String> {
        self.exits.get(direction)
    }

    /// 添加出口
    pub fn add_exit(&mut self, direction: String, room_id: String) {
        self.exits.insert(direction, room_id);
    }

    /// 移除出口
    pub fn remove_exit(&mut self, direction: &str) {
        self.exits.remove(direction);
    }

    /// 添加NPC
    pub fn add_npc(&mut self, npc_id: String) {
        self.npcs.push(npc_id);
    }

    /// 移除NPC
    pub fn remove_npc(&mut self, npc_id: &str) {
        self.npcs.retain(|x| x != npc_id);
    }

    /// 添加物品
    pub fn add_item(&mut self, item_id: String) {
        self.items.push(item_id);
    }

    /// 移除物品
    pub fn remove_item(&mut self, item_id: &str) {
        self.items.retain(|x| x != item_id);
    }

    /// 是否黑暗
    pub fn is_dark(&self) -> bool {
        self.light < 30
    }

    /// 格式化输出
    pub fn format(&self) -> String {
        let exits = self.format_exits();
        format!("{}\n\n{}\n\n出口: {}", self.name, self.long.trim(), exits)
    }

    /// 格式化出口列表
    pub fn format_exits(&self) -> String {
        let direction_names = vec![
            ("north", "北方"),
            ("south", "南方"),
            ("east", "东方"),
            ("west", "西方"),
            ("up", "上方"),
            ("down", "下方"),
            ("northeast", "东北"),
            ("northwest", "西北"),
            ("southeast", "东南"),
            ("southwest", "西南"),
        ];

        let exit_names: Vec<&str> = self.exits
            .keys()
            .map(|dir| {
                direction_names
                    .iter()
                    .find(|(k, _)| k == dir)
                    .map(|(_, v)| *v)
                    .unwrap_or(dir.as_str())
            })
            .collect();

        if exit_names.is_empty() {
            "无明显出口".to_string()
        } else {
            exit_names.join(" ")
        }
    }
}
