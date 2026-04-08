// core/value.rs - Value 类型实现
// 对应 Pike 的 mixed 类型

use serde::{Deserialize, Serialize};
use crate::core::ObjectId;

/// 任意值类型 (对应 Pike 的 mixed 类型)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Void,
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Mapping(std::collections::HashMap<String, Value>),
    Object(ObjectId),  // ObjectId
    Bool(bool),
    Null,
    Function(String),  // 函数名
}

impl Value {
    pub fn is_void(&self) -> bool {
        matches!(self, Value::Void)
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    // 对应 Pike 的 sizeof()
    pub fn sizeof(&self) -> usize {
        match self {
            Value::Void | Value::Null => 0,
            Value::Int(_) | Value::Float(_) | Value::Bool(_) => 1,
            Value::String(s) => s.len(),
            Value::Array(a) => a.len(),
            Value::Mapping(m) => m.len(),
            Value::Object(_) => 1,
            Value::Function(_) => 1,
        }
    }

    // 对应 Pike 的 sprintf() 格式化
    pub fn format(&self, fmt: &str) -> String {
        match self {
            Value::Int(i) => fmt.replace("%s", &i.to_string()),
            Value::String(s) => fmt.replace("%s", s),
            Value::Float(f) => fmt.replace("%s", &f.to_string()),
            _ => fmt.to_string(),
        }
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

// 对应 Pike 的 mapping 类型
pub type Mapping = std::collections::HashMap<String, Value>;

// 对应 Pike 的 array 类型
pub type Array = Vec<Value>;
