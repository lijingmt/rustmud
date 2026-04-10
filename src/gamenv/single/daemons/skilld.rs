// gamenv/single/daemons/skilld.rs - 技能系统守护进程
// 对应 txpike9 的技能系统

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 技能类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SkillType {
    /// 被动技能
    Passive,
    /// 主动技能
    Active,
    /// 攻击技能
    Attack,
    /// 防御技能
    Defense,
    /// 治疗技能
    Heal,
    /// 辅助技能
    Buff,
    /// 减益技能
    Debuff,
}

impl std::fmt::Display for SkillType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillType::Passive => write!(f, "被动"),
            SkillType::Active => write!(f, "主动"),
            SkillType::Attack => write!(f, "攻击"),
            SkillType::Defense => write!(f, "防御"),
            SkillType::Heal => write!(f, "治疗"),
            SkillType::Buff => write!(f, "辅助"),
            SkillType::Debuff => write!(f, "减益"),
        }
    }
}

/// 技能
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    /// 技能ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能中文名称
    pub name_cn: String,
    /// 技能类型
    pub skill_type: SkillType,
    /// 所属门派ID
    pub school: String,
    /// 等级要求
    pub level_req: i32,
    /// 消耗MP
    pub mp_cost: u32,
    /// 冷却时间（秒）
    pub cooldown: u32,
    /// 施法时间（秒）
    pub cast_time: u32,
    /// 伤害系数
    pub damage_multiplier: f32,
    /// 治疗系数
    pub heal_multiplier: f32,
    /// 描述
    pub description: String,
    /// 需要的前置技能
    pub prerequisite: Option<String>,
    /// 最大等级
    pub max_level: u32,
    /// 招式列表 (perform_id, required_level)
    #[serde(default)]
    pub performs: Vec<(String, u32)>,
}

impl Skill {
    /// 是否可以使用技能
    pub fn can_use(&self, player_level: u32, current_mp: u32) -> bool {
        if player_level < self.level_req as u32 {
            return false;
        }
        if current_mp < self.mp_cost {
            return false;
        }
        true
    }

    /// 格式化技能信息
    pub fn format(&self) -> String {
        let type_name = match self.skill_type {
            SkillType::Passive => "被动",
            SkillType::Active => "主动",
            SkillType::Attack => "攻击",
            SkillType::Defense => "防御",
            SkillType::Heal => "治疗",
            SkillType::Buff => "辅助",
            SkillType::Debuff => "减益",
        };

        format!(
            "§H[{}]§N {} - MP消耗:{} 冷却:{}秒\n{}",
            type_name,
            self.name,
            self.mp_cost,
            self.cooldown,
            self.description
        )
    }
}

/// 玩家技能
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSkill {
    /// 技能ID
    pub skill_id: String,
    /// 当前等级
    pub level: u32,
    /// 当前经验
    pub exp: u64,
    /// 学习时间
    pub learned_at: i64,
    /// 已学会的招式列表
    #[serde(default)]
    pub learned_performs: Vec<String>,
}

impl PlayerSkill {
    /// 创建新技能
    pub fn new(skill_id: String) -> Self {
        Self {
            skill_id,
            level: 1,
            exp: 0,
            learned_at: chrono::Utc::now().timestamp(),
            learned_performs: Vec::new(),
        }
    }

    /// 增加经验
    pub fn add_exp(&mut self, exp: u64) -> bool {
        self.exp += exp;

        // 简化升级公式
        let needed = self.level as u64 * 100;
        if self.exp >= needed {
            self.exp -= needed;
            self.level += 1;
            return true;
        }

        false
    }
}

/// 技能守护进程
pub struct SkillDaemon {
    /// 所有技能
    skills: HashMap<String, Skill>,
    /// 玩家技能
    player_skills: HashMap<String, HashMap<String, PlayerSkill>>,
}

