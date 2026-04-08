// pikenv/pike_save.rs - Pike save_object format parser
// 对应 txpike9 的 save_object 文件格式解析器

use crate::core::{MudError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 解析后的 Pike save_object 数据
#[derive(Debug, Clone)]
pub struct PikeSaveData {
    /// 程序路径 (如 #~/gamenv/clone/user.pike)
    pub program_path: Option<String>,
    /// 所有变量键值对
    pub variables: HashMap<String, PikeValue>,
    /// 数组数据 (对应 Pike 的 array)
    pub arrays: HashMap<String, Vec<PikeValue>>,
    /// 映射数据 (对应 Pike 的 mapping)
    pub mappings: HashMap<String, HashMap<String, PikeValue>>,
}

/// Pike 值类型
#[derive(Debug, Clone)]
pub enum PikeValue {
    String(String),
    Int(i64),
    Float(f64),
    Array(Vec<PikeValue>),
    Mapping(HashMap<String, PikeValue>),
    Null,
}

impl PikeValue {
    /// 获取字符串值
    pub fn as_str(&self) -> Option<&str> {
        match self {
            PikeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// 获取整数值
    pub fn as_int(&self) -> Option<i64> {
        match self {
            PikeValue::Int(i) => Some(*i),
            PikeValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// 获取浮点值
    pub fn as_float(&self) -> Option<f64> {
        match self {
            PikeValue::Float(f) => Some(*f),
            PikeValue::Int(i) => Some(*i as f64),
            PikeValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
}

/// 解析 Pike save_object 格式文件
pub fn parse_pike_save_file(path: &str) -> Result<PikeSaveData> {
    let content = fs::read_to_string(path)
        .map_err(|e| MudError::ObjectNotFound(format!("{}: {}", path, e)))?;

    parse_pike_save_content(&content)
}

/// 解析 Pike save_object 内容
pub fn parse_pike_save_content(content: &str) -> Result<PikeSaveData> {
    let mut data = PikeSaveData {
        program_path: None,
        variables: HashMap::new(),
        arrays: HashMap::new(),
        mappings: HashMap::new(),
    };

    let mut lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // 跳过空行
        if line.is_empty() {
            i += 1;
            continue;
        }

        // 解析程序路径 (#~/gamenv/clone/user.pike)
        if line.starts_with("#~/") {
            data.program_path = Some(line[2..].to_string());
            i += 1;
            continue;
        }

        // 解析键值对 (key value 或 key "value")
        if let Some(pos) = line.find(' ') {
            let key = line[..pos].trim().to_string();
            let value_str = line[pos + 1..].trim();

            // 处理带引号的字符串
            if value_str.starts_with('"') {
                let value = parse_quoted_string(value_str);
                data.variables.insert(key, PikeValue::String(value));
            }
            // 处理数组 ([...])
            else if value_str.starts_with('[') {
                let (array, next_i) = parse_array(value_str, &lines, i)?;
                data.arrays.insert(key.clone(), array.clone());
                i = next_i;
                // 同时存储到 variables
                data.variables.insert(key.clone(), PikeValue::Array(array));
            }
            // 处理映射 ({...})
            else if value_str.starts_with('{') {
                let (mapping, next_i) = parse_mapping(value_str, &lines, i)?;
                data.mappings.insert(key.clone(), mapping.clone());
                i = next_i;
                // 同时存储到 variables
                data.variables.insert(key.clone(), PikeValue::Mapping(mapping));
            }
            // 处理整数
            else if let Ok(int_val) = value_str.parse::<i64>() {
                data.variables.insert(key, PikeValue::Int(int_val));
            }
            // 处理描述等特殊字段
            else {
                data.variables.insert(key, PikeValue::String(value_str.to_string()));
            }
        }

        i += 1;
    }

    Ok(data)
}

/// 解析带引号的字符串
fn parse_quoted_string(s: &str) -> String {
    let s = s.trim_matches('"');
    s.to_string()
}

/// 解析数组
fn parse_array(s: &str, lines: &[&str], start_line: usize) -> Result<(Vec<PikeValue>, usize)> {
    let mut result = Vec::new();
    let mut content = s.to_string();
    let mut i = start_line;

    // 收集多行数组内容
    while !content.ends_with(']') {
        i += 1;
        if i >= lines.len() {
            break;
        }
        content.push_str(lines[i].trim());
    }

    // 简单解析：去除 [ ] 后分割
    let inner = content.trim_start_matches('[').trim_end_matches(']');
    if inner.is_empty() {
        return Ok((result, i));
    }

    // TODO: 实现完整的数组解析（支持嵌套）
    // 简化版本：按逗号分割
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.starts_with('"') {
            result.push(PikeValue::String(parse_quoted_string(part)));
        } else if let Ok(int_val) = part.parse::<i64>() {
            result.push(PikeValue::Int(int_val));
        } else {
            result.push(PikeValue::String(part.to_string()));
        }
    }

    Ok((result, i))
}

/// 解析映射
fn parse_mapping(s: &str, lines: &[&str], start_line: usize) -> Result<(HashMap<String, PikeValue>, usize)> {
    let mut result = HashMap::new();
    let mut content = s.to_string();
    let mut i = start_line;

    // 收集多行映射内容
    while !content.ends_with('}') {
        i += 1;
        if i >= lines.len() {
            break;
        }
        content.push_str(lines[i].trim());
    }

    // 简单解析：去除 { } 后分割
    let inner = content.trim_start_matches('{').trim_end_matches('}');
    if inner.is_empty() {
        return Ok((result, i));
    }

    // TODO: 实现完整的映射解析（支持嵌套）
    // 简化版本：按逗号分割键值对
    for pair in inner.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        if let Some(colon_pos) = pair.find(':') {
            let key = pair[..colon_pos].trim().trim_matches('"').to_string();
            let value_str = pair[colon_pos + 1..].trim();

            let value = if value_str.starts_with('"') {
                PikeValue::String(parse_quoted_string(value_str))
            } else if let Ok(int_val) = value_str.parse::<i64>() {
                PikeValue::Int(int_val)
            } else {
                PikeValue::String(value_str.to_string())
            };

            result.insert(key, value);
        }
    }

    Ok((result, i))
}

/// 获取用户文件路径 (txpike9 格式)
/// userid 映射到 gamenv/u/XX/XXXXXX.o
pub fn get_user_save_path(root: &str, userid: &str) -> String {
    let userid_len = userid.len();
    let dir = if userid_len >= 2 {
        &userid[0..2]
    } else {
        "00"
    };
    format!("{}/gamenv/u/{}/{}.o", root, dir, userid)
}

/// 检查用户文件是否存在
pub fn user_file_exists(root: &str, userid: &str) -> bool {
    let path = get_user_save_path(root, userid);
    Path::new(&path).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        let content = r#"
#~/gamenv/clone/user.pike
name "testuser"
level 10
"#;
        let data = parse_pike_save_content(content).unwrap();
        assert_eq!(data.variables.get("name").unwrap().as_str(), Some("testuser"));
        assert_eq!(data.variables.get("level").unwrap().as_int(), Some(10));
    }

    #[test]
    fn test_get_user_save_path() {
        let path = get_user_save_path("/usr/local/games", "tx0100");
        assert_eq!(path, "/usr/local/games/gamenv/u/00/tx0100.o");
    }
}
