// gamenv/combat/skill.rs - 技能系统
// 对应 txpike9 中的技能系统

use crate::core::*;
use crate::gamenv::combat::{Combatant, CombatStats, DamageResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 技能类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SkillType {
    /// 主动攻击
    ActiveAttack,
    /// 被动增益
    PassiveBuff,
    /// 主动增益
    ActiveBuff,
    /// 治疗
    Heal,
    /// 控制技能
    Control,
    /// 位移技能
    Movement,
}

/// 技能目标类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SkillTarget {
    /// 自身
    Self_,
    /// 单个敌人
    SingleEnemy,
    /// 多个敌人
    MultipleEnemies,
    /// 单个友方
    SingleAlly,
    /// 多个友方
    MultipleAllies,
    /// 地面范围
    Area,
}

/// 技能效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffect {
    /// 造成伤害 (倍率)
    Damage(f32),
    /// 固定伤害
    FixedDamage(u32),
    /// 治疗百分比
    HealPercent(f32),
    /// 固定治疗
    HealFixed(u32),
    /// 增加攻击 (固定值, 持续回合)
    BuffAttack(u32, u32),
    /// 增加防御 (固定值, 持续回合)
    BuffDefense(u32, u32),
    /// 眩晕 (回合)
    Stun(u32),
}

/// 技能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// 技能ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能中文名
    pub name_cn: String,
    /// 技能类型
    pub skill_type: SkillType,
    /// 技能目标类型
    pub target_type: SkillTarget,
    /// 技能效果
    pub effects: Vec<SkillEffect>,
    /// 消耗内力
    pub qi_cost: u32,
    /// 冷却时间 (回合)
    pub cooldown: u32,
    /// 当前冷却
    pub current_cooldown: u32,
    /// 要求等级
    pub required_level: u32,
    /// 技能描述
    pub description: String,
}

impl Skill {
    /// 创建新技能
    pub fn new(id: String, name_cn: String, skill_type: SkillType) -> Self {
        Self {
            id: id.clone(),
            name: id.clone(),
            name_cn,
            skill_type,
            target_type: SkillTarget::SingleEnemy,
            effects: Vec::new(),
            qi_cost: 10,
            cooldown: 3,
            current_cooldown: 0,
            required_level: 1,
            description: String::new(),
        }
    }

    /// 设置目标类型
    pub fn with_target(mut self, target: SkillTarget) -> Self {
        self.target_type = target;
        self
    }

    /// 设置效果
    pub fn with_effects(mut self, effects: Vec<SkillEffect>) -> Self {
        self.effects = effects;
        self
    }

    /// 设置内力消耗
    pub fn with_qi_cost(mut self, cost: u32) -> Self {
        self.qi_cost = cost;
        self
    }

    /// 设置冷却时间
    pub fn with_cooldown(mut self, cooldown: u32) -> Self {
        self.cooldown = cooldown;
        self
    }

    /// 设置要求等级
    pub fn with_required_level(mut self, level: u32) -> Self {
        self.required_level = level;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    /// 检查是否可以使用
    pub fn can_use(&self, caster_qi: u32, caster_level: u32) -> bool {
        self.current_cooldown == 0
            && caster_qi >= self.qi_cost
            && caster_level >= self.required_level
    }

    /// 使用技能
    pub fn use_skill(
        &self,
        caster: &mut impl Combatant,
        target: &mut impl Combatant,
    ) -> Result<Vec<SkillResult>> {
        // 先检查是否可以使用
        let caster_stats_before = caster.get_combat_stats();
        if !self.can_use(
            caster_stats_before.qi,
            caster.get_level(),
        ) {
            return Err(MudError::InvalidOperation("技能无法使用".to_string()));
        }

        // 获取施法者名称（在借用之前）
        let caster_name = caster.get_name().to_string();

        let mut results = Vec::new();
        let caster_stats = caster.get_combat_stats_mut();

        // 消耗内力
        caster_stats.qi = caster_stats.qi.saturating_sub(self.qi_cost);

        // 计算技能效果
        for effect in &self.effects {
            let result = match effect {
                SkillEffect::Damage(multiplier) => {
                    let base_damage = (caster_stats.attack as f32 * multiplier) as u32;
                    let damage = base_damage.saturating_sub(target.get_combat_stats().defense);
                    let final_damage = damage.max(1);
                    target.get_combat_stats_mut().take_damage(final_damage);
                    SkillResult {
                        effect_type: "damage".to_string(),
                        value: final_damage,
                        target: target.get_name().to_string(),
                    }
                }
                SkillEffect::FixedDamage(damage) => {
                    let final_damage = damage.saturating_sub(target.get_combat_stats().defense);
                    let final_damage = final_damage.max(1);
                    target.get_combat_stats_mut().take_damage(final_damage);
                    SkillResult {
                        effect_type: "damage".to_string(),
                        value: final_damage,
                        target: target.get_name().to_string(),
                    }
                }
                SkillEffect::HealPercent(percent) => {
                    let heal_amount = (caster_stats.max_hp as f32 * percent / 100.0) as u32;
                    caster_stats.heal(heal_amount);
                    SkillResult {
                        effect_type: "heal".to_string(),
                        value: heal_amount,
                        target: caster_name.clone(),
                    }
                }
                SkillEffect::HealFixed(amount) => {
                    caster_stats.heal(*amount);
                    SkillResult {
                        effect_type: "heal".to_string(),
                        value: *amount,
                        target: caster_name.clone(),
                    }
                }
                _ => SkillResult {
                    effect_type: "unknown".to_string(),
                    value: 0,
                    target: String::new(),
                }
            };
            results.push(result);
        }

        // 设置冷却
        // 注意：这里需要在外部处理技能的current_cooldown

        Ok(results)
    }

    /// 减少冷却
    pub fn reduce_cooldown(&mut self) {
        if self.current_cooldown > 0 {
            self.current_cooldown -= 1;
        }
    }

    /// 重置冷却
    pub fn reset_cooldown(&mut self) {
        self.current_cooldown = 0;
    }

    /// 渲染技能信息
    pub fn render_info(&self) -> String {
        let mut info = format!("{}【{}】{}§r\n",
            if self.current_cooldown == 0 { "§g" } else { "§c" },
            self.name_cn,
            if self.current_cooldown > 0 {
                format!(" (冷却: {})", self.current_cooldown)
            } else {
                String::new()
            }
        );

        if !self.description.is_empty() {
            info.push_str(&format!("{}\n", self.description));
        }

        info.push_str(&format!("内力消耗: {}\n", self.qi_cost));
        info.push_str(&format!("冷却时间: {}回合\n", self.cooldown));
        info.push_str(&format!("要求等级: {}\n", self.required_level));

        info
    }
}

/// 技能结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    /// 效果类型
    pub effect_type: String,
    /// 效果值
    pub value: u32,
    /// 目标名称
    pub target: String,
}

