// gamenv/d.rs - 房间/地图系统
// 对应 txpike9/gamenv/d/ 目录

use crate::core::*;
use crate::rustenv::config::CONFIG;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 房间出口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exit {
    /// 出口方向 (north, south, east, west, up, down 等)
    pub direction: String,
    /// 目标房间路径
    pub destination: String,
    /// 是否需要钥匙
    pub require_key: bool,
    /// 钥匙物品路径
    pub key_path: Option<String>,
    /// 是否被NPC看守
    pub guarded_by: Option<String>,
    /// 看守消息
    pub guard_msg: Option<String>,
}

impl Exit {
    pub fn new(direction: String, destination: String) -> Self {
        Self {
            direction,
            destination,
            require_key: false,
            key_path: None,
            guarded_by: None,
            guard_msg: None,
        }
    }

    pub fn with_key(mut self, key_path: String) -> Self {
        self.require_key = true;
        self.key_path = Some(key_path);
        self
    }

    pub fn with_guard(mut self, npc_path: String, msg: String) -> Self {
        self.guarded_by = Some(npc_path);
        self.guard_msg = Some(msg);
        self
    }
}

/// 房间类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoomType {
    /// 普通房间
    Normal,
    /// 商店
    Shop,
    /// 银行/钱庄
    Bank,
    /// 当铺
    Pawnshop,
    /// 睡房（可休息）
    Bedroom,
    /// 和平区（不可战斗）
    Peaceful,
    /// 玩家和平区
    PlayerPeace,
    /// 动态副本
    Dungeon,
}

/// 房间 (对应 WAPMUD_ROOM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// 房间ID (文件路径)
    pub id: String,
    /// 房间名称 (英文)
    pub name: String,
    /// 房间中文名
    pub name_cn: String,
    /// 房间描述
    pub desc: String,
    /// 出口列表
    pub exits: HashMap<String, Exit>,
    /// 房间类型
    pub room_type: RoomType,
    /// 额外连接 (links)
    pub links: String,
    /// 房间内的NPC
    pub npcs: Vec<String>,
    /// 房间内的物品
    pub items: Vec<String>,
    /// 店主 (如果是商店)
    pub shopkeeper: Option<String>,
    /// 可买卖物品 (如果是商店)
    pub goods: HashMap<String, String>,
}

impl Room {
    /// 创建新房间
    pub fn new(id: String, name_cn: String, desc: String) -> Self {
        let name = id.split('/').last().unwrap_or(&id).to_string();
        Self {
            id,
            name,
            name_cn,
            desc,
            exits: HashMap::new(),
            room_type: RoomType::Normal,
            links: String::new(),
            npcs: Vec::new(),
            items: Vec::new(),
            shopkeeper: None,
            goods: HashMap::new(),
        }
    }

    /// 添加出口
    pub fn add_exit(&mut self, direction: String, destination: String) {
        let exit = Exit::new(direction.clone(), destination);
        self.exits.insert(direction, exit);
    }

    /// 添加带门的出口
    pub fn add_closed_exit(&mut self, direction: String) {
        if let Some(exit) = self.exits.get_mut(&direction) {
            exit.require_key = true;
        }
    }

    /// 添加带钥匙的出口
    pub fn add_keyed_exit(&mut self, direction: String, key_path: String) {
        if let Some(exit) = self.exits.get_mut(&direction) {
            exit.require_key = true;
            exit.key_path = Some(key_path);
        }
    }

    /// 添加带看守的出口
    pub fn add_guarded_exit(&mut self, direction: String, npc_path: String, msg: String) {
        if let Some(exit) = self.exits.get_mut(&direction) {
            exit.guarded_by = Some(npc_path);
            exit.guard_msg = Some(msg);
        }
    }

    /// 添加NPC
    pub fn add_npc(&mut self, npc_path: String) {
        self.npcs.push(npc_path);
    }

    /// 添加物品
    pub fn add_item(&mut self, item_path: String) {
        self.items.push(item_path);
    }

    /// 设置房间类型
    pub fn set_room_type(&mut self, room_type: RoomType) {
        self.room_type = room_type;
    }

