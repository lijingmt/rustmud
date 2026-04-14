// gamenv/single/skills/skill_system.rs - 增强技能系统
// 完全复刻 txpike9 的技能系统架构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::perform::{Perform, DamageType, get_performd};

/// 技能类型 - 对应 txpike9 的技能分类
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SkillCategory {
    /// 基础技能 (拳脚、兵器、轻功、内功等)
    Basic,
    /// 特殊武功 (各门派独门武功)
    Special,
    /// 被动技能
    Passive,
}

/// 基础技能类型 - txpike9 中的 enable 机制
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BasicSkillType {
    /// 徒手 - unarmed
    Unarmed,
    /// 剑法 - sword
    Sword,
    /// 刀法 - blade/saber
    Blade,
    /// 棍法 - spear/staff
    Spear,
    /// 棍法 - club/stick
    Club,
    /// 暗器 - hidden weapon
    HiddenWeapon,
    /// 轻功 - movement/dodge
    Movement,
    /// 内功 - force/qi cultivation
    Force,
    /// 招架 - parry
    Parry,
    /// 闪避 - dodge
    Dodge,
}

impl BasicSkillType {
    /// 获取技能ID前缀
    pub fn id_prefix(&self) -> &str {
        match self {
            BasicSkillType::Unarmed => "unarmed",
            BasicSkillType::Sword => "sword",
            BasicSkillType::Blade => "blade",
            BasicSkillType::Spear => "spear",
            BasicSkillType::Club => "club",
            BasicSkillType::HiddenWeapon => "hidden",
            BasicSkillType::Movement => "dodge",
            BasicSkillType::Force => "force",
            BasicSkillType::Parry => "parry",
            BasicSkillType::Dodge => "dodge",
        }
    }

    /// 获取中文名称
    pub fn cn_name(&self) -> &str {
        match self {
            BasicSkillType::Unarmed => "拳脚",
            BasicSkillType::Sword => "剑法",
            BasicSkillType::Blade => "刀法",
            BasicSkillType::Spear => "枪棒",
            BasicSkillType::Club => "棍棒",
            BasicSkillType::HiddenWeapon => "暗器",
            BasicSkillType::Movement => "轻功",
            BasicSkillType::Force => "内功",
            BasicSkillType::Parry => "招架",
            BasicSkillType::Dodge => "闪避",
        }
    }
}

/// 技能熟练度曲线 - 对应 txpike9 的衰减机制
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProficiencyCurve {
    /// 初始等级
    pub initial: i32,
    /// 衰减系数
    pub attenuation: i32,
    /// 最大等级
    pub max_level: i32,
}

impl ProficiencyCurve {
    /// 创建默认熟练度曲线
    pub fn default() -> Self {
        Self {
            initial: 10,
            attenuation: 100,
            max_level: 200,
        }
    }

    /// 计算当前等级所需经验
    pub fn exp_for_level(&self, level: i32) -> i64 {
        if level <= self.initial {
            return 0;
        }

        // txpike9 风格的衰减公式
        let diff = level - self.initial;
        let attenuation = self.attenuation as f64;

        // 经验公式: base * (level^2 / attenuation)
        ((diff * diff * 100) as f64 / attenuation) as i64
    }

    /// 根据经验计算等级
    pub fn level_from_exp(&self, exp: i64) -> i32 {
        if exp == 0 {
            return self.initial;
        }

        let attenuation = self.attenuation as f64;
        let exp_f = exp as f64;

        // 反向计算: level = sqrt(exp * attenuation / 100) + initial
        let level = (exp_f * attenuation / 100.0).sqrt() as i32 + self.initial;

        level.min(self.max_level)
    }

    /// 计算当前等级的加成百分比
    pub fn bonus_percent(&self, level: i32) -> f32 {
        if level >= self.max_level {
            return 2.0; // 200% 加成
        }

        // 渐进式加成: 50% + 1.5% * level
        (50 + (level * 3 / 2)) as f32 / 100.0
    }
}

