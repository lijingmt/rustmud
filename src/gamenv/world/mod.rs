// gamenv/world/mod.rs - 游戏世界数据
// 对应 txpike9 的房间、NPC、物品等游戏内容

pub mod room;
pub mod npc;
pub mod item;
pub mod shop;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

pub use room::*;
pub use npc::*;
pub use item::*;
pub use shop::*;

/// 全局游戏世界实例
static WORLD: once_cell::sync::Lazy<Arc<TokioRwLock<GameWorld>>> =
    once_cell::sync::Lazy::new(|| Arc::new(TokioRwLock::new(GameWorld::new())));

/// 获取全局游戏世界
pub fn get_world() -> Arc<TokioRwLock<GameWorld>> {
    WORLD.clone()
}

/// 游戏世界
#[derive(Clone, Serialize, Deserialize)]
pub struct GameWorld {
    /// 所有房间
    pub rooms: HashMap<String, Room>,
    /// 所有NPC
    pub npcs: HashMap<String, Npc>,
    /// 所有物品模板
    pub items: HashMap<String, ItemTemplate>,
    /// 所有商店
    pub shops: HashMap<String, Shop>,
}

impl GameWorld {
    pub fn new() -> Self {
        let mut world = Self {
            rooms: HashMap::new(),
            npcs: HashMap::new(),
            items: HashMap::new(),
            shops: HashMap::new(),
        };

        // 初始化游戏世界
        world.init_world();

        world
    }

    /// 初始化游戏世界数据
    fn init_world(&mut self) {
        // 创建新手村区域
        self.create_newbie_village();

        // 创建主城
        self.create_main_city();

        // 创建野外区域
        self.create_wilderness();

        // 创建地牢
        self.create_dungeon();
    }

    /// 获取房间
    pub fn get_room(&self, id: &str) -> Option<&Room> {
        self.rooms.get(id)
    }

    /// 获取NPC
    pub fn get_npc(&self, id: &str) -> Option<&Npc> {
        self.npcs.get(id)
    }

    /// 获取物品模板
    pub fn get_item_template(&self, id: &str) -> Option<&ItemTemplate> {
        self.items.get(id)
    }

    /// 获取商店
    pub fn get_shop(&self, id: &str) -> Option<&Shop> {
        self.shops.get(id)
    }

