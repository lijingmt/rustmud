// gamenv/single/daemons/petd.rs - 宠物系统守护进程
// 对应 txpike9/gamenv/single/daemons/petd.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 宠物状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PetStatus {
    /// 休息中
    Resting,
    /// 跟随中
    Following,
    /// 战斗中
    Fighting,
    /// 死亡
    Dead,
}

/// 宠物成长阶段
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PetGrowthStage {
    /// 幼年期
    Baby,
    /// 成长期
    Growing,
    /// 成熟期
    Mature,
    /// 完全体
    Perfect,
}

/// 宠物属性
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PetStats {
    /// 力量
    pub strength: u32,
    /// 敏捷
    pub agility: u32,
    /// 智力
    pub intelligence: u32,
    /// 体质
    pub constitution: u32,
    /// 幸运
    pub luck: u32,
}

impl Default for PetStats {
    fn default() -> Self {
        Self {
            strength: 10,
            agility: 10,
            intelligence: 10,
            constitution: 10,
            luck: 5,
        }
    }
}

/// 宠物战斗属性
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PetCombatStats {
    /// HP
    pub hp: u32,
    /// 最大HP
    pub hp_max: u32,
    /// 攻击力
    pub attack: u32,
    /// 防御力
    pub defense: u32,
    /// 暴击率
    pub crit_rate: u32,
    /// 闪避率
    pub dodge_rate: u32,
}

/// 宠物技能
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PetSkill {
    /// 技能ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能等级
    pub level: u32,
    /// 技能描述
    pub description: String,
}

/// 宠物
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pet {
    /// 宠物唯一ID
    pub id: String,
    /// 宠物模板ID
    pub template_id: String,
    /// 宠物名称
    pub name: String,
    /// 主人ID
    pub owner_id: String,
    /// 等级
    pub level: u32,
    /// 经验值
    pub exp: u64,
    /// 成长阶段
    pub growth_stage: PetGrowthStage,
    /// 状态
    pub status: PetStatus,
    /// 基础属性
    pub stats: PetStats,
    /// 战斗属性
    pub combat: PetCombatStats,
    /// 技能列表
    pub skills: Vec<PetSkill>,
    /// 忠诚度 (0-100)
    pub loyalty: u32,
    /// 快乐度 (0-100)
    pub happiness: u32,
    /// 饱食度 (0-100)
    pub hunger: u32,
    /// 获得时间
    pub obtained_at: i64,
    /// 上次喂食时间
    pub last_fed: i64,
}