/// 玩家技能数据 - 对应 txpike9 的 skills mapping
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSkillData {
    /// 技能等级
    pub level: i32,
    /// 技能点数 (熟练度)
    pub points: i64,
    /// 使用次数
    pub use_count: u64,
    /// 已学会的招式列表
    pub learned_performs: Vec<String>,
    /// 是否启用 (enable mapping for special skills)
    pub enabled: bool,
}

impl PlayerSkillData {
    /// 创建新技能数据
    pub fn new(initial_level: i32) -> Self {
        Self {
            level: initial_level,
            points: 0,
            use_count: 0,
            learned_performs: Vec::new(),
            enabled: true,
        }
    }

    /// 增加经验
    pub fn add_exp(&mut self, exp: i64, curve: &ProficiencyCurve) -> bool {
        self.points += exp;
        let new_level = curve.level_from_exp(self.points);

        if new_level > self.level {
            self.level = new_level;
            true
        } else {
            false
        }
    }

    /// 记录使用
    pub fn record_use(&mut self) {
        self.use_count += 1;
        // 每10次使用获得1点熟练度
        if self.use_count % 10 == 0 {
            self.points += 1;
        }
    }

    /// 学习招式
    pub fn learn_perform(&mut self, perform_id: String) -> bool {
        if self.learned_performs.contains(&perform_id) {
            false
        } else {
            self.learned_performs.push(perform_id);
            true
        }
    }

    /// 检查是否学会招式
    pub fn has_perform(&self, perform_id: &str) -> bool {
        self.learned_performs.iter().any(|p| p == perform_id)
    }

    /// 获取技能加成
    pub fn get_bonus(&self, curve: &ProficiencyCurve) -> f32 {
        curve.bonus_percent(self.level)
    }
}

/// 增强技能定义
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnhancedSkill {
    /// 技能ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能中文名
    pub name_cn: String,
    /// 技能类型
    pub category: SkillCategory,
    /// 基础技能类型 (如果是基础技能)
    pub basic_type: Option<BasicSkillType>,
    /// 所属门派
    pub school: String,
    /// 熟练度曲线
    pub curve: ProficiencyCurve,
    /// 需要的前置技能
    pub prerequisites: Vec<String>,
    /// 需要的属性要求
    pub stat_requirements: StatRequirements,
    /// 技能效果
    pub effects: SkillEffects,
    /// 描述
    pub description: String,
}

/// 属性要求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatRequirements {
    /// 根骨要求 (影响内功学习)
    pub gen: Option<i32>,
    /// 臂力要求 (影响外功学习)
    pub str: Option<i32>,
    /// 体质要求 (影响内息)
    pub con: Option<i32>,
    /// 灵巧要求
    pub dex: Option<i32>,
    /// 悟性要求
    pub int: Option<i32>,
}

/// 技能效果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillEffects {
    /// 攻击力加成
    pub attack_bonus: i32,
    /// 防御力加成
    pub defense_bonus: i32,
    /// 闪避率加成
    pub dodge_bonus: i32,
    /// 招架率加成
    pub parry_bonus: i32,
    /// 命中率加成
    pub hit_bonus: i32,
    /// 暴击率加成
    pub crit_bonus: i32,
    /// 暴击伤害加成
    pub crit_damage_bonus: i32,
    /// HP 加成
    pub hp_bonus: i32,
    /// 内力加成
    pub qi_bonus: i32,
}

impl Default for SkillEffects {
    fn default() -> Self {
        Self {
            attack_bonus: 0,
            defense_bonus: 0,
            dodge_bonus: 0,
            parry_bonus: 0,
            hit_bonus: 0,
            crit_bonus: 0,
            crit_damage_bonus: 0,
            hp_bonus: 0,
            qi_bonus: 0,
        }
    }
}

impl EnhancedSkill {
    /// 检查玩家是否满足学习条件
    pub fn can_learn(&self, player_stats: &PlayerStats, learned_skills: &[String]) -> bool {
        // 检查属性要求
        if let Some(req) = self.stat_requirements.gen {
            if player_stats.gen < req {
                return false;
            }
        }
        if let Some(req) = self.stat_requirements.str {
            if player_stats.str < req {
                return false;
            }
        }
        if let Some(req) = self.stat_requirements.con {
            if player_stats.con < req {
                return false;
            }
        }

        // 检查前置技能
        for prereq in &self.prerequisites {
            if !learned_skills.contains(prereq) {
                return false;
            }
        }

        true
    }