    /// 添加房间
    pub fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.id.clone(), room);
    }

    /// 添加NPC
    pub fn add_npc(&mut self, npc: Npc) {
        self.npcs.insert(npc.id.clone(), npc);
    }

    /// 添加物品模板
    pub fn add_item_template(&mut self, item: ItemTemplate) {
        self.items.insert(item.id.clone(), item);
    }

    /// 添加商店
    pub fn add_shop(&mut self, shop: Shop) {
        self.shops.insert(shop.id.clone(), shop);
    }

    // ========================================================================
    // 地区创建函数
    // ========================================================================

    /// 创建新手村
    fn create_newbie_village(&mut self) {
        // ==================== 房间 ====================

        // 新手村广场
        let square = Room {
            id: "newbie_square".to_string(),
            name: "新手村广场".to_string(),
            short: "这里是新手村的中心广场".to_string(),
            long: r#"这里是新手村的中心广场，周围聚集着许多初来乍到的冒险者。
广场中央有一口古老的水井，旁边坐着一位慈祥的老人，似乎在等待什么人。
广场上人来人往，热闹非凡。
北边是武器店，南边是药品店，东边是布告栏，西边是村庄出口。
"#.to_string(),
            exits: {
                let mut m = HashMap::new();
                m.insert("north".to_string(), "newbie_weapon_shop".to_string());
                m.insert("south".to_string(), "newbie_potion_shop".to_string());
                m.insert("east".to_string(), "newbie_board".to_string());
                m.insert("west".to_string(), "newbie_exit".to_string());
                m
            },
            npcs: vec!["old_man".to_string()],
            monsters: vec![],
            items: vec![],
            zone: "newbie".to_string(),
            light: 100,
            danger_level: 0,
        };

        // 武器店
        let weapon_shop = Room {
            id: "newbie_weapon_shop".to_string(),
            name: "武器店".to_string(),
            short: "新手武器店".to_string(),
            long: "这是一家简陋的武器店，货架上摆着各种基础武器。\n老板是一位经验丰富的铁匠，正在精心打磨一把长剑。\n南边是广场。".to_string(),
            exits: {
                let mut m = HashMap::new();
                m.insert("south".to_string(), "newbie_square".to_string());
                m
            },
            npcs: vec!["weapon_smith".to_string()],
            monsters: vec![],
            items: vec![],
            zone: "newbie".to_string(),
            light: 100,
            danger_level: 0,
        };

        // 药品店
        let potion_shop = Room {
            id: "newbie_potion_shop".to_string(),
            name: "药品店".to_string(),
            short: "新手药品店".to_string(),
            long: "药品店里飘散着淡淡的草药香。\n柜台后面站着一位药剂师，正在调配新的药水。\n墙上挂着各种药材和瓶瓶罐罐。\n北边是广场。".to_string(),
            exits: {
                let mut m = HashMap::new();
                m.insert("north".to_string(), "newbie_square".to_string());
                m
            },
            npcs: vec!["potion_seller".to_string()],
            monsters: vec![],
            items: vec![],
            zone: "newbie".to_string(),
            light: 100,
            danger_level: 0,
        };

        // 布告栏
        let board = Room {
            id: "newbie_board".to_string(),
            name: "布告栏".to_string(),
            short: "新手布告栏".to_string(),
            long: "这里竖立着一块巨大的木制布告栏，上面贴满了各种任务委托和公告。\n几个冒险者正在查看布告内容。\n西边是广场。".to_string(),
            exits: {
                let mut m = HashMap::new();
                m.insert("west".to_string(), "newbie_square".to_string());
                m
            },
            npcs: vec![],
            monsters: vec![],
            items: vec![],
            zone: "newbie".to_string(),
            light: 100,
            danger_level: 0,
        };

        // 村庄出口
        let exit = Room {
            id: "newbie_exit".to_string(),
            name: "村庄出口".to_string(),
            short: "新手村出口".to_string(),
            long: "这里是新手村的西出口，外面是危险的野外。\n一位守卫正在这里站岗，检查每一个离开村庄的人。\n东边是广场，西边通往野外。".to_string(),
            exits: {
                let mut m = HashMap::new();
                m.insert("east".to_string(), "newbie_square".to_string());
                m.insert("west".to_string(), "wilderness_01".to_string());
                m
            },
            npcs: vec!["village_guard".to_string()],
            monsters: vec![],
            items: vec![],
            zone: "newbie".to_string(),
            light: 100,
            danger_level: 0,
        };

        // 添加房间
        self.add_room(square);
        self.add_room(weapon_shop);
        self.add_room(potion_shop);
        self.add_room(board);
        self.add_room(exit);

        // ==================== NPC ====================

        // 老人
        let old_man = Npc {
            id: "old_man".to_string(),
            name: "老人".to_string(),
            short: "一位慈祥的老人".to_string(),
            long: "老人白发苍苍，但精神矍铄。他似乎在这里等待新手冒险者很久了。".to_string(),
            level: 10,
            hp: 500,
            hp_max: 500,
            mp: 200,
            mp_max: 200,
            attack: 50,
            defense: 30,
            exp: 0,
            gold: 0,
            behavior: NpcBehavior::Passive,
            dialogs: vec![],
            shop: None,
            loot: vec![],
        };

        // 武器店老板
        let weapon_smith = Npc {
            id: "weapon_smith".to_string(),
            name: "铁匠".to_string(),
            short: "经验丰富的武器店老板".to_string(),
            long: "铁匠身材魁梧，手臂粗壮，一看就是常年打铁的人。他的眼神锐利，能一眼看出武器的优劣。".to_string(),
            level: 15,
            hp: 800,
            hp_max: 800,
            mp: 100,
            mp_max: 100,
            attack: 80,
            defense: 60,
            exp: 0,
            gold: 500,
            behavior: NpcBehavior::Passive,
            dialogs: vec![],
            shop: Some("newbie_weapon_shop".to_string()),
            loot: vec![],
        };

        // 药剂师
        let potion_seller = Npc {
            id: "potion_seller".to_string(),
            name: "药剂师".to_string(),
            short: "神秘的药品店老板".to_string(),
            long: "药剂师穿着一身素净的长袍，身上总是带着淡淡的草药香。他的眼睛闪烁着智慧的光芒。".to_string(),
            level: 12,
            hp: 400,
            hp_max: 400,
            mp: 500,
            mp_max: 500,
            attack: 30,
            defense: 40,
            exp: 0,
            gold: 300,
            behavior: NpcBehavior::Passive,
            dialogs: vec![],
            shop: Some("newbie_potion_shop".to_string()),
            loot: vec![],
        };

        // 守卫
        let village_guard = Npc {
            id: "village_guard".to_string(),
            name: "守卫".to_string(),
            short: "村庄守卫".to_string(),
            long: "守卫身穿铠甲，手持长矛，警惕地注视着每一个经过的人。".to_string(),
            level: 20,
            hp: 1000,
            hp_max: 1000,
            mp: 100,
            mp_max: 100,
            attack: 100,
            defense: 80,
            exp: 0,
            gold: 100,
            behavior: NpcBehavior::Passive,
            dialogs: vec![],
            shop: None,
            loot: vec![],
        };

        // 添加 NPC
        self.add_npc(old_man);
        self.add_npc(weapon_smith);
        self.add_npc(potion_seller);
        self.add_npc(village_guard);

        // ==================== 商店 ====================

        // 武器店
        let weapon_shop_shop = Shop {
            id: "newbie_weapon_shop".to_string(),
            name: "新手武器店".to_string(),
            items: vec![
                ShopItem {
                    item_id: "wooden_sword".to_string(),
                    name: "木剑".to_string(),
                    price: 10,
                    stock: 999,
                },
                ShopItem {
                    item_id: "iron_sword".to_string(),
                    name: "铁剑".to_string(),
                    price: 50,
                    stock: 50,
                },
            ],
        };

        // 药品店
        let potion_shop_shop = Shop {
            id: "newbie_potion_shop".to_string(),
            name: "新手药品店".to_string(),
            items: vec![
                ShopItem {
                    item_id: "small_hp_potion".to_string(),
                    name: "小生命药水".to_string(),
                    price: 5,
                    stock: 999,
                },
                ShopItem {
                    item_id: "hp_potion".to_string(),
                    name: "生命药水".to_string(),
                    price: 20,
                    stock: 100,
                },
            ],
        };

        self.add_shop(weapon_shop_shop);
        self.add_shop(potion_shop_shop);

        // ==================== 物品模板 ====================
        self.add_newbie_items();
    }

    /// 添加新手村物品模板
    fn add_newbie_items(&mut self) {
        let items = vec![
            // 武器
            ItemTemplate {
                id: "wooden_sword".to_string(),
                name: "木剑".to_string(),
                item_type: ItemType::Weapon,
                subtype: "sword".to_string(),
                description: "一把简陋的木剑，勉强可以用作武器。".to_string(),
                quality: ItemQuality::Common,
                level: 1,
                stats: ItemStats { attack: 5, defense: 0, hp_bonus: 0, mp_bonus: 0, crit_rate: 0, crit_damage: 0 },
                price: 10,
                stackable: false,
                ..Default::default()
            },
            ItemTemplate {
                id: "iron_sword".to_string(),
                name: "铁剑".to_string(),
                item_type: ItemType::Weapon,
                subtype: "sword".to_string(),
                description: "一把锋利的铁剑，新手冒险者的好选择。".to_string(),
                quality: ItemQuality::Common,
                level: 5,
                stats: ItemStats { attack: 15, defense: 0, hp_bonus: 0, mp_bonus: 0, crit_rate: 0, crit_damage: 0 },
                price: 50,
                stackable: false,
                ..Default::default()
            },
            // 药水
            ItemTemplate {
                id: "small_hp_potion".to_string(),
                name: "小生命药水".to_string(),
                item_type: ItemType::Potion,
                subtype: "potion".to_string(),
                description: "恢复30点HP。".to_string(),
                quality: ItemQuality::Common,
                level: 1,
                stats: ItemStats { attack: 0, defense: 0, hp_bonus: 30, mp_bonus: 0, crit_rate: 0, crit_damage: 0 },
                price: 5,
                stackable: true,
                ..Default::default()
            },
            ItemTemplate {
                id: "hp_potion".to_string(),
                name: "生命药水".to_string(),
                item_type: ItemType::Potion,
                subtype: "potion".to_string(),
                description: "恢复100点HP。".to_string(),
                quality: ItemQuality::Uncommon,
                level: 5,
                stats: ItemStats { attack: 0, defense: 0, hp_bonus: 100, mp_bonus: 0, crit_rate: 0, crit_damage: 0 },
                price: 20,
                stackable: true,
                ..Default::default()
            },
        ];

        for item in items {
            self.add_item_template(item);
        }
    }

    /// 创建主城
    fn create_main_city(&mut self) {
        // TODO: 添加主城房间、NPC、物品等
        tracing::info!("Creating main city...");
    }

    /// 创建野外区域
    fn create_wilderness(&mut self) {
        // TODO: 添加野外房间、NPC、怪物等
        tracing::info!("Creating wilderness...");
    }

    /// 创建地牢
    fn create_dungeon(&mut self) {
        // TODO: 添加地牢房间、NPC、怪物等
        tracing::info!("Creating dungeon...");
    }
}