impl Pet {
    /// 创建新宠物
    pub fn new(id: String, template_id: String, name: String, owner_id: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            template_id,
            name,
            owner_id,
            level: 1,
            exp: 0,
            growth_stage: PetGrowthStage::Baby,
            status: PetStatus::Resting,
            stats: PetStats::default(),
            combat: PetCombatStats {
                hp: 100,
                hp_max: 100,
                attack: 15,
                defense: 10,
                crit_rate: 5,
                dodge_rate: 5,
            },
            skills: Vec::new(),
            loyalty: 50,
            happiness: 50,
            hunger: 80,
            obtained_at: now,
            last_fed: now,
        }
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.status != PetStatus::Dead && self.combat.hp > 0
    }

    /// 是否饥饿
    pub fn is_hungry(&self) -> bool {
        self.hunger < 30
    }

    /// 是否不快乐
    pub fn is_unhappy(&self) -> bool {
        self.happiness < 30
    }

    /// 增加经验
    pub fn add_exp(&mut self, exp: u64) -> bool {
        self.exp += exp;

        // 升级公式
        let needed = self.level as u64 * 100;
        if self.exp >= needed {
            self.exp -= needed;
            return self.level_up();
        }

        false
    }

    /// 升级
    pub fn level_up(&mut self) -> bool {
        if self.level >= 100 {
            return false;
        }

        self.level += 1;

        // 提升属性
        self.combat.hp_max += 10;
        self.combat.hp = self.combat.hp_max;
        self.combat.attack += 3;
        self.combat.defense += 2;

        // 检查成长阶段
        self.check_growth_stage();

        true
    }

    /// 检查成长阶段
    pub fn check_growth_stage(&mut self) {
        self.growth_stage = match self.level {
            1..=19 => PetGrowthStage::Baby,
            20..=49 => PetGrowthStage::Growing,
            50..=79 => PetGrowthStage::Mature,
            80..=100 => PetGrowthStage::Perfect,
            _ => PetGrowthStage::Baby,
        };
    }

    /// 喂食
    pub fn feed(&mut self, amount: u32) -> Result<()> {
        if self.status == PetStatus::Dead {
            return Err(MudError::RuntimeError("宠物已死亡".to_string()));
        }

        self.hunger = (self.hunger + amount).min(100);
        self.happiness = (self.happiness + 5).min(100);
        self.loyalty = (self.loyalty + 2).min(100);
        self.last_fed = chrono::Utc::now().timestamp();

        Ok(())
    }

    /// 抚摸
    pub fn pet(&mut self) -> Result<()> {
        if self.status == PetStatus::Dead {
            return Err(MudError::RuntimeError("宠物已死亡".to_string()));
        }

        self.happiness = (self.happiness + 10).min(100);
        self.loyalty = (self.loyalty + 3).min(100);

        Ok(())
    }

    /// 造成伤害
    pub fn take_damage(&mut self, damage: u32) {
        if damage >= self.combat.hp {
            self.combat.hp = 0;
            self.status = PetStatus::Dead;
        } else {
            self.combat.hp -= damage;
        }
    }

    /// 治疗
    pub fn heal(&mut self, amount: u32) {
        if self.status == PetStatus::Dead {
            return;
        }
        self.combat.hp = (self.combat.hp + amount).min(self.combat.hp_max);
    }

    /// 复活
    pub fn revive(&mut self) {
        if self.status == PetStatus::Dead {
            self.status = PetStatus::Resting;
            self.combat.hp = (self.combat.hp_max / 2).max(1);
        }
    }

    /// 计算攻击力
    pub fn calculate_attack(&self) -> u32 {
        let base = self.combat.attack;
        let stat_bonus = (self.stats.strength as f32 * 0.5) as u32;
        let level_bonus = self.level * 2;
        base + stat_bonus + level_bonus
    }

    /// 计算防御力
    pub fn calculate_defense(&self) -> u32 {
        let base = self.combat.defense;
        let stat_bonus = (self.stats.constitution as f32 * 0.3) as u32;
        let level_bonus = self.level;
        base + stat_bonus + level_bonus
    }

    /// 格式化宠物信息
    pub fn format_info(&self) -> String {
        let stage = match self.growth_stage {
            PetGrowthStage::Baby => "§C幼年期§N",
            PetGrowthStage::Growing => "§G成长期§N",
            PetGrowthStage::Mature => "§B成熟期§N",
            PetGrowthStage::Perfect => "§Y完全体§N",
        };

        let status = match self.status {
            PetStatus::Resting => "休息中",
            PetStatus::Following => "§G跟随中§N",
            PetStatus::Fighting => "§R战斗中§N",
            PetStatus::Dead => "§X死亡§N",
        };

        format!(
            "§H[{}]§N {} Lv.{}\n\
             状态: {} | 阶段: {}\n\
             HP: {}/{} | 攻击: {} | 防御: {}\n\
             忠诚: {}% | 快乐: {}% | 饱食: {}%\n\
             力:{} 敏:{} 智:{} 体:{} 运:{}",
            self.name,
            stage,
            self.level,
            status,
            match self.growth_stage {
                PetGrowthStage::Baby => "幼年",
                PetGrowthStage::Growing => "成长",
                PetGrowthStage::Mature => "成熟",
                PetGrowthStage::Perfect => "完全",
            },
            self.combat.hp,
            self.combat.hp_max,
            self.calculate_attack(),
            self.calculate_defense(),
            self.loyalty,
            self.happiness,
            self.hunger,
            self.stats.strength,
            self.stats.agility,
            self.stats.intelligence,
            self.stats.constitution,
            self.stats.luck
        )
    }

    /// 更新状态（定时调用）
    pub fn update(&mut self) {
        // 饱食度自然下降
        self.hunger = self.hunger.saturating_sub(1);

        // 饥饿影响快乐度和忠诚度
        if self.is_hungry() {
            self.happiness = self.happiness.saturating_sub(2);
            self.loyalty = self.loyalty.saturating_sub(1);
        }

        // 快乐度过低会逃跑
        if self.happiness == 0 || self.loyalty == 0 {
            // 宠物可能会逃跑（逻辑在守护进程处理）
        }
    }
}