    /// 计算技能对玩家属性的加成
    pub fn calculate_bonuses(&self, player_level: i32) -> SkillEffects {
        let bonus_percent = self.curve.bonus_percent(player_level);

        SkillEffects {
            attack_bonus: (self.effects.attack_bonus as f32 * bonus_percent) as i32,
            defense_bonus: (self.effects.defense_bonus as f32 * bonus_percent) as i32,
            dodge_bonus: (self.effects.dodge_bonus as f32 * bonus_percent) as i32,
            parry_bonus: (self.effects.parry_bonus as f32 * bonus_percent) as i32,
            hit_bonus: (self.effects.hit_bonus as f32 * bonus_percent) as i32,
            crit_bonus: (self.effects.crit_bonus as f32 * bonus_percent) as i32,
            crit_damage_bonus: (self.effects.crit_damage_bonus as f32 * bonus_percent) as i32,
            hp_bonus: (self.effects.hp_bonus as f32 * bonus_percent) as i32,
            qi_bonus: (self.effects.qi_bonus as f32 * bonus_percent) as i32,
        }
    }

    /// 格式化技能信息
    pub fn format_info(&self, player_data: Option<&PlayerSkillData>) -> String {
        let level_info = if let Some(data) = player_data {
            format!("Lv.{}", data.level)
        } else {
            "未学习".to_string()
        };

        let mut info = format!("§Y【{}】{}§N\n", self.name_cn, level_info);

        if let Some(basic_type) = &self.basic_type {
            info.push_str(&format!("类型: {}\n", basic_type.cn_name()));
        }

        info.push_str(&format!("门派: {}\n", self.school));
        info.push_str(&format!("{}\n\n", self.description));

        // 属性要求
        if self.stat_requirements.gen.is_some() || self.stat_requirements.str.is_some() {
            info.push_str("§H属性要求§N\n");
            if let Some(req) = self.stat_requirements.gen {
                info.push_str(&format!("根骨 ≥ {}\n", req));
            }
            if let Some(req) = self.stat_requirements.str {
                info.push_str(&format!("臂力 ≥ {}\n", req));
            }
            if let Some(req) = self.stat_requirements.con {
                info.push_str(&format!("体质 ≥ {}\n", req));
            }
            info.push_str("\n");
        }

        info
    }
}

/// 玩家属性 (用于检查学习条件)
#[derive(Clone, Debug)]
pub struct PlayerStats {
    pub gen: i32,  // 根骨
    pub str: i32,  // 臂力
    pub con: i32,  // 体质
    pub dex: i32,  // 灵巧
    pub int: i32,  // 悟性
}

/// 增强技能管理器
pub struct EnhancedSkillManager {
    /// 所有技能
    skills: HashMap<String, EnhancedSkill>,
    /// 玩家技能数据
    player_skills: HashMap<String, HashMap<String, PlayerSkillData>>,
    /// 玩家 enable 映射 (特殊技能 -> 基础技能)
    player_enable: HashMap<String, HashMap<String, String>>,
}

impl EnhancedSkillManager {
    /// 创建新管理器
    pub fn new() -> Self {
        let mut mgr = Self {
            skills: HashMap::new(),
            player_skills: HashMap::new(),
            player_enable: HashMap::new(),
        };

        mgr.init_default_skills();
        mgr
    }

