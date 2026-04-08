// gamenv/npc/trainer.rs - 训练师NPC
// 对应 txpike9/gamenv/clone/npc/ 中的训练师

use crate::core::*;
use crate::gamenv::npc::npc::{Npc, NpcType};
use serde::{Deserialize, Serialize};

/// 可学技能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachableSkill {
    /// 技能ID
    pub skill_id: String,
    /// 技能名称
    pub skill_name: String,
    /// 要求等级
    pub required_level: u32,
    /// 要求修为
    pub required_exp: u64,
    /// 学习费用
    pub cost: u64,
    /// 要求前置技能
    pub prerequisite: Option<String>,
}

/// 训练师NPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trainer {
    /// 基础NPC数据
    #[serde(flatten)]
    pub npc: Npc,
    /// 门派/类别
    pub school: String,
    /// 可学技能列表
    pub teachable_skills: Vec<TeachableSkill>,
}

impl Trainer {
    /// 创建新训练师
    pub fn new(name: String, name_cn: String, school: String) -> Self {
        let mut npc = Npc::new(name.clone(), name_cn.clone(), NpcType::Trainer, 1);
        npc.base.desc = format!("{}的训练师，可以教授各种技能。", school);

        Self {
            npc,
            school,
            teachable_skills: Vec::new(),
        }
    }

    /// 添加可学技能
    pub fn add_skill(&mut self, skill: TeachableSkill) {
        self.teachable_skills.push(skill);
    }

    /// 获取可学技能列表
    pub fn get_learnable_skills(&self, player_level: u32, player_exp: u64) -> Vec<&TeachableSkill> {
        self.teachable_skills.iter()
            .filter(|skill| {
                skill.required_level <= player_level && skill.required_exp <= player_exp
            })
            .collect()
    }

    /// 检查是否可以学习技能
    pub fn can_learn(&self, skill_id: &str, player_level: u32, player_exp: u64,
                     learned_skills: &[String]) -> Result<()> {
        let skill = self.teachable_skills.iter()
            .find(|s| s.skill_id == skill_id)
            .ok_or_else(|| MudError::NotFound("技能不存在".to_string()))?;

        if skill.required_level > player_level {
            return Err(MudError::InvalidOperation(
                format!("等级不足，需要{}级", skill.required_level)
            ));
        }

        if skill.required_exp > player_exp {
            return Err(MudError::InvalidOperation(
                format!("修为不足，需要{}", skill.chinese_num(skill.required_exp))
            ));
        }

        if let Some(ref prereq) = skill.prerequisite {
            if !learned_skills.contains(prereq) {
                return Err(MudError::InvalidOperation(
                    format!("需要先学习前置技能: {}", prereq)
                ));
            }
        }

        Ok(())
    }

    /// 渲染技能列表
    pub fn render_skill_list(&self, player_level: u32, player_exp: u64) -> String {
        let mut result = format!("=== {} 技能列表 ===\n", self.school);
        result.push_str(&format!("训练师: {}\n\n", self.npc.base.name_cn));

        for skill in &self.teachable_skills {
            let can_learn = skill.required_level <= player_level && skill.required_exp <= player_exp;
            let status = if can_learn { "§g可学§r" } else { "§c未解锁§r" };

            result.push_str(&format!("{}【{}】{} - {} 金 {}\n",
                status,
                skill.skill_name,
                skill.skill_id,
                skill.cost,
                if skill.required_level > player_level {
                    format!("(需{}级)", skill.required_level)
                } else if skill.prerequisite.is_some() {
                    format!("(需前置技能)",)
                } else {
                    String::new()
                }
            ));
        }

        result
    }
}

impl TeachableSkill {
    /// 转换数字为中文单位 (修为显示用)
    fn chinese_num(&self, n: u64) -> String {
        if n >= 1_0000_0000 {
            format!("{}亿", n / 1_0000_0000)
        } else if n >= 1_0000 {
            format!("{}万", n / 1_0000)
        } else {
            n.to_string()
        }
    }
}

/// 预设训练师列表
pub fn create_preset_trainers() -> Vec<Trainer> {
    vec![
        // 基础战斗训练师
        {
            let mut trainer = Trainer::new(
                "trainer_basic".to_string(),
                "武师".to_string(),
                "基础战斗".to_string(),
            );
            trainer.npc.base.desc = "一位经验丰富的武师，可以教给你基本的战斗技巧。".to_string();
            trainer.npc.base.add_dialogue("想学功夫吗？我可以教你几招！".to_string());

            trainer.add_skill(TeachableSkill {
                skill_id: "skill_basic_attack".to_string(),
                skill_name: "基础攻击".to_string(),
                required_level: 1,
                required_exp: 0,
                cost: 100,
                prerequisite: None,
            });

            trainer.add_skill(TeachableSkill {
                skill_id: "skill_power_strike".to_string(),
                skill_name: "强力攻击".to_string(),
                required_level: 5,
                required_exp: 1000,
                cost: 500,
                prerequisite: Some("skill_basic_attack".to_string()),
            });

            trainer.add_skill(TeachableSkill {
                skill_id: "skill_defense".to_string(),
                skill_name: "防御姿态".to_string(),
                required_level: 3,
                required_exp: 500,
                cost: 300,
                prerequisite: None,
            });

            trainer
        },

        // 内功训练师
        {
            let mut trainer = Trainer::new(
                "trainer_neigong".to_string(),
                "道长".to_string(),
                "内功心法".to_string(),
            );
            trainer.npc.base.desc = "一位修为高深的道长，精通各种内功心法。".to_string();

            trainer.add_skill(TeachableSkill {
                skill_id: "skill_qigong".to_string(),
                skill_name: "基础气功".to_string(),
                required_level: 10,
                required_exp: 10000,
                cost: 1000,
                prerequisite: None,
            });

            trainer.add_skill(TeachableSkill {
                skill_id: "skill_meditation".to_string(),
                skill_name: "打坐调息".to_string(),
                required_level: 15,
                required_exp: 50000,
                cost: 5000,
                prerequisite: Some("skill_qigong".to_string()),
            });

            trainer
        },
    ]
}
