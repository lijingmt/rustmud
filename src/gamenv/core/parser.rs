// gamenv/core/parser.rs - 解析器抽象
// 统一的输出解析和格式化接口

use serde::{Deserialize, Serialize};
use super::entity::Entity;

/// 输出段落 - 所有输出的基础单位
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub r#type: SegmentType,
    pub text: Option<String>,
    pub label: Option<String>,
    pub cmd: Option<String>,
    pub class: Option<String>,
    pub parts: Option<Vec<TextPart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
}

/// 段落类型
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    Text,
    Button,
    Link,
    Input,
    Image,
}

/// 文本部分（用于样式）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextPart {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
}

/// 内联样式
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
}

/// MUD输出行
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputLine {
    pub r#type: LineType,
    pub segments: Vec<Segment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineType {
    Line,
    Empty,
    Separator,
}

/// 解析器trait - 所有解析器实现此接口
pub trait Parser: Send + Sync {
    /// 解析文本为输出行
    fn parse(&self, input: &str) -> Vec<OutputLine>;

    /// 解析单个段落
    fn parse_segment(&self, text: &str) -> Segment;

    /// 检查是否支持此格式
    fn supports(&self, format: &str) -> bool;
}

/// 格式化器trait - 将游戏数据转换为输出
pub trait Formatter: Send + Sync {
    /// 格式化实体
    fn format_entity(&self, entity: &dyn Entity) -> OutputLine;

    /// 格式化房间
    fn format_room(&self, room: &dyn Entity) -> Vec<OutputLine>;

    /// 格式化战斗信息
    fn format_combat(&self, combat: &CombatInfo) -> Vec<OutputLine>;

    /// 格式化错误
    fn format_error(&self, error: &str) -> OutputLine;
}

/// 战斗信息
#[derive(Clone, Debug)]
pub struct CombatInfo {
    pub attacker: String,
    pub defender: String,
    pub attacker_hp: i32,
    pub defender_hp: i32,
    pub round: i32,
    pub log: Vec<String>,
}

/// 输出构建器 - 流式构建输出
pub struct OutputBuilder {
    lines: Vec<OutputLine>,
    current_segments: Vec<Segment>,
}

impl OutputBuilder {
    pub fn new() -> Self {
        Self {
            lines: vec![],
            current_segments: vec![],
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        self.current_segments.push(Segment {
            r#type: SegmentType::Text,
            text: Some(text.to_string()),
            label: None,
            cmd: None,
            class: None,
            parts: None,
            url: None,
            style: None,
        });
        self
    }

    pub fn colored_text(mut self, text: &str, color: &str) -> Self {
        self.current_segments.push(Segment {
            r#type: SegmentType::Text,
            text: Some(text.to_string()),
            label: None,
            cmd: None,
            class: None,
            parts: Some(vec![TextPart {
                text: text.to_string(),
                color: Some(color.to_string()),
                bold: None,
                underline: None,
                italic: None,
            }]),
            url: None,
            style: None,
        });
        self
    }

    pub fn button(mut self, label: &str, cmd: &str, style: &str) -> Self {
        self.current_segments.push(Segment {
            r#type: SegmentType::Button,
            text: None,
            label: Some(label.to_string()),
            cmd: Some(cmd.to_string()),
            class: Some(style.to_string()),
            parts: None,
            url: None,
            style: None,
        });
        self
    }

    pub fn newline(mut self) -> Self {
        if !self.current_segments.is_empty() {
            self.lines.push(OutputLine {
                r#type: LineType::Line,
                segments: std::mem::take(&mut self.current_segments),
            });
        }
        self
    }

    pub fn empty_line(mut self) -> Self {
        self.lines.push(OutputLine {
            r#type: LineType::Empty,
            segments: vec![],
        });
        self
    }

    pub fn separator(mut self) -> Self {
        self.lines.push(OutputLine {
            r#type: LineType::Separator,
            segments: vec![],
        });
        self
    }

    pub fn build(self) -> Vec<OutputLine> {
        let mut result = self.lines;
        if !self.current_segments.is_empty() {
            result.push(OutputLine {
                r#type: LineType::Line,
                segments: self.current_segments,
            });
        }
        result
    }
}

impl Default for OutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// WAPMUD格式解析器 - 解析 [label:command] 格式
pub struct WapmudParser;

impl Parser for WapmudParser {
    fn parse(&self, input: &str) -> Vec<OutputLine> {
        let mut lines = vec![];

        for line in input.lines() {
            let segments = self.parse_line(line);
            lines.push(OutputLine {
                r#type: if segments.is_empty() {
                    LineType::Empty
                } else {
                    LineType::Line
                },
                segments,
            });
        }

        lines
    }

    fn parse_segment(&self, text: &str) -> Segment {
        // 检查是否是按钮格式 [label:command]
        if let Some(start) = text.find('[') {
            if let Some(end) = text[start..].find(']') {
                let content = &text[start + 1..start + end];
                if let Some(colon) = content.find(':') {
                    let label = &content[..colon];
                    let command = &content[colon + 1..];
                    return Segment {
                        r#type: SegmentType::Button,
                        text: None,
                        label: Some(label.to_string()),
                        cmd: Some(command.to_string()),
                        class: Some("btn-primary".to_string()),
                        parts: None,
                        url: None,
                        style: None,
                    };
                }
            }
        }

        // 普通文本
        Segment {
            r#type: SegmentType::Text,
            text: Some(text.to_string()),
            label: None,
            cmd: None,
            class: None,
            parts: Some(vec![TextPart {
                text: text.to_string(),
                color: None,
                bold: None,
                underline: None,
                italic: None,
            }]),
            url: None,
            style: None,
        }
    }

    fn supports(&self, format: &str) -> bool {
        format == "wapmud" || format == "menu"
    }
}

impl WapmudParser {
    fn parse_line(&self, line: &str) -> Vec<Segment> {
        let mut segments = vec![];
        let mut remaining = line;

        while let Some(start) = remaining.find('[') {
            // 添加 [ 之前的文本
            if start > 0 {
                segments.push(self.parse_segment(&remaining[..start]));
            }

            // 查找 ]
            let after_bracket = &remaining[start + 1..];
            if let Some(end) = after_bracket.find(']') {
                let content = &after_bracket[..end];
                remaining = &after_bracket[end + 1..];

                // 解析按钮
                segments.push(self.parse_segment(&format!("[{}]", content)));
            } else {
                break;
            }
        }

        // 添加剩余文本
        if !remaining.is_empty() {
            segments.push(self.parse_segment(remaining));
        }

        segments
    }
}