impl SkillDaemon {
    /// 创建新的技能守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            skills: HashMap::new(),
            player_skills: HashMap::new(),
        };

        daemon.init_default_skills();
        daemon
    }

    /// 初始化默认技能
    fn init_default_skills(&mut self) {
        // 基础攻击
        let basic_attack = Skill {
            id: "skill_basic_attack".to_string(),
            name: "基础攻击".to_string(),
            name_cn: "基础攻击".to_string(),
            skill_type: SkillType::Attack,
            school: "wutang".to_string(),
            level_req: 1,
            mp_cost: 0,
            cooldown: 1,
            cast_time: 1,
            damage_multiplier: 1.0,
            heal_multiplier: 0.0,
            description: "最基础的攻击技能，所有冒险者都会。".to_string(),
            prerequisite: None,
            max_level: 10,
            performs: vec![],
        };

        // 火球术
        let fireball = Skill {
            id: "skill_fireball".to_string(),
            name: "火球术".to_string(),
            name_cn: "火球术".to_string(),
            skill_type: SkillType::Attack,
            school: "huashan".to_string(),
            level_req: 5,
            mp_cost: 20,
            cooldown: 3,
            cast_time: 2,
            damage_multiplier: 2.5,
            heal_multiplier: 0.0,
            description: "向敌人投掷火球，造成大量魔法伤害。".to_string(),
            prerequisite: Some("skill_basic_attack".to_string()),
            max_level: 10,
            performs: vec![],
        };

        // 治愈术
        let heal = Skill {
            id: "skill_heal".to_string(),
            name: "治愈术".to_string(),
            name_cn: "治愈术".to_string(),
            skill_type: SkillType::Heal,
            school: "wudang".to_string(),
            level_req: 3,
            mp_cost: 15,
            cooldown: 5,
            cast_time: 2,
            damage_multiplier: 0.0,
            heal_multiplier: 1.5,
            description: "恢复自身HP值。".to_string(),
            prerequisite: None,
            max_level: 10,
            performs: vec![],
        };

        // 强力防御
        let power_defense = Skill {
            id: "skill_power_defense".to_string(),
            name: "强力防御".to_string(),
            name_cn: "强力防御".to_string(),
            skill_type: SkillType::Defense,
            school: "shaolin".to_string(),
            level_req: 5,
            mp_cost: 10,
            cooldown: 10,
            cast_time: 0,
            damage_multiplier: 0.0,
            heal_multiplier: 0.0,
            description: "暂时提升防御力，减少受到的伤害。".to_string(),
            prerequisite: None,
            max_level: 5,
            performs: vec![],
        };

        // 暴击训练
        let crit_training = Skill {
            id: "skill_crit_training".to_string(),
            name: "暴击训练".to_string(),
            name_cn: "暴击训练".to_string(),
            skill_type: SkillType::Passive,
            school: "wutang".to_string(),
            level_req: 10,
            mp_cost: 0,
            cooldown: 0,
            cast_time: 0,
            damage_multiplier: 0.0,
            heal_multiplier: 0.0,
            description: "被动技能：永久提升暴击率。".to_string(),
            prerequisite: Some("skill_basic_attack".to_string()),
            max_level: 5,
            performs: vec![],
        };

        self.skills.insert(basic_attack.id.clone(), basic_attack);
        self.skills.insert(fireball.id.clone(), fireball);
        self.skills.insert(heal.id.clone(), heal);
        self.skills.insert(power_defense.id.clone(), power_defense);
        self.skills.insert(crit_training.id.clone(), crit_training);
    }

    /// 获取技能
    pub fn get_skill(&self, skill_id: &str) -> Option<&Skill> {
        self.skills.get(skill_id)
    }

    /// 获取所有技能
    pub fn get_all_skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// 学习技能
    pub fn learn_skill(&mut self, userid: String, skill_id: String) -> Result<()> {
        // 检查技能是否存在
        let skill = self.skills.get(&skill_id)
            .ok_or_else(|| MudError::NotFound("技能不存在".to_string()))?;

        // 检查是否已学习
        if let Some(skills) = self.player_skills.get(&userid) {
            if skills.contains_key(&skill_id) {
                return Err(MudError::RuntimeError("已学习此技能".to_string()));
            }
        }

        // 检查前置技能
        if let Some(ref prereq) = skill.prerequisite {
            if let Some(skills) = self.player_skills.get(&userid) {
                if !skills.contains_key(prereq) {
                    return Err(MudError::RuntimeError("需要先学习前置技能".to_string()));
                }
            } else {
                return Err(MudError::RuntimeError("需要先学习前置技能".to_string()));
            }
        }

        // 添加技能
        self.player_skills
            .entry(userid)
            .or_insert_with(HashMap::new)
            .insert(skill_id.clone(), PlayerSkill::new(skill_id.clone()));

        Ok(())
    }

    /// 获取玩家技能
    pub fn get_player_skills(&self, userid: &str) -> Vec<(&Skill, &PlayerSkill)> {
        let mut result = Vec::new();

        if let Some(skills) = self.player_skills.get(userid) {
            for (skill_id, player_skill) in skills {
                if let Some(skill) = self.get_skill(skill_id) {
                    result.push((skill, player_skill));
                }
            }
        }

        result
    }

    /// 使用技能
    pub fn use_skill(
        &self,
        userid: &str,
        skill_id: &str,
        player_level: u32,
        current_mp: u32,
    ) -> Result<SkillEffect> {
        let skill = self.get_skill(skill_id)
            .ok_or_else(|| MudError::NotFound("技能不存在".to_string()))?;

        // 检查是否学习
        if let Some(skills) = self.player_skills.get(userid) {
            if !skills.contains_key(skill_id) {
                return Err(MudError::RuntimeError("未学习此技能".to_string()));
            }
        } else {
            return Err(MudError::RuntimeError("未学习此技能".to_string()));
        }

        // 检查是否可以使用
        if !skill.can_use(player_level, current_mp) {
            return Err(MudError::RuntimeError("无法使用此技能".to_string()));
        }

        // 计算效果
        let effect = SkillEffect {
            damage: (skill.damage_multiplier * 100.0) as u32,
            heal: (skill.heal_multiplier * 50.0) as u32,
            mp_cost: skill.mp_cost,
            cooldown: skill.cooldown,
        };

        Ok(effect)
    }

    /// 格式化技能列表
    pub fn format_skill_list(&self, skills: &[(&Skill, &PlayerSkill)]) -> String {
        let mut output = String::from("§H=== 技能列表 ===§N\n");

        if skills.is_empty() {
            output.push_str("你还没有学习任何技能。\n");
        } else {
            for (skill, player_skill) in skills {
                let type_name = match skill.skill_type {
                    SkillType::Passive => "被动",
                    SkillType::Active => "主动",
                    SkillType::Attack => "攻击",
                    SkillType::Defense => "防御",
                    SkillType::Heal => "治疗",
                    SkillType::Buff => "辅助",
                    SkillType::Debuff => "减益",
                };
                output.push_str(&format!(
                    "§Y[{} Lv.{}]§N [{}] {}\n",
                    skill.name,
                    player_skill.level,
                    type_name,
                    skill.description
                ));
            }
        }

        output
    }

    /// 格式化可学技能列表
    pub fn format_available_skills(&self, player_level: u32, learned: &[String]) -> String {
        let mut output = String::from("§H=== 可学技能 ===§N\n");

        let mut available: Vec<_> = self.skills.values()
            .filter(|skill| {
                player_level >= skill.level_req as u32
                    && !learned.contains(&skill.id)
            })
            .collect();

        available.sort_by(|a, b| a.level_req.cmp(&b.level_req));

        let is_empty = available.is_empty();

        for skill in available {
            output.push_str(&format!(
                "§Y[{}]§N Lv.{} - {}\n",
                skill.name,
                skill.level_req,
                skill.description
            ));
        }

        if is_empty {
            output.push_str("暂时没有可学的技能。\n");
        }

        output
    }
}

/// 技能效果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillEffect {
    /// 伤害
    pub damage: u32,
    /// 治疗
    pub heal: u32,
    /// MP消耗
    pub mp_cost: u32,
    /// 冷却时间
    pub cooldown: u32,
}

impl Default for SkillDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局技能守护进程
pub static SKILLD: std::sync::OnceLock<RwLock<SkillDaemon>> = std::sync::OnceLock::new();

/// 获取技能守护进程
pub fn get_skilld() -> &'static RwLock<SkillDaemon> {
    SKILLD.get_or_init(|| RwLock::new(SkillDaemon::default()))
}