    /// 是否是和平区
    pub fn is_peaceful(&self) -> bool {
        matches!(self.room_type, RoomType::Peaceful | RoomType::PlayerPeace)
    }

    /// 是否是玩家和平区
    pub fn is_player_peaceful(&self) -> bool {
        matches!(self.room_type, RoomType::PlayerPeace)
    }

    /// 是否是睡房
    pub fn is_bedroom(&self) -> bool {
        matches!(self.room_type, RoomType::Bedroom)
    }

    /// 是否是商店
    pub fn is_shop(&self) -> bool {
        matches!(self.room_type, RoomType::Shop)
    }

    /// 是否是银行
    pub fn is_bank(&self) -> bool {
        matches!(self.room_type, RoomType::Bank)
    }

    /// 是否是当铺
    pub fn is_pawnshop(&self) -> bool {
        matches!(self.room_type, RoomType::Pawnshop)
    }

    /// 设置为商店
    pub fn set_shopkeeper(&mut self, npc_path: String) {
        self.shopkeeper = Some(npc_path);
        self.room_type = RoomType::Shop;
    }

    /// 添加商品
    pub fn add_goods(&mut self, name: String, item_path: String) {
        self.goods.insert(name, item_path);
    }

    /// 添加链接
    pub fn add_link(&mut self, link: String) {
        if !self.links.is_empty() {
            self.links.push('\n');
        }
        self.links.push_str(&link);
    }

    /// 获取出口
    pub fn get_exit(&self, direction: &str) -> Option<&Exit> {
        self.exits.get(direction)
    }

    /// 渲染房间描述
    pub fn render(&self, show_exits: bool) -> String {
        let mut result = format!("§c§e{}§r\n", self.name_cn);
        result.push_str(&format!("{}\n", self.desc));

        if show_exits && !self.exits.is_empty() {
            let exits: Vec<&str> = self.exits.keys().map(|k| k.as_str()).collect();
            result.push_str(&format!("§c出口: {}§r\n", exits.join(" ")));
        }

        if !self.links.is_empty() {
            result.push_str(&format!("{}\n", self.links));
        }

        result
    }
}

/// 地图管理器 (对应 mapd.pike)
pub struct MapManager {
    /// 房间缓存
    rooms: HashMap<String, Arc<RwLock<Room>>>,
    /// 区域列表
    areas: Vec<String>,
}

