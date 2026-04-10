// gamenv/quest.rs - 任务系统模块入口
// 对应 txpike9/gamenv/single/daemons/questd.pike 和 gamenv/inherit/questnpc.pike

pub mod types;
pub mod daemon;

// 重新导出常用类型
pub use types::{
    QuestType, RewardType, Currency, Quest, PlayerQuestData,
    QuestTemplate, QuestDaemonData, QuestStatus,
};
pub use daemon::{QuestDaemon, QUESTD};
