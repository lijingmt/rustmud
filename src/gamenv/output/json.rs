// gamenv/output/json.rs - JSON格式化输出
// 对应 txpike9 HTTP API 的JSON响应格式

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

/// JSON输出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
    /// 输出文本 (可能包含颜色代码)
    pub text: String,
    /// 输出类型 (message, room, combat, system等)
    #[serde(rename = "type")]
    pub output_type: String,
    /// HTML格式的文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// 额外数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

impl JsonOutput {
    /// 创建新的JSON输出
    pub fn new(text: String, output_type: String) -> Self {
        Self {
            text,
            output_type,
            html: None,
            data: None,
        }
    }

    /// 创建消息类型输出
    pub fn message(text: String) -> Self {
        Self::new(text, "message".to_string())
    }

    /// 创建房间类型输出
    pub fn room(text: String) -> Self {
        Self::new(text, "room".to_string())
    }

    /// 创建战斗类型输出
    pub fn combat(text: String) -> Self {
        Self::new(text, "combat".to_string())
    }

    /// 创建系统类型输出
    pub fn system(text: String) -> Self {
        Self::new(text, "system".to_string())
    }

    /// 创建错误类型输出
    pub fn error(text: String) -> Self {
        Self::new(text, "error".to_string())
    }

    /// 设置HTML内容
    pub fn with_html(mut self, html: String) -> Self {
        self.html = Some(html);
        self
    }

    /// 设置数据
    pub fn with_data(mut self, data: JsonValue) -> Self {
        self.data = Some(data);
        self
    }

    /// 转换为JSON字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// JSON响应构建器
pub struct JsonBuilder {
    outputs: Vec<JsonOutput>,
}

impl JsonBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
        }
    }

    /// 添加输出
    pub fn add(mut self, output: JsonOutput) -> Self {
        self.outputs.push(output);
        self
    }

    /// 添加消息
    pub fn message(mut self, text: &str) -> Self {
        self.outputs.push(JsonOutput::message(text.to_string()));
        self
    }

    /// 添加系统消息
    pub fn system(mut self, text: &str) -> Self {
        self.outputs.push(JsonOutput::system(text.to_string()));
        self
    }

    /// 添加错误消息
    pub fn error(mut self, text: &str) -> Self {
        self.outputs.push(JsonOutput::error(text.to_string()));
        self
    }

    /// 添加房间输出
    pub fn room(mut self, text: &str) -> Self {
        self.outputs.push(JsonOutput::room(text.to_string()));
        self
    }

    /// 添加战斗输出
    pub fn combat(mut self, text: &str) -> Self {
        self.outputs.push(JsonOutput::combat(text.to_string()));
        self
    }

    /// 构建JSON数组
    pub fn build(self) -> JsonValue {
        json!(self.outputs)
    }

    /// 构建JSON字符串
    pub fn build_json(self) -> String {
        self.build().to_string()
    }
}

impl Default for JsonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP API响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
    /// 错误消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 输出内容 (用于MUD输出)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<JsonOutput>>,
}

impl ApiResponse {
    /// 创建成功响应
    pub fn success() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            output: None,
        }
    }

    /// 创建带数据的成功响应
    pub fn with_data(data: JsonValue) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            output: None,
        }
    }

    /// 创建带输出的成功响应
    pub fn with_output(output: Vec<JsonOutput>) -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            output: Some(output),
        }
    }

    /// 创建错误响应
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            output: None,
        }
    }

    /// 设置数据
    pub fn set_data(mut self, data: JsonValue) -> Self {
        self.data = Some(data);
        self
    }

    /// 设置输出
    pub fn set_output(mut self, output: Vec<JsonOutput>) -> Self {
        self.output = Some(output);
        self
    }

    /// 转换为JSON字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// 房间信息JSON结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    /// 房间ID
    pub id: String,
    /// 房间名称
    pub name: String,
    /// 房间描述
    pub description: String,
    /// 出口列表
    pub exits: Vec<String>,
    /// NPC列表
    pub npcs: Vec<NpcInfo>,
    /// 物品列表
    pub items: Vec<ItemInfo>,
    /// 玩家列表
    pub players: Vec<PlayerInfo>,
}

/// NPC信息JSON结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcInfo {
    pub id: String,
    pub name: String,
    pub name_cn: String,
    pub level: u32,
    pub hp_percent: i32,
}

/// 物品信息JSON结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    pub id: String,
    pub name: String,
    pub name_cn: String,
}

/// 玩家信息JSON结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
    pub name_cn: String,
    pub level: u32,
}

/// 战斗信息JSON结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatInfo {
    /// 战斗ID
    pub battle_id: String,
    /// 己方信息
    pub self_info: CombatantInfo,
    /// 敌方信息
    pub enemy_info: CombatantInfo,
    /// 当前回合
    pub current_round: u32,
    /// 战斗日志
    pub log: Vec<String>,
}

/// 战斗者信息JSON结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatantInfo {
    pub id: String,
    pub name: String,
    pub name_cn: String,
    pub hp: i32,
    pub hp_max: i32,
    pub hp_percent: i32,
    pub level: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_output() {
        let output = JsonOutput::message("Hello".to_string());
        assert_eq!(output.output_type, "message");
        assert_eq!(output.text, "Hello");
    }

    #[test]
    fn test_api_response() {
        let response = ApiResponse::with_data(json!({"key": "value"}));
        assert!(response.success);
        assert!(response.data.is_some());
    }

    #[test]
    fn test_json_builder() {
        let json = JsonBuilder::new()
            .message("Hello")
            .system("System message")
            .build();
        assert_eq!(json.as_array().unwrap().len(), 2);
    }
}
