// gamenv/world/loader.rs - 世界数据加载器
// 从 JSON 文件加载 txpike9 的房间、NPC、物品数据

use crate::gamenv::world::{Room, Npc, ItemTemplate, Shop, ShopItem, NpcBehavior, ItemType, ItemQuality, ItemStats};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

/// 世界数据加载器
pub struct WorldLoader {
    data_dir: String,
}

impl WorldLoader {
    pub fn new(data_dir: &str) -> Self {
        Self {
            data_dir: data_dir.to_string(),
        }
    }

    /// 从 JSON 文件加载所有房间
    pub fn load_rooms_from_json(&self) -> Result<HashMap<String, Room>, String> {
        let json_path = format!("{}/rooms_data.json", self.data_dir);
        let json_content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read room data: {}", e))?;

        let json_value: Value = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse room JSON: {}", e))?;

        let mut rooms = HashMap::new();

        if let Some(obj) = json_value.as_object() {
            for (room_id, room_data) in obj {
                if let Some(room_obj) = room_data.as_object() {
                    let room = self.parse_room(room_id, room_obj)?;
                    rooms.insert(room_id.clone(), room);
                }
            }
        }

        tracing::info!("Loaded {} rooms from JSON", rooms.len());
        Ok(rooms)
    }

    /// 从 JSON 文件加载所有NPC
    pub fn load_npcs_from_json(&self) -> Result<HashMap<String, Npc>, String> {
        let json_path = format!("{}/npcs_data.json", self.data_dir);
        let json_content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read NPC data: {}", e))?;

        let json_value: Value = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse NPC JSON: {}", e))?;

        let mut npcs = HashMap::new();

        if let Some(obj) = json_value.as_object() {
            for (npc_id, npc_data) in obj {
                if let Some(npc_obj) = npc_data.as_object() {
                    let npc = self.parse_npc(npc_id, npc_obj)?;
                    npcs.insert(npc_id.clone(), npc);
                }
            }
        }

        tracing::info!("Loaded {} NPCs from JSON", npcs.len());
        Ok(npcs)
    }

    /// 从 JSON 文件加载所有物品
    pub fn load_items_from_json(&self) -> Result<HashMap<String, ItemTemplate>, String> {
        let json_path = format!("{}/items_data.json", self.data_dir);
        let json_content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read item data: {}", e))?;

        let json_value: Value = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse item JSON: {}", e))?;

        let mut items = HashMap::new();

        if let Some(obj) = json_value.as_object() {
            for (item_id, item_data) in obj {
                if let Some(item_obj) = item_data.as_object() {
                    let item = self.parse_item(item_id, item_obj)?;
                    items.insert(item_id.clone(), item);
                }
            }
        }

        tracing::info!("Loaded {} items from JSON", items.len());
        Ok(items)
    }

    /// 从 JSON 文件加载所有商店
    pub fn load_shops_from_json(&self) -> Result<HashMap<String, Shop>, String> {
        let json_path = format!("{}/shops_data.json", self.data_dir);
        let json_content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read shop data: {}", e))?;

        let json_value: Value = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse shop JSON: {}", e))?;

        let mut shops = HashMap::new();

        if let Some(obj) = json_value.as_object() {
            for (shop_id, shop_data) in obj {
                if let Some(shop_obj) = shop_data.as_object() {
                    let shop = self.parse_shop(shop_id, shop_obj)?;
                    shops.insert(shop_id.clone(), shop);
                }
            }
        }

        tracing::info!("Loaded {} shops from JSON", shops.len());
        Ok(shops)
    }

    fn parse_room(&self, room_id: &str, data: &serde_json::Map<String, Value>) -> Result<Room, String> {
        let name_cn = data.get("name_cn")
            .and_then(|v| v.as_str())
            .unwrap_or("未知房间")
            .to_string();

        let desc = data.get("desc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let room_type = data.get("room_type")
            .and_then(|v| v.as_str())
            .unwrap_or("normal")
            .to_string();

        // 解析出口
        let mut exits = HashMap::new();
        if let Some(exits_array) = data.get("exits").and_then(|v| v.as_array()) {
            for exit_str in exits_array {
                if let Some(s) = exit_str.as_str() {
                    if let Some((direction, target)) = s.split_once(':') {
                        exits.insert(direction.to_string(), target.to_string());
                    }
                }
            }
        }

        // 解析NPC
        let npcs = data.get("npcs")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect())
            .unwrap_or_default();

        // 解析属性
        let is_peaceful = data.get("is_peaceful").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
        let is_bedroom = data.get("is_bedroom").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

        let links = data.get("links")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(Room {
            id: room_id.to_string(),
            name: name_cn.clone(),
            short: name_cn,
            long: desc,
            exits,
            npcs,
            monsters: vec![],
            items: vec![],
            zone: self.extract_zone(room_id),
            light: 100,
            danger_level: if is_peaceful { 0 } else { 1 },
            room_type,
            links,
            is_peaceful,
            is_bedroom,
            spawn_configs: vec![],
            killed_npcs: vec![],
            last_reset: 0,
            reset_interval: 100,
        })
    }

