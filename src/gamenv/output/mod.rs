// gamenv/output/mod.rs - 输出格式化模块
// 对应 txpike9 的各种输出格式化功能
//
// 提供 MUD 文本格式和 JSON 格式的输出转换

pub mod mud;
pub mod json;

pub use mud::*;
pub use json::*;