/// 宠物模板
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PetTemplate {
    /// 模板ID
    pub id: String,
    /// 名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 初始属性加成
    pub stat_bonus: PetStats,
    /// 稀有度 (1-5)
    pub rarity: u32,
    /// 初始技能
    pub initial_skills: Vec<String>,
    /// 获取方式
    pub obtain_method: String,
}

/// 宠物守护进程
pub struct PetDaemon {
    /// 所有宠物
    pets: HashMap<String, Pet>,
    /// 玩家宠物映射
    player_pets: HashMap<String, Vec<String>>,
    /// 宠物模板
    templates: HashMap<String, PetTemplate>,
    /// 出战宠物
    active_pets: HashMap<String, String>, // player_id -> pet_id
}

impl PetDaemon {
    /// 创建新的宠物守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            pets: HashMap::new(),
            player_pets: HashMap::new(),
            templates: HashMap::new(),
            active_pets: HashMap::new(),
        };

        daemon.init_default_templates();
        daemon
    }

    /// 初始化默认宠物模板
    fn init_default_templates(&mut self) {
        // 小火龙
        let fire_dragon = PetTemplate {
            id: "pet_fire_dragon".to_string(),
            name: "小火龙".to_string(),
            description: "拥有火焰之力的幼龙，成长后会变得非常强大。".to_string(),
            stat_bonus: PetStats {
                strength: 15,
                agility: 10,
                intelligence: 12,
                constitution: 10,
                luck: 5,
            },
            rarity: 4,
            initial_skills: vec!["skill_fire_breath".to_string()],
            obtain_method: "完成「火之试炼」任务".to_string(),
        };

        // 风鹰
        let wind_eagle = PetTemplate {
            id: "pet_wind_eagle".to_string(),
            name: "风鹰".to_string(),
            description: "翱翔于天际的猎鹰，敏捷极高。".to_string(),
            stat_bonus: PetStats {
                strength: 8,
                agility: 18,
                intelligence: 10,
                constitution: 8,
                luck: 10,
            },
            rarity: 3,
            initial_skills: vec!["skill_wind_dash".to_string()],
            obtain_method: "在悬崖峭壁间捕捉".to_string(),
        };

        // 地精
        let earth_golem = PetTemplate {
            id: "pet_earth_golem".to_string(),
            name: "地精".to_string(),
            description: "由岩石构成的生命，防御力惊人。".to_string(),
            stat_bonus: PetStats {
                strength: 12,
                agility: 5,
                intelligence: 5,
                constitution: 20,
                luck: 3,
            },
            rarity: 3,
            initial_skills: vec!["skill_earth_armor".to_string()],
            obtain_method: "完成「大地守护」任务".to_string(),
        };

        // 小灵狐
        let spirit_fox = PetTemplate {
            id: "pet_spirit_fox".to_string(),
            name: "小灵狐".to_string(),
            description: "机灵可爱的狐狸，幸运值很高。".to_string(),
            stat_bonus: PetStats {
                strength: 8,
                agility: 12,
                intelligence: 15,
                constitution: 8,
                luck: 15,
            },
            rarity: 2,
            initial_skills: vec!["skill_luck_boost".to_string()],
            obtain_method: "在灵狐森林捕获".to_string(),
        };

        self.templates.insert(fire_dragon.id.clone(), fire_dragon);
        self.templates.insert(wind_eagle.id.clone(), wind_eagle);
        self.templates.insert(earth_golem.id.clone(), earth_golem);
        self.templates.insert(spirit_fox.id.clone(), spirit_fox);
    }

    /// 获取宠物模板
    pub fn get_template(&self, template_id: &str) -> Option<&PetTemplate> {
        self.templates.get(template_id)
    }

    /// 获取所有模板
    pub fn get_all_templates(&self) -> Vec<&PetTemplate> {
        self.templates.values().collect()
    }

    /// 创建宠物
    pub fn create_pet(
        &mut self,
        template_id: &str,
        owner_id: String,
        custom_name: Option<String>,
    ) -> Result<Pet> {
        let template = self.get_template(template_id)
            .ok_or_else(|| MudError::NotFound("宠物模板不存在".to_string()))?;

        let pet_id = format!("pet_{}_{}_{}",
            owner_id,
            template_id,
            chrono::Utc::now().timestamp()
        );

        let name = custom_name.unwrap_or_else(|| template.name.clone());
        let mut pet = Pet::new(pet_id.clone(), template_id.to_string(), name, owner_id.clone());

        // 应用模板属性加成
        pet.stats.strength = template.stat_bonus.strength;
        pet.stats.agility = template.stat_bonus.agility;
        pet.stats.intelligence = template.stat_bonus.intelligence;
        pet.stats.constitution = template.stat_bonus.constitution;
        pet.stats.luck = template.stat_bonus.luck;

        // 添加初始技能
        for skill_id in &template.initial_skills {
            pet.skills.push(PetSkill {
                id: skill_id.clone(),
                name: format!("技能_{}", skill_id),
                level: 1,
                description: "初始技能".to_string(),
            });
        }

        // 保存宠物
        self.pets.insert(pet_id.clone(), pet.clone());

        // 添加到玩家宠物列表
        self.player_pets
            .entry(owner_id)
            .or_insert_with(Vec::new)
            .push(pet_id);

        Ok(pet)
    }

    /// 获取玩家的宠物列表
    pub fn get_player_pets(&self, player_id: &str) -> Vec<&Pet> {
        if let Some(pet_ids) = self.player_pets.get(player_id) {
            pet_ids.iter()
                .filter_map(|id| self.pets.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取宠物
    pub fn get_pet(&self, pet_id: &str) -> Option<&Pet> {
        self.pets.get(pet_id)
    }

    /// 获取可变宠物
    pub fn get_pet_mut(&mut self, pet_id: &str) -> Option<&mut Pet> {
        self.pets.get_mut(pet_id)
    }

    /// 设置出战宠物
    pub fn set_active_pet(&mut self, player_id: &str, pet_id: &str) -> Result<()> {
        // 验证宠物属于该玩家
        if let Some(pet_ids) = self.player_pets.get(player_id) {
            if !pet_ids.contains(&pet_id.to_string()) {
                return Err(MudError::PermissionDenied);
            }
        } else {
            return Err(MudError::NotFound("没有宠物".to_string()));
        }

        // 验证宠物状态
        if let Some(pet) = self.get_pet(pet_id) {
            if pet.status == PetStatus::Dead {
                return Err(MudError::RuntimeError("宠物已死亡".to_string()));
            }
        }

        self.active_pets.insert(player_id.to_string(), pet_id.to_string());
        Ok(())
    }

    /// 获取出战宠物
    pub fn get_active_pet(&self, player_id: &str) -> Option<&Pet> {
        if let Some(pet_id) = self.active_pets.get(player_id) {
            self.get_pet(pet_id)
        } else {
            None
        }
    }

    /// 获取可变出战宠物
    pub fn get_active_pet_mut(&mut self, player_id: &str) -> Option<&mut Pet> {
        let pet_id = self.active_pets.get(player_id)?.clone();
        self.get_pet_mut(&pet_id)
    }

    /// 收回宠物
    pub fn recall_pet(&mut self, player_id: &str) -> Result<()> {
        if let Some(pet_id) = self.active_pets.get(player_id).cloned() {
            if let Some(pet) = self.get_pet_mut(&pet_id) {
                pet.status = PetStatus::Resting;
            }
            self.active_pets.remove(player_id);
            Ok(())
        } else {
            Err(MudError::NotFound("没有出战宠物".to_string()))
        }
    }

    /// 放生宠物
    pub fn release_pet(&mut self, player_id: &str, pet_id: &str) -> Result<()> {
        // 验证宠物属于该玩家
        if let Some(pet_ids) = self.player_pets.get_mut(player_id) {
            if !pet_ids.contains(&pet_id.to_string()) {
                return Err(MudError::PermissionDenied);
            }
            // 从列表移除
            pet_ids.retain(|id| id != pet_id);
        } else {
            return Err(MudError::NotFound("没有宠物".to_string()));
        }

        // 如果是出战宠物，先收回
        if self.active_pets.get(player_id).map_or(false, |id| id == pet_id) {
            self.active_pets.remove(player_id);
        }

        // 删除宠物
        self.pets.remove(pet_id);
        Ok(())
    }

    /// 更新所有宠物状态（定时调用）
    pub fn update_all_pets(&mut self) {
        for pet in self.pets.values_mut() {
            pet.update();
        }
    }

    /// 喂养宠物
    pub fn feed_pet(&mut self, player_id: &str, pet_id: &str, amount: u32) -> Result<String> {
        // 验证宠物属于该玩家
        if let Some(pet_ids) = self.player_pets.get(player_id) {
            if !pet_ids.contains(&pet_id.to_string()) {
                return Err(MudError::PermissionDenied);
            }
        }

        let pet = self.get_pet_mut(pet_id)
            .ok_or_else(|| MudError::NotFound("宠物不存在".to_string()))?;

        pet.feed(amount)?;
        Ok(format!("你喂了{}，它看起来很开心！", pet.name))
    }

    /// 抚摸宠物
    pub fn pet_pet(&mut self, player_id: &str, pet_id: &str) -> Result<String> {
        // 验证宠物属于该玩家
        if let Some(pet_ids) = self.player_pets.get(player_id) {
            if !pet_ids.contains(&pet_id.to_string()) {
                return Err(MudError::PermissionDenied);
            }
        }

        let pet = self.get_pet_mut(pet_id)
            .ok_or_else(|| MudError::NotFound("宠物不存在".to_string()))?;

        pet.pet()?;
        Ok(format!("你摸了摸{}，它摇着尾巴很开心！", pet.name))
    }

    /// 格式化宠物列表
    pub fn format_pet_list(&self, player_id: &str) -> String {
        let pets = self.get_player_pets(player_id);
        let active_id = self.active_pets.get(player_id).cloned();

        let mut output = format!("§H=== 宠物列表 ({}只) ===§N\n", pets.len());

        if pets.is_empty() {
            output.push_str("你还没有宠物。\n");
        } else {
            for pet in pets {
                let active_mark = active_id.as_ref().map_or(false, |id| id == &pet.id);
                let mark = if active_mark { "§G[出战]§N" } else { "" };

                output.push_str(&format!(
                    "{} {} [Lv.{}] {}\n",
                    mark,
                    pet.name,
                    pet.level,
                    match pet.status {
                        PetStatus::Resting => "休息中",
                        PetStatus::Following => "跟随中",
                        PetStatus::Fighting => "战斗中",
                        PetStatus::Dead => "§R已死亡§N",
                    }
                ));
            }
        }

        output
    }
}

impl Default for PetDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局宠物守护进程
pub static PETD: std::sync::OnceLock<RwLock<PetDaemon>> = std::sync::OnceLock::new();

/// 获取宠物守护进程
pub fn get_petd() -> &'static RwLock<PetDaemon> {
    PETD.get_or_init(|| RwLock::new(PetDaemon::default()))
}
