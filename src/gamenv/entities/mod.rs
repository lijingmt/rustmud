// gamenv/entities/mod.rs - 实体模块
// 对应 txpike9/wapmud2/single/ 中的单例对象
// 使用trait组合来模拟LPC的多继承

pub mod character;
pub mod player;
pub mod npc;
pub mod room;

pub use character::*;
pub use player::*;
pub use npc::*;
pub use room::*;