impl MapManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            areas: vec![
                "beijing".to_string(),
                "xinshoucun".to_string(),
                "huashan".to_string(),
                "dali".to_string(),
                "gaochang".to_string(),
            ],
        }
    }

    /// 加载房间
    pub async fn load_room(&mut self, room_path: &str) -> Result<Arc<RwLock<Room>>> {
        // 检查缓存
        if let Some(room) = self.rooms.get(room_path) {
            return Ok(room.clone());
        }

        // 尝试从文件加载
        let full_path = format!("{}/{}", CONFIG.root, room_path);
        let room = self.load_room_from_file(&full_path).await?;

        // 缓存房间
        self.rooms.insert(room_path.to_string(), room.clone());
        Ok(room)
    }

    /// 从文件加载房间
    async fn load_room_from_file(&self, path: &str) -> Result<Arc<RwLock<Room>>> {
        // TODO: 实现从 .pike 或 .json 文件加载房间
        // 暂时返回一个默认房间
        let room = Room::new(
            path.to_string(),
            "未知房间".to_string(),
            "这是一个未描述的房间。\n".to_string(),
        );
        Ok(Arc::new(RwLock::new(room)))
    }

    /// 获取房间
    pub async fn get_room(&mut self, room_path: &str) -> Result<Arc<RwLock<Room>>> {
        self.load_room(room_path).await
    }

    /// 移动到目标房间
    pub async fn move_to(&mut self, room_path: &str, direction: &str) -> Result<String> {
        let room = self.get_room(room_path).await?;
        let room_ref = room.read().await;

        if let Some(exit) = room_ref.get_exit(direction) {
            Ok(exit.destination.clone())
        } else {
            Err(MudError::NotFound(format!("没有这个方向: {}", direction)))
        }
    }

    /// 创建新手村初始房间
    pub fn create_starter_rooms(&mut self) {
        // 新手村广场
        let mut plaza = Room::new(
            "gamenv/d/xinshoucun/guangchang".to_string(),
            "新手村广场".to_string(),
            "这里是新手村的中心广场，四周聚集着许多新来的冒险者。\n".to_string(),
        );
        plaza.add_exit("north".to_string(), "gamenv/d/xinshoucun/zhongxin".to_string());
        plaza.add_exit("south".to_string(), "gamenv/d/xinshoucun/menkou".to_string());
        plaza.add_exit("east".to_string(), "gamenv/d/xinshoucun/wujiang".to_string());
        plaza.add_exit("west".to_string(), "gamenv/d/xinshoucun/yaojian".to_string());
        plaza.set_room_type(RoomType::Peaceful);
        plaza.add_link("§c[查看帮助:help]§r".to_string());

        // 新手村中心
        let mut center = Room::new(
            "gamenv/d/xinshoucun/zhongxin".to_string(),
            "新手村中心".to_string(),
            "村里的老人们经常在这里聊天休息。\n".to_string(),
        );
        center.add_exit("south".to_string(), "gamenv/d/xinshoucun/guangchang".to_string());
        center.set_room_type(RoomType::Bedroom);
        center.add_link("§c[睡觉:sleep]§r".to_string());

        // 新手村门口
        let mut entrance = Room::new(
            "gamenv/d/xinshoucun/menkou".to_string(),
            "新手村门口".to_string(),
            "村口是一条通往外界的大路。\n".to_string(),
        );
        entrance.add_exit("north".to_string(), "gamenv/d/xinshoucun/guangchang".to_string());
        entrance.add_exit("south".to_string(), "gamenv/d/beijing/zhengyangmen".to_string());

        // 武将馆
        let mut warrior = Room::new(
            "gamenv/d/xinshoucun/wujiang".to_string(),
            "武将馆".to_string(),
            "这里可以学习战斗技能。\n".to_string(),
        );
        warrior.add_exit("west".to_string(), "gamenv/d/xinshoucun/guangchang".to_string());

        // 药店
        let mut pharmacy = Room::new(
            "gamenv/d/xinshoucun/yaojian".to_string(),
            "药店".to_string(),
            "药香扑鼻，这里可以买到各种药品。\n".to_string(),
        );
        pharmacy.add_exit("east".to_string(), "gamenv/d/xinshoucun/guangchang".to_string());
        pharmacy.set_room_type(RoomType::Shop);

        // 缓存房间
        self.rooms.insert("gamenv/d/xinshoucun/guangchang".to_string(), Arc::new(RwLock::new(plaza)));
        self.rooms.insert("gamenv/d/xinshoucun/zhongxin".to_string(), Arc::new(RwLock::new(center)));
        self.rooms.insert("gamenv/d/xinshoucun/menkou".to_string(), Arc::new(RwLock::new(entrance)));
        self.rooms.insert("gamenv/d/xinshoucun/wujiang".to_string(), Arc::new(RwLock::new(warrior)));
        self.rooms.insert("gamenv/d/xinshoucun/yaojian".to_string(), Arc::new(RwLock::new(pharmacy)));
    }

    /// 创建北京地图
    pub fn create_beijing_rooms(&mut self) {
        // 正阳门
        let mut zhengyangmen = Room::new(
            "gamenv/d/beijing/zhengyangmen".to_string(),
            "正阳门".to_string(),
            "北京城的正南门，气势宏伟。\n".to_string(),
        );
        zhengyangmen.add_exit("north".to_string(), "gamenv/d/beijing/qianmen".to_string());
        zhengyangmen.add_exit("south".to_string(), "gamenv/d/xinshoucun/menkou".to_string());

        // 前门大街
        let mut qianmen = Room::new(
            "gamenv/d/beijing/qianmen".to_string(),
            "前门大街".to_string(),
            "繁华的商业街，人来人往。\n".to_string(),
        );
        qianmen.add_exit("north".to_string(), "gamenv/d/beijing/dongdajie".to_string());
        qianmen.add_exit("south".to_string(), "gamenv/d/beijing/zhengyangmen".to_string());

        // 鼓楼
        let mut gulou = Room::new(
            "gamenv/d/beijing/gulou".to_string(),
            "鼓楼".to_string(),
            "气氛非常平静，人们来去匆匆。\n".to_string(),
        );
        gulou.add_exit("south".to_string(), "gamenv/d/beijing/dalisi".to_string());
        gulou.add_exit("west".to_string(), "gamenv/d/beijing/guloudajie".to_string());
        gulou.set_room_type(RoomType::Bedroom);
        gulou.add_link("§c[睡觉:sleep]§r".to_string());

        // 大理寺
        let mut dalisi = Room::new(
            "gamenv/d/beijing/dalisi".to_string(),
            "大理寺".to_string(),
            "这里是处理刑案的地方。\n".to_string(),
        );
        dalisi.add_exit("north".to_string(), "gamenv/d/beijing/gulou".to_string());

        // 鼓楼大街
        let mut guloudajie = Room::new(
            "gamenv/d/beijing/guloudajie".to_string(),
            "鼓楼大街".to_string(),
            "宽阔的大街，两旁店铺林立。\n".to_string(),
        );
        guloudajie.add_exit("east".to_string(), "gamenv/d/beijing/gulou".to_string());

        // 东大街
        let mut dongdajie = Room::new(
            "gamenv/d/beijing/dongdajie".to_string(),
            "东大街".to_string(),
            "北京城的东边主干道。\n".to_string(),
        );
        dongdajie.add_exit("north".to_string(), "gamenv/d/beijing/guloudajie".to_string());
        dongdajie.add_exit("south".to_string(), "gamenv/d/beijing/qianmen".to_string());

        // 缓存房间
        self.rooms.insert("gamenv/d/beijing/zhengyangmen".to_string(), Arc::new(RwLock::new(zhengyangmen)));
        self.rooms.insert("gamenv/d/beijing/qianmen".to_string(), Arc::new(RwLock::new(qianmen)));
        self.rooms.insert("gamenv/d/beijing/gulou".to_string(), Arc::new(RwLock::new(gulou)));
        self.rooms.insert("gamenv/d/beijing/dalisi".to_string(), Arc::new(RwLock::new(dalisi)));
        self.rooms.insert("gamenv/d/beijing/guloudajie".to_string(), Arc::new(RwLock::new(guloudajie)));
        self.rooms.insert("gamenv/d/beijing/dongdajie".to_string(), Arc::new(RwLock::new(dongdajie)));
    }
}

