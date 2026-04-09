// gamenv/core/mod.rs - 核心抽象层
// 提供游戏系统的通用抽象接口

pub mod registry;
pub mod entity;
pub mod command;
pub mod parser;

pub use registry::*;
pub use entity::*;
pub use command::*;
pub use parser::*;

// 重新导出常用类型
pub use registry::{HashMapRegistry, TypedRegistry, Registry};
