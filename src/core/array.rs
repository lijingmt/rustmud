// core/array.rs - Array 类型实现
// 对应 Pike 的 array 类型

use crate::core::Value;
use serde::{Deserialize, Serialize};

/// Array 类型 (对应 Pike 的 array)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Array {
    data: Vec<Value>,
}

impl Array {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    // 对应 Pike 的 ({value1, value2, ...})
    pub fn from_values(values: Vec<Value>) -> Self {
        Self { data: values }
    }

    // 对应 Pike 的 arr[index]
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.data.get(index)
    }

    // 对应 Pike 的 arr[index] = value
    pub fn set(&mut self, index: usize, value: Value) {
        if index < self.data.len() {
            self.data[index] = value;
        }
    }

    // 对应 Pike 的 sizeof(arr)
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // 对应 Pike 的 arr + arr2
    pub fn append(&mut self, other: &Array) {
        self.data.extend(other.data.iter().cloned());
    }

    // 对应 Pike 的 a_delete(arr, index)
    pub fn remove(&mut self, index: usize) -> Option<Value> {
        if index < self.data.len() {
            Some(self.data.remove(index))
        } else {
            None
        }
    }

    // 对应 Pike 的 a_insert(arr, index, value)
    pub fn insert(&mut self, index: usize, value: Value) {
        if index <= self.data.len() {
            self.data.insert(index, value);
        }
    }

    // 对应 Pike 的 arr[..n]
    pub fn slice(&self, start: usize, end: Option<usize>) -> Self {
        let end = end.unwrap_or(self.data.len());
        Self {
            data: self.data[start..end.min(self.data.len())].to_vec(),
        }
    }
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Value>> for Array {
    fn from(data: Vec<Value>) -> Self {
        Self { data }
    }
}

// 对应 Pike 的 a_delete()
pub fn a_delete(arr: &mut Array, index: usize) -> Option<Value> {
    arr.remove(index)
}

// 对应 Pike 的 a_insert()
pub fn a_insert(arr: &mut Array, index: usize, value: Value) {
    arr.insert(index, value)
}
