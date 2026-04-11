// gamenv/entities/room.rs - 房间实体
// 对应 txpike9/wapmud2/single/room.pike
// 房间不使用战斗trait，只负责管理位置和出口

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 房间实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// 房间ID
    pub id: String,
    /// 房间名称
    pub name: String,
    /// 房间描述
    pub long: String,
    /// 短描述
    pub short: String,
    /// 出口 (方向 -> 目标房间ID)
    pub exits: HashMap<String, String>,
    /// 房间中的NPC列表
    pub npcs: Vec<String>,
    /// 房间中的怪物列表
    pub monsters: Vec<String>,
    /// 房间类型
    pub room_type: RoomType,
    /// 区域标识
    pub area: String,
}

/// 房间类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    /// 普通
    Normal,
    /// 安全区 (不能战斗)
    Safe,
    /// 副本入口
    Dungeon,
    /// 商店
    Shop,
    /// 门派
    Guild,
}

impl Room {
    /// 创建新房间
    pub fn new(id: String, name: String, long: String) -> Self {
        Self {
            id,
            name,
            long,
            short: String::new(),
            exits: HashMap::new(),
            npcs: Vec::new(),
            monsters: Vec::new(),
            room_type: RoomType::Normal,
            area: "default".to_string(),
        }
    }

    /// 添加出口
    pub fn add_exit(&mut self, direction: String, target: String) {
        self.exits.insert(direction, target);
    }

    /// 检查是否有指定方向的出口
    pub fn has_exit(&self, direction: &str) -> bool {
        self.exits.contains_key(direction)
    }

    /// 获取指定方向的出口
    pub fn get_exit(&self, direction: &str) -> Option<&String> {
        self.exits.get(direction)
    }

    /// 格式化出口列表
    pub fn format_exits(&self) -> String {
        if self.exits.is_empty() {
            return "无".to_string();
        }

        let names: Vec<String> = self.exits.keys()
            .map(|d| direction_name_cn(d).to_string())
            .collect();

        names.join(" ")
    }

    /// 添加NPC
    pub fn add_npc(&mut self, npc_id: String) {
        self.npcs.push(npc_id);
    }

    /// 添加怪物
    pub fn add_monster(&mut self, monster_id: String) {
        self.monsters.push(monster_id);
    }
}

/// 方向中文名
pub fn direction_name_cn(direction: &str) -> &str {
    match direction {
        "north" | "n" => "北",
        "south" | "s" => "南",
        "east" | "e" => "东",
        "west" | "w" => "西",
        "northeast" | "ne" => "东北",
        "northwest" | "nw" => "西北",
        "southeast" | "se" => "东南",
        "southwest" | "sw" => "西南",
        "up" | "u" => "上",
        "down" | "d" => "下",
        _ => direction,
    }
}

/// 方向短名
pub fn direction_short(direction: &str) -> &str {
    match direction {
        "north" => "n",
        "south" => "s",
        "east" => "e",
        "west" => "w",
        "northeast" => "ne",
        "northwest" => "nw",
        "southeast" => "se",
        "southwest" => "sw",
        "up" => "u",
        "down" => "d",
        _ => direction,
    }
}
