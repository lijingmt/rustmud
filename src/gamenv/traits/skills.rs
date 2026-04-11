// gamenv/traits/skills.rs - 技能特性
// 对应 txpike9/wapmud2/inherit/feature/skills.pike

use std::collections::HashMap;

/// 技能特性 - 所有可使用技能的对象都应实现此trait
pub trait Skills {
    /// 获取技能列表 (技能ID -> 等级)
    fn skills(&self) -> &HashMap<String, u32>;

    /// 学习技能
    async fn learn_skill(&mut self, skill_id: &str) -> Result<(), String>;

    /// 使用技能
    async fn use_skill(&mut self, skill_id: &str) -> Result<String, String>;

    /// 获取技能等级
    fn get_skill_level(&self, skill_id: &str) -> u32 {
        self.skills().get(skill_id).copied().unwrap_or(0)
    }

    /// 是否拥有指定技能
    fn has_skill(&self, skill_id: &str) -> bool {
        self.skills().contains_key(skill_id)
    }

    /// 列出所有技能
    fn list_skills(&self) -> String {
        let skills = self.skills();
        if skills.is_empty() {
            return "你还不会任何技能。".to_string();
        }

        let mut output = "你掌握的技能：\n".to_string();
        for (skill_id, level) in skills {
            output.push_str(&format!("  {} (等级 {})\n", skill_id, level));
        }
        output
    }
}
