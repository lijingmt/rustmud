// gamenv/single/daemons/runtime_npc_d.rs - 运行时NPC状态守护进程
// 对应 txpike9 的对象映射管理
// 跟踪房间中的NPC实例、死亡状态和复活时间

use crate::core::*;
use crate::gamenv::world::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

/// NPC实例状态
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcInstanceState {
    /// NPC模板ID
    pub template_id: String,
    /// 实例ID (唯一标识)
    pub instance_id: String,
    /// 当前房间ID
    pub current_room: String,
    /// 是否存活
    pub is_alive: bool,
    /// 死亡时间
    pub died_at: Option<i64>,
    /// 预计复活时间
    pub respawn_at: Option<i64>,
    /// 复活间隔（秒）
    pub respawn_interval: i64,
    /// 创建时间
    pub created_at: i64,
}

impl NpcInstanceState {
    /// 创建新的NPC实例
    pub fn new(template_id: String, room_id: String, respawn_seconds: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        let instance_id = format!("{}_{}_{}", template_id, room_id, now);
        Self {
            template_id,
            instance_id,
            current_room: room_id,
            is_alive: true,
            died_at: None,
            respawn_at: None,
            respawn_interval: respawn_seconds,
            created_at: now,
        }
    }

    /// 标记为死亡
    pub fn mark_dead(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.is_alive = false;
        self.died_at = Some(now);
        self.respawn_at = Some(now + self.respawn_interval);
    }

    /// 复活
    pub fn respawn(&mut self) {
        self.is_alive = true;
        self.died_at = None;
        self.respawn_at = None;
    }

    /// 检查是否可以复活
    pub fn can_respawn(&self) -> bool {
        if let Some(respawn_time) = self.respawn_at {
            chrono::Utc::now().timestamp() >= respawn_time
        } else {
            false
        }
    }

    /// 获取剩余复活时间（秒）
    pub fn respawn_remaining(&self) -> i64 {
        if let Some(respawn_time) = self.respawn_at {
            let now = chrono::Utc::now().timestamp();
            (respawn_time - now).max(0)
        } else {
            0
        }
    }

    /// 格式化剩余时间
    pub fn format_remaining(&self) -> String {
        if self.is_alive {
            "存活中".to_string()
        } else {
            let secs = self.respawn_remaining();
            if secs <= 0 {
                "即将复活".to_string()
            } else if secs < 60 {
                format!("{}秒后复活", secs)
            } else if secs < 3600 {
                format!("{}分{}秒后复活", secs / 60, secs % 60)
            } else {
                format!("{}小时{}分后复活", secs / 3600, (secs % 3600) / 60)
            }
        }
    }
}

/// 运行时NPC守护进程
pub struct RuntimeNpcDaemon {
    /// 房间 -> NPC实例列表
    /// 每个房间维护一个NPC实例列表
    room_npcs: HashMap<String, Vec<NpcInstanceState>>,

    /// NPC模板ID -> 默认复活间隔
    respawn_intervals: HashMap<String, i64>,

    /// 后台任务是否运行中
    background_running: bool,
}