impl Default for MapManager {
    fn default() -> Self {
        let mut mgr = Self::new();
        mgr.create_starter_rooms();
        mgr.create_beijing_rooms();
        mgr
    }
}

/// 全局地图管理器
pub static MAPD: once_cell::sync::Lazy<std::sync::Mutex<MapManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(MapManager::default()));

/// 方向别名映射
pub fn normalize_direction(dir: &str) -> String {
    match dir.to_lowercase().as_str() {
        "n" | "north" => "north".to_string(),
        "s" | "south" => "south".to_string(),
        "e" | "east" => "east".to_string(),
        "w" | "west" => "west".to_string(),
        "u" | "up" => "up".to_string(),
        "d" | "down" => "down".to_string(),
        "ne" | "northeast" => "northeast".to_string(),
        "nw" | "northwest" => "northwest".to_string(),
        "se" | "southeast" => "southeast".to_string(),
        "sw" | "southwest" => "southwest".to_string(),
        "北" => "north".to_string(),
        "南" => "south".to_string(),
        "东" => "east".to_string(),
        "西" => "west".to_string(),
        "上" => "up".to_string(),
        "下" => "down".to_string(),
        _ => dir.to_lowercase(),
    }
}

/// 方向中文名
pub fn direction_name(dir: &str) -> &str {
    match dir {
        "north" => "北",
        "south" => "南",
        "east" => "东",
        "west" => "西",
        "up" => "上",
        "down" => "下",
        "northeast" => "东北",
        "northwest" => "西北",
        "southeast" => "东南",
        "southwest" => "西南",
        _ => dir,
    }
}