    /// 初始化默认技能
    fn init_default_skills(&mut self) {
        // 基础拳脚
        self.skills.insert(
            "skill_unarmed_basic".to_string(),
            EnhancedSkill {
                id: "skill_unarmed_basic".to_string(),
                name: "unarmed_basic".to_string(),
                name_cn: "基础拳脚".to_string(),
                category: SkillCategory::Basic,
                basic_type: Some(BasicSkillType::Unarmed),
                school: "通用".to_string(),
                curve: ProficiencyCurve::default(),
                prerequisites: vec![],
                stat_requirements: StatRequirements {
                    gen: None,
                    str: None,
                    con: None,
                    dex: None,
                    int: None,
                },
                effects: SkillEffects {
                    attack_bonus: 5,
                    defense_bonus: 2,
                    ..Default::default()
                },
                description: "最基本的拳脚功夫。".to_string(),
            },
        );

        // 武堂 - 猛虎拳
        self.skills.insert(
            "skill_xionghuquan".to_string(),
            EnhancedSkill {
                id: "skill_xionghuquan".to_string(),
                name: "xionghuquan".to_string(),
                name_cn: "猛虎拳".to_string(),
                category: SkillCategory::Special,
                basic_type: Some(BasicSkillType::Unarmed),
                school: "武堂".to_string(),
                curve: ProficiencyCurve {
                    initial: 10,
                    attenuation: 80,
                    max_level: 150,
                },
                prerequisites: vec!["skill_unarmed_basic".to_string()],
                stat_requirements: StatRequirements {
                    gen: Some(20),
                    str: Some(25),
                    con: None,
                    dex: None,
                    int: None,
                },
                effects: SkillEffects {
                    attack_bonus: 30,
                    crit_bonus: 10,
                    crit_damage_bonus: 20,
                    hp_bonus: 50,
                    ..Default::default()
                },
                description: "武堂独门拳法，刚猛如虎，气势磅礴。".to_string(),
            },
        );

        // 武当 - 太极剑
        self.skills.insert(
            "skill_taiji".to_string(),
            EnhancedSkill {
                id: "skill_taiji".to_string(),
                name: "taiji".to_string(),
                name_cn: "太极剑".to_string(),
                category: SkillCategory::Special,
                basic_type: Some(BasicSkillType::Sword),
                school: "武当".to_string(),
                curve: ProficiencyCurve {
                    initial: 10,
                    attenuation: 90,
                    max_level: 180,
                },
                prerequisites: vec!["skill_unarmed_basic".to_string()],
                stat_requirements: StatRequirements {
                    gen: Some(30),
                    str: Some(15),
                    con: None,
                    dex: Some(20),
                    int: None,
                },
                effects: SkillEffects {
                    attack_bonus: 25,
                    defense_bonus: 15,
                    parry_bonus: 20,
                    qi_bonus: 100,
                    ..Default::default()
                },
                description: "武当镇派剑法，以柔克刚，四两拨千斤。".to_string(),
            },
        );

        // 少林 - 罗汉拳
        self.skills.insert(
            "skill_luohanquan".to_string(),
            EnhancedSkill {
                id: "skill_luohanquan".to_string(),
                name: "luohanquan".to_string(),
                name_cn: "罗汉拳".to_string(),
                category: SkillCategory::Special,
                basic_type: Some(BasicSkillType::Unarmed),
                school: "少林".to_string(),
                curve: ProficiencyCurve {
                    initial: 10,
                    attenuation: 85,
                    max_level: 170,
                },
                prerequisites: vec!["skill_unarmed_basic".to_string()],
                stat_requirements: StatRequirements {
                    gen: Some(15),
                    str: Some(30),
                    con: Some(20),
                    dex: None,
                    int: None,
                },
                effects: SkillEffects {
                    attack_bonus: 35,
                    defense_bonus: 10,
                    hp_bonus: 100,
                    ..Default::default()
                },
                description: "少林七十二绝技之一，刚猛有力，威震八方。".to_string(),
            },
        );

        // 华山 - 独孤九剑
        self.skills.insert(
            "skill_dugujiujian".to_string(),
            EnhancedSkill {
                id: "skill_dugujiujian".to_string(),
                name: "dugujiujian".to_string(),
                name_cn: "独孤九剑".to_string(),
                category: SkillCategory::Special,
                basic_type: Some(BasicSkillType::Sword),
                school: "华山".to_string(),
                curve: ProficiencyCurve {
                    initial: 20,
                    attenuation: 70,
                    max_level: 200,
                },
                prerequisites: vec!["skill_unarmed_basic".to_string()],
                stat_requirements: StatRequirements {
                    gen: Some(35),
                    str: Some(20),
                    con: None,
                    dex: Some(30),
                    int: Some(25),
                },
                effects: SkillEffects {
                    attack_bonus: 40,
                    hit_bonus: 30,
                    crit_bonus: 20,
                    ..Default::default()
                },
                description: "华山派镇派绝学，专破天下武学，无招胜有招。".to_string(),
            },
        );
    }