impl RuntimeNpcDaemon {
    /// 创建新的运行时NPC守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            room_npcs: HashMap::new(),
            respawn_intervals: HashMap::new(),
            background_running: false,
        };

        daemon.init_default_intervals();
        daemon
    }

    /// 初始化默认NPC复活间隔
    fn init_default_intervals(&mut self) {
        // 普通NPC - 5分钟复活
        self.respawn_intervals.insert("default".to_string(), 300);

        // 新手村NPC - 3分钟
        self.respawn_intervals.insert("xinshoucun/".to_string(), 180);

        // 野外怪物 - 2分钟
        self.respawn_intervals.insert("monster".to_string(), 120);

        // Boss怪物 - 30分钟到1小时
        self.respawn_intervals.insert("boss".to_string(), 3600);

        // 精英怪 - 10分钟
        self.respawn_intervals.insert("elite".to_string(), 600);
    }

    /// 获取NPC的复活间隔
    fn get_respawn_interval(&self, template_id: &str) -> i64 {
        // 精确匹配
        if let Some(&interval) = self.respawn_intervals.get(template_id) {
            return interval;
        }

        // 前缀匹配
        for (prefix, &interval) in &self.respawn_intervals {
            if template_id.starts_with(prefix) {
                return interval;
            }
        }

        // Boss匹配
        if template_id.contains("boss") {
            return *self.respawn_intervals.get("boss").unwrap_or(&3600);
        }
        if template_id.contains("elite") {
            return *self.respawn_intervals.get("elite").unwrap_or(&600);
        }

        // 默认值
        *self.respawn_intervals.get("default").unwrap_or(&300)
    }

    /// 初始化房间中的NPC
    /// 在服务器启动或房间首次加载时调用
    pub fn init_room_npcs(&mut self, room_id: &str, npc_templates: &[String]) {
        let now = chrono::Utc::now().timestamp();
        let mut instances = Vec::new();

        for template_id in npc_templates {
            let respawn_seconds = self.get_respawn_interval(template_id);
            let instance = NpcInstanceState::new(
                template_id.clone(),
                room_id.to_string(),
                respawn_seconds,
            );
            instances.push(instance);
        }

        self.room_npcs.insert(room_id.to_string(), instances);

        tracing::info!(
            "Initialized room {} with {} NPCs from templates",
            room_id,
            npc_templates.len()
        );
    }

    /// 获取房间中的活跃NPC列表（过滤已死亡的）
    pub fn get_alive_npcs(&self, room_id: &str) -> Vec<String> {
        if let Some(instances) = self.room_npcs.get(room_id) {
            instances.iter()
                .filter(|npc| npc.is_alive)
                .map(|npc| npc.template_id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取房间中的所有NPC（包括已死亡的）
    pub fn get_all_npcs(&self, room_id: &str) -> Vec<NpcInstanceState> {
        if let Some(instances) = self.room_npcs.get(room_id) {
            instances.clone()
        } else {
            Vec::new()
        }
    }

    /// NPC被杀死
    pub fn on_npc_killed(&mut self, template_id: &str, room_id: &str) {
        if let Some(instances) = self.room_npcs.get_mut(room_id) {
            for instance in instances.iter_mut() {
                if instance.template_id == template_id && instance.is_alive {
                    instance.mark_dead();

                    tracing::info!(
                        "NPC {} @ {} killed, will respawn in {} seconds (at {})",
                        template_id,
                        room_id,
                        instance.respawn_interval,
                        instance.respawn_at.unwrap_or(0)
                    );
                    return;
                }
            }

            // 如果没找到活着的实例，可能是已经死了，记录警告
            tracing::warn!(
                "NPC {} @ {} already dead or not found in room instances",
                template_id, room_id
            );
        } else {
            tracing::warn!(
                "Room {} not initialized in runtime NPC daemon",
                room_id
            );
        }
    }

    /// 处理复活的NPC
    pub fn process_respawns(&mut self) -> usize {
        let mut respawned_count = 0;

        for instances in self.room_npcs.values_mut() {
            for instance in instances.iter_mut() {
                if !instance.is_alive && instance.can_respawn() {
                    instance.respawn();
                    respawned_count += 1;

                    tracing::info!(
                        "NPC {} respawned in room {}",
                        instance.template_id,
                        instance.current_room
                    );
                }
            }
        }

        respawned_count
    }

    /// 清理过期的死亡状态（已复活的）
    pub fn cleanup(&mut self) -> usize {
        // 已在 process_respawns 中处理
        0
    }

    /// 检查NPC是否存活
    pub fn is_npc_alive(&self, template_id: &str, room_id: &str) -> bool {
        if let Some(instances) = self.room_npcs.get(room_id) {
            for instance in instances {
                if instance.template_id == template_id {
                    return instance.is_alive;
                }
            }
        }
        // 如果没有运行时实例，默认为存活（从静态模板加载）
        true
    }

    /// 强制复活NPC
    pub fn force_respawn(&mut self, template_id: &str, room_id: &str) -> bool {
        if let Some(instances) = self.room_npcs.get_mut(room_id) {
            for instance in instances.iter_mut() {
                if instance.template_id == template_id && !instance.is_alive {
                    instance.respawn();
                    tracing::info!(
                        "Force respawned NPC {} @ {}",
                        template_id, room_id
                    );
                    return true;
                }
            }
        }
        false
    }

    /// 获取房间NPC状态信息
    pub fn get_room_npc_status(&self, room_id: &str) -> String {
        if let Some(instances) = self.room_npcs.get(room_id) {
            let mut status = String::new();
            for instance in instances {
                status.push_str(&format!(
                    "{}: {}\n",
                    instance.template_id,
                    instance.format_remaining()
                ));
            }
            status
        } else {
            "房间未初始化".to_string()
        }
    }

    /// 设置NPC的复活间隔
    pub fn set_respawn_interval(&mut self, pattern: String, seconds: i64) {
        self.respawn_intervals.insert(pattern, seconds);
    }

    /// 启动后台任务
    pub async fn start_background_task(daemon: Arc<TokioRwLock<Self>>) {
        // 每30秒检查一次复活
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            let mut guard = daemon.write().await;
            let respawned = guard.process_respawns();

            if respawned > 0 {
                tracing::debug!("Respawned {} NPCs", respawned);
            }
        }
    }

    /// 获取死亡NPC数量
    pub fn dead_count(&self) -> usize {
        let mut count = 0;
        for instances in self.room_npcs.values() {
            for instance in instances {
                if !instance.is_alive {
                    count += 1;
                }
            }
        }
        count
    }

    /// 获取总NPC实例数量
    pub fn total_count(&self) -> usize {
        let mut count = 0;
        for instances in self.room_npcs.values() {
            count += instances.len();
        }
        count
    }
}

impl Default for RuntimeNpcDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局运行时NPC守护进程
pub static RUNTIME_NPC_D: std::sync::OnceLock<TokioRwLock<RuntimeNpcDaemon>> = std::sync::OnceLock::new();

/// 获取运行时NPC守护进程
pub fn get_runtime_npc_d() -> &'static TokioRwLock<RuntimeNpcDaemon> {
    RUNTIME_NPC_D.get_or_init(|| TokioRwLock::new(RuntimeNpcDaemon::default()))
}
