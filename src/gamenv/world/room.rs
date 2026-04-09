// gamenv/world/room.rs - 房间系统

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

impl Room {
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
