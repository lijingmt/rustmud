// core/mod.rs - Core type system
// Corresponds to txpike9's Pike core types

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod object;
pub mod mapping;
pub mod array;
pub mod error;
pub mod value;
pub mod program;

// Define GObject type alias first (before re-exports)
pub type GObject = Arc<RwLock<ObjectInner>>;

// Re-export specific items to avoid conflicts
pub use object::{ObjectInner, ObjectManager, GObjectExt};
pub use mapping::Mapping;
pub use error::{MudError, Result};
pub use value::Value;
pub use program::{Program, ProgramManager};

// Global object ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u64);

impl ObjectId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

// Backtrace structure (corresponds to Pike's backtrace)
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

