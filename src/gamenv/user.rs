// gamenv/user.rs - 用户对象
// 对应 txpike9/gamenv/clone/user.pike

use crate::core::*;
use crate::rustenv::config::CONFIG;
use crate::rustenv::pike_save::{parse_pike_save_file, PikeValue, get_user_save_path, user_file_exists};
use crate::gamenv::combat::{CombatStats, Combatant};
use crate::gamenv::item::{Item, ItemType};
use crate::gamenv::item::equipment::{Equipment, EquipmentSlots, EquipSlot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 背包栏位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    /// 物品
    pub item: Option<Item>,
    /// 解锁状态
    pub unlocked: bool,
}

impl InventorySlot {
    pub fn new(unlocked: bool) -> Self {
        Self {
            item: None,
            unlocked,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item.is_none()
    }

    pub fn put(&mut self, item: Item) -> Result<()> {
        if !self.unlocked {
            return Err(MudError::InvalidOperation("栏位未解锁".to_string()));
        }
        if !self.is_empty() {
            return Err(MudError::InvalidOperation("栏位已有物品".to_string()));
        }
        self.item = Some(item);
        Ok(())
    }

    pub fn take(&mut self) -> Result<Item> {
        self.item.take().ok_or_else(|| MudError::NotFound("栏位为空".to_string()))
    }
}

/// 用户对象 (对应 /gamenv/clone/user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: ObjectId,
    pub name: String,
    pub name_cn: String,
    pub level: u32,
    pub exp: u64,
    pub hp: i32,
    pub hp_max: i32,
    pub qi: i32,       // 内力
    pub qi_max: i32,
    pub shen: i32,     // 精神
    pub shen_max: i32,
    pub potential: i32, // 潜能
    pub money: u64,
    pub room_id: Option<String>,
    // 战斗属性 (基础值，不含装备加成)
    pub base_stats: CombatStats,
    // 背包
    pub inventory: Vec<InventorySlot>,
    pub inventory_size: usize,
    // 装备栏
    pub equipment: EquipmentSlots,
    // 已学技能
    pub learned_skills: Vec<String>,
    // 技能冷却状态
    pub skill_cooldowns: HashMap<String, u32>,
    // txpike9 兼容字段
    pub password: Option<String>,
    pub login_time: Option<i64>,
    pub online_time: Option<i64>,
    pub first_login: Option<i64>,
    pub userip: Option<String>,
    // 扩展数据 (兼容 txpike9 的 data 字段)
    pub extra_data: serde_json::Value,
}

impl User {
    /// 创建新用户
    pub fn new(name: String) -> Self {
        // 初始20个背包栏位，前10个解锁
        let inventory = (0..20).map(|i| InventorySlot::new(i < 10)).collect();

        Self {
            id: ObjectId::new(),
            name: name.clone(),
            name_cn: name,
            level: 1,
            exp: 0,
            hp: 100,
            hp_max: 100,
            qi: 50,
            qi_max: 50,
            shen: 50,
            shen_max: 50,
            potential: 100,
            money: 0,
            room_id: None,
            base_stats: CombatStats::for_level(1),
            inventory,
            inventory_size: 20,
            equipment: EquipmentSlots::default(),
            learned_skills: vec!["skill_basic_attack".to_string()],
            skill_cooldowns: HashMap::new(),
            password: None,
            login_time: None,
            online_time: None,
            first_login: None,
            userip: None,
            extra_data: serde_json::json!({}),
        }
    }

    /// 获取计算后的战斗属性 (基础 + 装备)
    pub fn get_total_stats(&self) -> CombatStats {
        let mut total_stats = self.base_stats.clone();

        // 应用所有装备的属性加成
        let equip_stats = self.equipment.total_stats();
        total_stats.apply_equip_bonus(&equip_stats);

        total_stats
    }

