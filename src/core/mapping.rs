// core/mapping.rs - Mapping 类型实现
// 对应 Pike 的 mapping 类型

use crate::core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mapping 类型 (对应 Pike 的 mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    data: HashMap<String, Value>,
}

impl Mapping {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    // 对应 Pike 的 m[key] = value
    pub fn insert(&mut self, key: &str, value: Value) {
        self.data.insert(key.to_string(), value);
    }

    // 对应 Pike 的 m[key]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    // 对应 Pike 的 m_delete(mapping, key)
    pub fn delete(&mut self, key: &str) -> Option<Value> {
        self.data.remove(key)
    }

    // 对应 Pike 的 sizeof(mapping)
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // 对应 Pike 的 indices(mapping)
    pub fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    // 对应 Pike 的 values(mapping)
    pub fn values(&self) -> Vec<Value> {
        self.data.values().cloned().collect()
    }

    // 对应 Pike 的 mkmapping(keys, values)
    pub fn from_pairs(pairs: Vec<(String, Value)>) -> Self {
        let mut mapping = Self::with_capacity(pairs.len());
        for (key, value) in pairs {
            mapping.data.insert(key, value);
        }
        mapping
    }
}

impl Default for Mapping {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, Value>> for Mapping {
    fn from(data: HashMap<String, Value>) -> Self {
        Self { data }
    }
}

// 对应 Pike 的 mkmapping()
pub fn mkmapping(pairs: Vec<(String, Value)>) -> Mapping {
    Mapping::from_pairs(pairs)
}

// 对应 Pike 的 m_delete()
pub fn m_delete(mapping: &mut Mapping, key: &str) -> Option<Value> {
    mapping.delete(key)
}