    fn parse_npc(&self, npc_id: &str, data: &serde_json::Map<String, Value>) -> Result<Npc, String> {
        let name_cn = data.get("name_cn")
            .and_then(|v| v.as_str())
            .unwrap_or("未知NPC")
            .to_string();

        let desc = data.get("desc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let level = data.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let hp = data.get("hp").and_then(|v| v.as_i64()).unwrap_or(100) as i32;
        let max_hp = data.get("max_hp").and_then(|v| v.as_i64()).unwrap_or(hp as i64) as i32;
        let daoheng = data.get("daoheng").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        Ok(Npc {
            id: npc_id.to_string(),
            name: name_cn.clone(),
            short: name_cn.clone(),
            long: desc,
            level,
            hp,
            hp_max: max_hp,
            mp: daoheng,
            mp_max: daoheng,
            attack: level * 10,
            defense: level * 5,
            exp: level * 100,
            gold: level * 10,
            behavior: NpcBehavior::Passive,
            dialogs: vec![],
            shop: None,
            loot: vec![],
        })
    }

    fn parse_item(&self, item_id: &str, data: &serde_json::Map<String, Value>) -> Result<ItemTemplate, String> {
        let name_cn = data.get("name_cn")
            .and_then(|v| v.as_str())
            .unwrap_or("未知物品")
            .to_string();

        let desc = data.get("desc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let item_type_str = data.get("item_type")
            .and_then(|v| v.as_str())
            .unwrap_or("misc");

        let item_type = match item_type_str {
            "weapon" => ItemType::Weapon,
            "armor" => ItemType::Armor,
            "food" => ItemType::Potion,
            _ => ItemType::Other,
        };

        let level = data.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let value = data.get("value").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        let attack_power = data.get("attack_power").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let parry_power = data.get("parry_power").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        let skill = data.get("skill")
            .and_then(|v| v.as_str())
            .unwrap_or("unarmed")
            .to_string();

        let quality = if level > 500 {
            ItemQuality::Legendary
        } else if level > 300 {
            ItemQuality::Epic
        } else if level > 100 {
            ItemQuality::Rare
        } else if level > 50 {
            ItemQuality::Uncommon
        } else {
            ItemQuality::Common
        };

        let stackable = matches!(item_type, ItemType::Potion);

        Ok(ItemTemplate {
            id: item_id.to_string(),
            name: name_cn,
            item_type,
            subtype: skill,
            description: desc,
            quality,
            level,
            stats: ItemStats {
                attack: attack_power,
                defense: parry_power,
                hp_bonus: 0,
                mp_bonus: 0,
                crit_rate: 0,
                crit_damage: 0,
            },
            price: value,
            stackable,
            effects: vec![],
        })
    }

    fn parse_shop(&self, shop_id: &str, data: &serde_json::Map<String, Value>) -> Result<Shop, String> {
        let name_cn = data.get("name_cn")
            .and_then(|v| v.as_str())
            .unwrap_or("未知商店")
            .to_string();

        let mut shop_items = vec![];

        if let Some(goods_array) = data.get("goods").and_then(|v| v.as_array()) {
            for goods_item in goods_array {
                if let Some(obj) = goods_item.as_object() {
                    let item_path = obj.get("item_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let short_name = obj.get("short_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(item_path)
                        .to_string();

                    shop_items.push(ShopItem {
                        item_id: item_path.to_string(),
                        name: short_name,
                        price: 100,
                        stock: -1,
                    });
                }
            }
        }

        Ok(Shop {
            id: shop_id.to_string(),
            name: name_cn,
            items: shop_items,
        })
    }

    fn extract_zone(&self, room_id: &str) -> String {
        // 从 room_id 中提取区域
        // 例如: "beijing/zhengyangmen" -> "beijing"
        if let Some((zone, _)) = room_id.split_once('/') {
            zone.to_string()
        } else {
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = WorldLoader::new("./data/world");
        assert_eq!(loader.data_dir, "./data/world");
    }
}