    /// 获取技能
    pub fn get_skill(&self, skill_id: &str) -> Option<&EnhancedSkill> {
        self.skills.get(skill_id)
    }

    /// 学习技能
    pub fn learn_skill(
        &mut self,
        player_id: String,
        skill_id: String,
        player_stats: &PlayerStats,
    ) -> std::result::Result<String, String> {
        let skill = self.skills.get(&skill_id)
            .ok_or_else(|| "技能不存在".to_string())?;

        let learned = self.player_skills
            .entry(player_id.clone())
            .or_insert_with(HashMap::new);

        // 检查是否已学习
        if learned.contains_key(&skill_id) {
            return Err("已经学习过这个技能".to_string());
        }

        // 检查学习条件
        let learned_ids: Vec<String> = learned.keys().cloned().collect();
        if !skill.can_learn(player_stats, &learned_ids) {
            return Err("不满足学习条件".to_string());
        }

        // 添加技能
        learned.insert(
            skill_id.clone(),
            PlayerSkillData::new(skill.curve.initial),
        );

        Ok(format!("你学会了{}！", skill.name_cn))
    }

    /// 获取玩家技能
    pub fn get_player_skills(&self, player_id: &str) -> Vec<(&EnhancedSkill, &PlayerSkillData)> {
        let mut result = Vec::new();

        if let Some(skills) = self.player_skills.get(player_id) {
            for (skill_id, player_skill) in skills {
                if let Some(skill) = self.get_skill(skill_id) {
                    result.push((skill, player_skill));
                }
            }
        }

        result
    }

    /// 使用技能 - 增加熟练度
    pub fn use_skill(&mut self, player_id: &str, skill_id: &str) -> std::result::Result<Vec<String>, String> {
        // 先获取技能信息的副本，避免借用冲突
        let skill_info = {
            let skill = self.get_skill(skill_id);
            if skill.is_none() {
                return Err("技能不存在".to_string());
            }
            skill.unwrap().clone()
        };

        let skills = self.player_skills.get_mut(player_id)
            .ok_or_else(|| "你还没有学习这个技能".to_string())?;

        let player_skill = skills.get_mut(skill_id)
            .ok_or_else(|| "你还没有学习这个技能".to_string())?;

        // 记录使用并检查升级
        player_skill.record_use();
        let leveled_up = player_skill.add_exp(10, &skill_info.curve);

        let mut results = vec![];
        results.push(format!("你使用了{}", skill_info.name_cn));

        if leveled_up {
            results.push(format!("§g你的 {} 提升到了 {} 级！§N", skill_info.name_cn, player_skill.level));
        }

        Ok(results)
    }
}

impl Default for EnhancedSkillManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局增强技能管理器
pub static ENHANCED_SKILLD: once_cell::sync::Lazy<Arc<RwLock<EnhancedSkillManager>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(EnhancedSkillManager::new())));

/// 获取全局增强技能管理器
pub fn get_enhanced_skilld() -> Arc<RwLock<EnhancedSkillManager>> {
    ENHANCED_SKILLD.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proficiency_curve() {
        let curve = ProficiencyCurve::default();

        // 浝始等级不需要经验
        assert_eq!(curve.exp_for_level(10), 0);

        // 计算经验
        let exp_20 = curve.exp_for_level(20);
        assert!(exp_20 > 0);

        // 从经验反推等级
        let level = curve.level_from_exp(exp_20);
        assert_eq!(level, 20);
    }

    #[test]
    fn test_skill_learning() {
        let mut mgr = EnhancedSkillManager::new();

        let stats = PlayerStats {
            gen: 30,
            str: 30,
            con: 20,
            dex: 20,
            int: 20,
        };

        let result = mgr.learn_skill("player1".to_string(), "skill_xionghuquan".to_string(), &stats);
        assert!(result.is_ok());
    }
}