/// 已学技能状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedSkill {
    /// 技能数据
    pub skill: Skill,
    /// 技能熟练度 (0-100)
    pub proficiency: u32,
    /// 技能使用次数
    pub use_count: u32,
}

impl LearnedSkill {
    /// 创建新学习的技能
    pub fn new(skill: Skill) -> Self {
        Self {
            skill,
            proficiency: 0,
            use_count: 0,
        }
    }

    /// 使用技能后更新
    pub fn after_use(&mut self) {
        self.use_count += 1;
        // 每10次使用提升1点熟练度
        if self.use_count % 10 == 0 && self.proficiency < 100 {
            self.proficiency = (self.proficiency + 1).min(100);
        }
    }

    /// 获取熟练度加成
    pub fn proficiency_bonus(&self) -> f32 {
        1.0 + (self.proficiency as f32 / 100.0) * 0.5 // 最高50%加成
    }
}

/// 预设技能列表
pub fn create_preset_skills() -> Vec<Skill> {
    vec![
        // 基础技能
        Skill::new(
            "skill_basic_attack".to_string(),
            "基础攻击".to_string(),
            SkillType::ActiveAttack,
        )
        .with_effects(vec![SkillEffect::Damage(1.0)])
        .with_qi_cost(0)
        .with_cooldown(0)
        .with_required_level(1)
        .with_description("最基础的攻击技能。".to_string()),

        Skill::new(
            "skill_power_strike".to_string(),
            "强力一击".to_string(),
            SkillType::ActiveAttack,
        )
        .with_effects(vec![SkillEffect::Damage(1.5)])
        .with_qi_cost(20)
        .with_cooldown(3)
        .with_required_level(5)
        .with_description("集中力量进行一次强力攻击。".to_string()),

        Skill::new(
            "skill_quick_strike".to_string(),
            "快速连击".to_string(),
            SkillType::ActiveAttack,
        )
        .with_effects(vec![SkillEffect::Damage(0.6), SkillEffect::Damage(0.6)])
        .with_qi_cost(15)
        .with_cooldown(2)
        .with_required_level(3)
        .with_description("快速连续攻击两次。".to_string()),

        Skill::new(
            "skill_heal".to_string(),
            "治疗术".to_string(),
            SkillType::Heal,
        )
        .with_effects(vec![SkillEffect::HealPercent(30.0)])
        .with_qi_cost(30)
        .with_cooldown(5)
        .with_required_level(10)
        .with_description("恢复自身30%的生命值。".to_string()),

        Skill::new(
            "skill_defense".to_string(),
            "防御姿态".to_string(),
            SkillType::ActiveBuff,
        )
        .with_effects(vec![SkillEffect::BuffDefense(20, 3)])
        .with_qi_cost(10)
        .with_cooldown(4)
        .with_required_level(3)
        .with_description("提升防御力，持续3回合。".to_string()),

        // 高级技能
        Skill::new(
            "skill_critical_strike".to_string(),
            "致命一击".to_string(),
            SkillType::ActiveAttack,
        )
        .with_effects(vec![SkillEffect::Damage(2.5)])
        .with_qi_cost(50)
        .with_cooldown(5)
        .with_required_level(20)
        .with_description("以全身力量发动致命一击。".to_string()),

        Skill::new(
            "skill_whirlwind".to_string(),
            "旋风斩".to_string(),
            SkillType::ActiveAttack,
        )
        .with_effects(vec![SkillEffect::Damage(1.2)])
        .with_target(SkillTarget::MultipleEnemies)
        .with_qi_cost(40)
        .with_cooldown(4)
        .with_required_level(15)
        .with_description("攻击周围所有敌人。".to_string()),
    ]
}

/// 技能管理器
pub struct SkillManager {
    /// 技能模板
    skills: HashMap<String, Skill>,
}

impl SkillManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            skills: HashMap::new(),
        };

        // 注册预设技能
        for skill in create_preset_skills() {
            mgr.skills.insert(skill.id.clone(), skill);
        }

        mgr
    }

    /// 获取技能
    pub fn get_skill(&self, skill_id: &str) -> Option<&Skill> {
        self.skills.get(skill_id)
    }

    /// 学习技能
    pub fn learn_skill(&mut self, skill_id: &str) -> Result<LearnedSkill> {
        if let Some(skill) = self.skills.get(skill_id) {
            Ok(LearnedSkill::new(skill.clone()))
        } else {
            Err(MudError::NotFound(format!("技能不存在: {}", skill_id)))
        }
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局技能管理器
pub static SKILLD: once_cell::sync::Lazy<std::sync::Mutex<SkillManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(SkillManager::default()));
