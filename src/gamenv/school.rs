// gamenv/school.rs - 门派系统模块
// 重新导出门派守护进程

// 重新导出门派守护进程的内容
pub use crate::gamenv::single::schoold::{School, SchoolDaemon, get_schoold};

// 为了兼容性，重新导出技能相关的类型
pub use crate::gamenv::single::skilld::{Skill, PlayerSkill};

impl Skill {
    /// 计算升级所需经验
    pub fn exp_needed_for_level(level: u32) -> u64 {
        level as u64 * 100
    }
}
