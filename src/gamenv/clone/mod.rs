// gamenv/clone/mod.rs - 可克隆对象模块
// 对应 txpike9/gamenv/clone/ 目录
//
// clone/ 包含可克隆对象的模板，包括:
// - item/: 物品模板 (武器、防具、药品等)
// - npc/: NPC模板
// - user.pike: 用户对象模板

pub mod item;
pub mod npc;

pub use item::*;
pub use npc::*;
