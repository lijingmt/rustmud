// gamenv/single/skills/perform.rs - 招式系统
// 对应 txpike9/gamenv/single/performs/
// 招式是技能的具体表现形式，每个技能包含多个招式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 招式 - 技能的具体表现形式
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Perform {
    /// 招式ID
    pub id: String,
    /// 招式名称
    pub name: String,
    /// 招式中文名
    pub name_cn: String,
    /// 所属技能ID
    pub skill_id: String,
    /// 需要技能等级
    pub required_level: u32,
    /// 消耗内力
    pub qi_cost: u32,
    /// 消耗精神
    pub shen_cost: u32,
    /// 攻击系数
    pub attack_factor: f32,
    /// 额外伤害类型
    pub damage_type: DamageType,
    /// 额外伤害值
    pub bonus_damage: i32,
    /// 气伤 (对敌人内力造成伤害)
    pub qi_damage: u32,
    /// 身伤 (对敌人身体造成伤害)
    pub shen_damage: u32,
    /// 精伤 (对敌人精神造成伤害)
    pub jing_damage: u32,
    /// 是否群攻
    pub is_aoe: bool,
    /// 眩晕回合
    pub stun_rounds: u32,
    /// 冷却时间
    pub cooldown: u32,
    /// 描述
    pub description: String,
}

/// 伤害类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DamageType {
    /// 普通
    Normal,
    /// 毒
    Poison,
    /// 冰
    Ice,
    /// 火
    Fire,
    /// 雷
    Thunder,
    /// 内伤
    Internal,
}

impl Perform {
    /// 创建新招式
    pub fn new(
        id: String,
        name_cn: String,
        skill_id: String,
        required_level: u32,
    ) -> Self {
        Self {
            id,
            name: name_cn.clone(),
            name_cn,
            skill_id,
            required_level,
            qi_cost: 10,
            shen_cost: 0,
            attack_factor: 1.0,
            damage_type: DamageType::Normal,
            bonus_damage: 0,
            qi_damage: 0,
            shen_damage: 0,
            jing_damage: 0,
            is_aoe: false,
            stun_rounds: 0,
            cooldown: 1,
            description: String::new(),
        }
    }

    /// 设置消耗
    pub fn with_cost(mut self, qi_cost: u32, shen_cost: u32) -> Self {
        self.qi_cost = qi_cost;
        self.shen_cost = shen_cost;
        self
    }

    /// 设置攻击系数
    pub fn with_attack(mut self, factor: f32) -> Self {
        self.attack_factor = factor;
        self
    }

    /// 设置伤害类型
    pub fn with_damage_type(mut self, dtype: DamageType) -> Self {
        self.damage_type = dtype;
        self
    }

    /// 设置特殊伤害
    pub fn with_special_damage(mut self, qi: u32, shen: u32, jing: u32) -> Self {
        self.qi_damage = qi;
        self.shen_damage = shen;
        self.jing_damage = jing;
        self
    }

    /// 设置额外伤害
    pub fn with_bonus_damage(mut self, bonus: i32) -> Self {
        self.bonus_damage = bonus;
        self
    }

    /// 设置群攻
    pub fn with_aoe(mut self, is_aoe: bool) -> Self {
        self.is_aoe = is_aoe;
        self
    }

    /// 设置冷却
    pub fn with_cooldown(mut self, cd: u32) -> Self {
        self.cooldown = cd;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    /// 计算最终伤害
    pub fn calculate_damage(&self, attacker_attack: u32, defender_defense: u32) -> PerformResult {
        let base_damage = (attacker_attack as f32 * self.attack_factor) as u32;
        let mut final_damage = base_damage.saturating_sub(defender_defense);
        final_damage = final_damage.max(1);

        // 加上额外伤害
        final_damage = final_damage.saturating_add(self.bonus_damage as u32);

        PerformResult {
            damage: final_damage,
            qi_damage: self.qi_damage,
            shen_damage: self.shen_damage,
            jing_damage: self.jing_damage,
            damage_type: self.damage_type.clone(),
            is_aoe: self.is_aoe,
            stun_rounds: self.stun_rounds,
        }
    }

    /// 格式化招式信息
    pub fn format_info(&self, current_cd: u32) -> String {
        let cd_text = if current_cd > 0 {
            format!(" §c(冷却:{})§N", current_cd)
        } else {
            String::new()
        };

        format!(
            "§Y【{}】§N {}级\n消耗:内力{} 精神{}\n攻击系数: {:.1}{}\n{}",
            self.name_cn,
            self.required_level,
            self.qi_cost,
            self.shen_cost,
            self.attack_factor,
            cd_text,
            self.description
        )
    }
}

/// 招式结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformResult {
    /// 伤害值
    pub damage: u32,
    /// 内力伤害
    pub qi_damage: u32,
    /// 身体伤害
    pub shen_damage: u32,
    /// 精神伤害
    pub jing_damage: u32,
    /// 伤害类型
    pub damage_type: DamageType,
    /// 是否群攻
    pub is_aoe: bool,
    /// 眩晕回合
    pub stun_rounds: u32,
}

/// 招式管理器
pub struct PerformManager {
    /// 所有招式
    performs: HashMap<String, Perform>,
}

impl PerformManager {
    /// 创建新招式管理器
    pub fn new() -> Self {
        let mut mgr = Self {
            performs: HashMap::new(),
        };

        mgr.init_default_performs();
        mgr
    }

