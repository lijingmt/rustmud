// gamenv/traits/mod.rs - Trait特性系统
// 对应 txpike9/wapmud2/inherit/feature/ 目录
// 使用Rust的trait系统模拟LPC的多继承特性组合

pub mod composition;
pub mod fight;
pub mod inventory;
pub mod skills;
pub mod equip;
pub mod movable;
pub mod talkable;
pub mod entity;

pub use composition::*;
pub use fight::*;
pub use inventory::*;
pub use skills::*;
pub use equip::*;
pub use movable::*;
pub use talkable::*;
pub use entity::*;