    /// 添加物品到背包
    pub fn add_item(&mut self, item: Item) -> Result<()> {
        // 首先尝试堆叠
        if item.max_stack > 1 {
            for slot in &mut self.inventory {
                if let Some(ref existing) = slot.item {
                    if existing.can_stack_with(&item) {
                        let can_add = item.quantity.min(existing.max_stack - existing.quantity);
                        if can_add > 0 {
                            slot.item.as_mut().unwrap().add_quantity(can_add)?;
                            if item.quantity > can_add {
                                // 剩余部分继续寻找空位
                                let mut remaining = item.clone();
                                remaining.quantity = item.quantity - can_add;
                                return self.add_item_to_empty_slot(remaining);
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }

        // 寻找空位
        self.add_item_to_empty_slot(item)
    }

    /// 添加物品到空位
    fn add_item_to_empty_slot(&mut self, item: Item) -> Result<()> {
        for slot in &mut self.inventory {
            if slot.is_empty() && slot.unlocked {
                return slot.put(item);
            }
        }
        Err(MudError::InvalidOperation("背包已满".to_string()))
    }

    /// 从背包取出物品
    pub fn remove_item(&mut self, slot_index: usize, quantity: u32) -> Result<Item> {
        if slot_index >= self.inventory.len() {
            return Err(MudError::NotFound("无效的栏位".to_string()));
        }

        let slot = &mut self.inventory[slot_index];
        let item = slot.item.as_ref().ok_or_else(|| MudError::NotFound("栏位为空".to_string()))?;

        if item.quantity <= quantity {
            // 全部取出
            slot.take()
        } else {
            // 部分取出
            let mut taken = item.clone();
            taken.quantity = quantity;
            slot.item.as_mut().unwrap().reduce_quantity(quantity)?;
            Ok(taken)
        }
    }

    /// 装备物品
    pub fn equip_item(&mut self, slot_index: usize) -> Result<()> {
        if slot_index >= self.inventory.len() {
            return Err(MudError::NotFound("无效的栏位".to_string()));
        }

        let slot = &mut self.inventory[slot_index];
        let item = slot.item.take().ok_or_else(|| MudError::NotFound("栏位为空".to_string()))?;

        // 检查是否为装备，并创建Equipment对象
        let equip_slot = match item.item_type {
            ItemType::Weapon => Some(EquipSlot::Weapon),
            ItemType::Armor => {
                // 根据 armor.rs 中定义的护甲类型区分
                // 简化处理：全部作为衣服
                Some(EquipSlot::Armor)
            }
            _ => None,
        };

        let equip_slot = match equip_slot {
            Some(slot) => slot,
            None => {
                slot.put(item)?;
                return Err(MudError::InvalidOperation("无法装备此物品".to_string()));
            }
        };

        // 创建Equipment对象
        let equipment = Equipment {
            item: item.clone(),
            slot: equip_slot,
            stats: Default::default(),
            realm: Default::default(),
            reinforce_level: 0,
            suit_active: false,
            suit_id: None,
        };

        // 装备物品，返回旧装备
        let old = self.equipment.equip(equip_slot, equipment)?;

        // 如果有旧装备，放回背包
        if let Some(old) = old {
            slot.put(old.item)?;
        }

        Ok(())
    }

    /// 卸下装备
    pub fn unequip_item(&mut self, slot: &str) -> Result<Item> {
        let equip_slot = EquipSlot::from_str(slot)
            .ok_or_else(|| MudError::NotFound("无效的装备栏位".to_string()))?;

        let equipment = self.equipment.unequip(equip_slot)?;

        // 返回装备中的物品
        Ok(equipment.item)
    }

    /// 渲染背包
    pub fn render_inventory(&self) -> String {
        let mut result = String::from("=== 背包 ===\n");

        for (i, slot) in self.inventory.iter().enumerate() {
            if slot.unlocked {
                if let Some(ref item) = slot.item {
                    result.push_str(&format!("{}. {}\n", i + 1, item.render_name()));
                } else {
                    result.push_str(&format!("{}. (空)\n", i + 1));
                }
            } else {
                result.push_str(&format!("{}. [锁定]\n", i + 1));
            }
        }

        result
    }

    /// 渲染装备栏
    pub fn render_equipment(&self) -> String {
        self.equipment.render()
    }

    /// 登录处理 (对应 logon())
    pub async fn logon(&mut self) -> Result<String> {
        // TODO: 实现登录流程
        Ok("欢迎使用 RustMUD！\n".to_string())
    }

    /// 移动到房间
    pub fn move_to(&mut self, room_id: String) {
        self.room_id = Some(room_id);
    }

    /// 保存用户数据 (对应 save_object)
    /// 同时保存 JSON 格式 (RustMUD) 和 Pike 格式 (txpike9 兼容)
    pub fn save(&self) -> Result<()> {
        // 保存 JSON 格式
        let user_dir = format!("{}/gamenv/u", CONFIG.root);
        std::fs::create_dir_all(&user_dir)?;
        let user_path = format!("{}/{}.json", user_dir, self.name);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(user_path, json)?;

        // 保存 Pike 格式 (txpike9 兼容)
        self.save_pike_format()?;

        Ok(())
    }

    /// 保存为 Pike save_object 格式 (txpike9 兼容)
    fn save_pike_format(&self) -> Result<()> {
        let pike_path = get_user_save_path(&CONFIG.root, &self.name);
        let dir = Path::new(&pike_path).parent().unwrap();
        std::fs::create_dir_all(dir)?;

        let mut content = format!("#~/gamenv/clone/user.pike\n");
        content.push_str(&format!("name \"{}\"\n", self.name));
        content.push_str(&format!("name_newbei \"{}\"\n", self.name_cn));
        content.push_str(&format!("level {}\n", self.level));
        content.push_str(&format!("exp {}\n", self.exp));
        content.push_str(&format!("hp {}\n", self.hp));
        content.push_str(&format!("hp_max {}\n", self.hp_max));
        content.push_str(&format!("qi {}\n", self.qi));
        content.push_str(&format!("qi_max {}\n", self.qi_max));
        content.push_str(&format!("shen {}\n", self.shen));
        content.push_str(&format!("shen_max {}\n", self.shen_max));
        content.push_str(&format!("potential {}\n", self.potential));
        content.push_str(&format!("money {}\n", self.money));

        if let Some(ref pwd) = self.password {
            content.push_str(&format!("password \"{}\"\n", pwd));
        }
        if let Some(login_time) = self.login_time {
            content.push_str(&format!("login_time {}\n", login_time));
        }
        if let Some(online_time) = self.online_time {
            content.push_str(&format!("online_time {}\n", online_time));
        }

        // 空数据字段
        content.push_str("msgs ([])\n");
        content.push_str("inbox ({})\n");
        content.push_str("inventory_data ({})\n");
        content.push_str("skill_data ({})\n");
        content.push_str("data ({})\n");

        std::fs::write(pike_path, content)?;
        Ok(())
    }

    /// 恢复用户数据 (对应 restore_object)
    /// 优先尝试从 txpike9 格式加载，如果不存在则从 JSON 格式加载
    pub fn load(&mut self) -> Result<bool> {
        let name = self.name.clone();

        // 首先尝试从 txpike9 格式加载
        let pike_path = get_user_save_path(&CONFIG.root, &name);
        if Path::new(&pike_path).exists() {
            return self.load_from_pike(&pike_path);
        }

        // 然后尝试从 JSON 格式加载
        let json_path = format!("{}/gamenv/u/{}.json", CONFIG.root, name);
        if Path::new(&json_path).exists() {
            return self.load_from_json(&json_path);
        }

        Ok(false)
    }

    /// 从 txpike9 Pike 格式加载用户数据
    fn load_from_pike(&mut self, path: &str) -> Result<bool> {
        let save_data = parse_pike_save_file(path)?;

        // 解析基本字段
        if let Some(PikeValue::String(name)) = save_data.variables.get("name") {
            self.name = name.clone();
        }
        if let Some(PikeValue::String(name_cn)) = save_data.variables.get("name_newbei") {
            self.name_cn = name_cn.clone();
        }
        if let Some(level) = save_data.variables.get("level").and_then(|v| v.as_int()) {
            self.level = level as u32;
            // 初始化对应等级的战斗属性
            self.base_stats = CombatStats::for_level(self.level);
        }
        if let Some(exp) = save_data.variables.get("exp").and_then(|v| v.as_int()) {
            self.exp = exp as u64;
        }
        if let Some(hp) = save_data.variables.get("hp").and_then(|v| v.as_int()) {
            self.hp = hp as i32;
        }
        if let Some(hp_max) = save_data.variables.get("hp_max").and_then(|v| v.as_int()) {
            self.hp_max = hp_max as i32;
        }
        if let Some(qi) = save_data.variables.get("qi").and_then(|v| v.as_int()) {
            self.qi = qi as i32;
        }
        if let Some(qi_max) = save_data.variables.get("qi_max").and_then(|v| v.as_int()) {
            self.qi_max = qi_max as i32;
        }
        if let Some(shen) = save_data.variables.get("shen").and_then(|v| v.as_int()) {
            self.shen = shen as i32;
        }
        if let Some(shen_max) = save_data.variables.get("shen_max").and_then(|v| v.as_int()) {
            self.shen_max = shen_max as i32;
        }
        if let Some(potential) = save_data.variables.get("potential").and_then(|v| v.as_int()) {
            self.potential = potential as i32;
        }
        if let Some(money) = save_data.variables.get("money").and_then(|v| v.as_int()) {
            self.money = money as u64;
        }
        if let Some(PikeValue::String(password)) = save_data.variables.get("password") {
            self.password = Some(password.clone());
        }
        if let Some(login_time) = save_data.variables.get("login_time").and_then(|v| v.as_int()) {
            self.login_time = Some(login_time);
        }
        if let Some(online_time) = save_data.variables.get("online_time").and_then(|v| v.as_int()) {
            self.online_time = Some(online_time);
        }
        if let Some(first_login) = save_data.variables.get("first_login").and_then(|v| v.as_int()) {
            self.first_login = Some(first_login);
        }
        if let Some(PikeValue::String(userip)) = save_data.variables.get("userip") {
            self.userip = Some(userip.clone());
        }

        // 初始化新增的字段（txpike9 格式中没有这些）
        if self.inventory.is_empty() {
            let inventory = (0..20).map(|i| InventorySlot::new(i < 10)).collect();
            self.inventory = inventory;
            self.inventory_size = 20;
        }
        if self.learned_skills.is_empty() {
            self.learned_skills = vec!["skill_basic_attack".to_string()];
        }

        tracing::info!("Loaded user {} from txpike9 format", self.name);
        Ok(true)
    }

    /// 从 JSON 格式加载用户数据
    fn load_from_json(&mut self, path: &str) -> Result<bool> {
        let json = std::fs::read_to_string(path)?;
        let loaded: User = serde_json::from_str(&json)?;
        // 复制属性
        self.id = loaded.id;
        self.name = loaded.name;
        self.name_cn = loaded.name_cn;
        self.level = loaded.level;
        self.exp = loaded.exp;
        self.hp = loaded.hp;
        self.hp_max = loaded.hp_max;
        self.qi = loaded.qi;
        self.qi_max = loaded.qi_max;
        self.shen = loaded.shen;
        self.shen_max = loaded.shen_max;
        self.potential = loaded.potential;
        self.money = loaded.money;
        self.room_id = loaded.room_id;
        self.base_stats = loaded.base_stats;
        self.inventory = loaded.inventory;
        self.inventory_size = loaded.inventory_size;
        self.equipment = loaded.equipment;
        self.learned_skills = loaded.learned_skills;
        self.skill_cooldowns = loaded.skill_cooldowns;
        self.password = loaded.password;
        self.login_time = loaded.login_time;
        self.online_time = loaded.online_time;
        self.first_login = loaded.first_login;
        self.userip = loaded.userip;
        self.extra_data = loaded.extra_data;

        tracing::info!("Loaded user {} from JSON format", self.name);
        Ok(true)
    }

    /// 检查用户文件是否存在
    pub fn exists(&self) -> bool {
        user_file_exists(&CONFIG.root, &self.name)
    }

    /// 发送提示符 (对应 write_prompt())
    pub fn write_prompt(&self) -> String {
        format!("> ")
    }

    /// 获取玩家状态显示
    pub fn render_status(&self) -> String {
        let stats = self.get_total_stats();
        format!(
            "【{}】Lv.{} HP:{}/{} Qi:{}/{}\n金币: {}\n",
            self.name_cn,
            self.level,
            self.hp,
            self.hp_max,
            self.qi,
            self.qi_max,
            self.money
        )
    }
}

/// 实现 Combatant trait for User
impl Combatant for User {
    fn get_name(&self) -> &str {
        &self.name_cn
    }

    fn get_level(&self) -> u32 {
        self.level
    }

    fn get_combat_stats(&self) -> &CombatStats {
        // 返回计算后的属性
        // 注意：这里需要内部缓存或使用unsafe，简化处理返回base_stats
        &self.base_stats
    }

    fn get_combat_stats_mut(&mut self) -> &mut CombatStats {
        &mut self.base_stats
    }

    fn is_alive(&self) -> bool {
        self.hp > 0
    }
}