    /// 初始化默认招式
    fn init_default_performs(&mut self) {
        // 基础攻击招式
        self.performs.insert(
            "perform_basic_attack_1".to_string(),
            Perform::new("perform_basic_attack_1".to_string(), "普通攻击".to_string(), "skill_basic_attack".to_string(), 1)
                .with_cost(0, 0)
                .with_attack(1.0)
                .with_cooldown(0)
                .with_description("最基础的攻击招式。".to_string()),
        );

        // 武堂 - 猛虎拳招式
        self.performs.insert(
            "perform_xionghuquan_1".to_string(),
            Perform::new("perform_xionghuquan_1".to_string(), "猛虎出笼".to_string(), "skill_xionghuquan".to_string(), 10)
                .with_cost(15, 0)
                .with_attack(1.3)
                .with_special_damage(0, 10, 0)
                .with_cooldown(2)
                .with_description("武堂入门招式，如猛虎出笼。".to_string()),
        );

        self.performs.insert(
            "perform_xionghuquan_2".to_string(),
            Perform::new("perform_xionghuquan_2".to_string(), "虎啸山林".to_string(), "skill_xionghuquan".to_string(), 30)
                .with_cost(25, 5)
                .with_attack(1.6)
                .with_special_damage(10, 20, 5)
                .with_cooldown(3)
                .with_description("武堂进阶招式，虎啸山林震八方。".to_string()),
        );

        // 武当 - 太极剑招式
        self.performs.insert(
            "perform_taiji_1".to_string(),
            Perform::new("perform_taiji_1".to_string(), "太极起手".to_string(), "skill_taiji".to_string(), 10)
                .with_cost(10, 0)
                .with_attack(1.1)
                .with_damage_type(DamageType::Internal)
                .with_cooldown(2)
                .with_description("武当入门剑法，以柔克刚。".to_string()),
        );

        self.performs.insert(
            "perform_taiji_2".to_string(),
            Perform::new("perform_taiji_2".to_string(), "太极圆转".to_string(), "skill_taiji".to_string(), 30)
                .with_cost(20, 5)
                .with_attack(1.4)
                .with_damage_type(DamageType::Internal)
                .with_special_damage(0, 15, 10)
                .with_cooldown(3)
                .with_description("武当进阶剑法，圆转如意。".to_string()),
        );

        // 少林 - 罗汉拳招式
        self.performs.insert(
            "perform_luohanquan_1".to_string(),
            Perform::new("perform_luohanquan_1".to_string(), "罗汉拜佛".to_string(), "skill_luohanquan".to_string(), 10)
                .with_cost(15, 0)
                .with_attack(1.2)
                .with_bonus_damage(5)
                .with_cooldown(2)
                .with_description("少林入门拳法，刚猛有力。".to_string()),
        );

        self.performs.insert(
            "perform_luohanquan_2".to_string(),
            Perform::new("perform_luohanquan_2".to_string(), "罗汉降龙".to_string(), "skill_luohanquan".to_string(), 30)
                .with_cost(25, 5)
                .with_attack(1.5)
                .with_bonus_damage(15)
                .with_special_damage(10, 0, 0)
                .with_cooldown(3)
                .with_description("少林进阶拳法，势不可挡。".to_string()),
        );

        // 华山 - 独孤九剑招式
        self.performs.insert(
            "perform_dugu_1".to_string(),
            Perform::new("perform_dugu_1".to_string(), "破剑式".to_string(), "skill_dugujiujian".to_string(), 20)
                .with_cost(30, 0)
                .with_attack(1.5)
                .with_damage_type(DamageType::Internal)
                .with_special_damage(0, 20, 0)
                .with_cooldown(3)
                .with_description("华山剑法，专破诸般兵刃。".to_string()),
        );
    }

    /// 获取招式
    pub fn get_perform(&self, perform_id: &str) -> Option<&Perform> {
        self.performs.get(perform_id)
    }

    /// 获取技能的所有招式
    pub fn get_skill_performs(&self, skill_id: &str) -> Vec<&Perform> {
        self.performs
            .values()
            .filter(|p| p.skill_id == skill_id)
            .collect()
    }

    /// 获取玩家可用的招式
    pub fn get_available_performs(
        &self,
        skill_id: &str,
        skill_level: u32,
    ) -> Vec<&Perform> {
        self.performs
            .values()
            .filter(|p| {
                p.skill_id == skill_id && p.required_level <= skill_level
            })
            .collect()
    }
}

impl Default for PerformManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局招式管理器
pub static PERFORMD: once_cell::sync::Lazy<std::sync::Mutex<PerformManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(PerformManager::new()));

/// 获取招式管理器
pub fn get_performd() -> &'static std::sync::Mutex<PerformManager> {
    &PERFORMD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perform_creation() {
        let perform = Perform::new(
            "test_perform".to_string(),
            "测试招式".to_string(),
            "test_skill".to_string(),
            10,
        );

        assert_eq!(perform.name_cn, "测试招式");
        assert_eq!(perform.required_level, 10);
        assert_eq!(perform.qi_cost, 10);
    }

    #[test]
    fn test_perform_damage_calculation() {
        let perform = Perform::new(
            "test".to_string(),
            "测试".to_string(),
            "test_skill".to_string(),
            1,
        )
        .with_attack(1.5)
        .with_bonus_damage(10);

        let result = perform.calculate_damage(100, 30);
        // (100 * 1.5) - 30 + 10 = 150 - 30 + 10 = 130
        assert_eq!(result.damage, 130);
    }
}
