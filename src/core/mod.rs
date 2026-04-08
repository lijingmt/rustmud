// core/mod.rs - 核心类型系统
// 对应 txpike9 的 Pike 核心类型

use serde::{Serialize, Deserialize};

pub mod object;
pub mod mapping;
pub mod array;
pub mod error;
pub mod value;
pub mod program;

pub use object::*;
pub use mapping::*;
pub use array::*;
pub use error::*;
pub use value::*;
pub use program::*;

use std::sync::Arc;
use tokio::sync::RwLock;

// 全局对象 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u64);

impl ObjectId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

// 对象引用类型 (对应 Pike 的 object 类型)
pub type GObject = Arc<RwLock<ObjectInner>>;

// 对应 Pike 的 backtrace
#[derive(Debug, Clone)]
pub struct Backtrace {
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub file: String,
    pub line: usize,
    pub function: String,
}

