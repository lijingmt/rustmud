// gamenv/http_api/mud_output.rs - MUD 输出解析和格式化
// 将后端输出转换为 Vue 前端需要的 JSON 格式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MUD 输出行（Vue 前端格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MudLine {
    pub r#type: String,  // "line" | "empty"
    pub segments: Vec<MudSegment>,
}

/// MUD 输出段落
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MudSegment {
    pub r#type: SegmentType,
    pub text: Option<String>,
    pub label: Option<String>,
    pub cmd: Option<String>,
    pub class: Option<String>,
    pub src: Option<String>,
    pub alt: Option<String>,
    pub name: Option<String>,
    pub default: Option<String>,
    pub is_password: Option<bool>,
    pub cmd_prefix: Option<String>,
    pub width: Option<String>,
    pub input_type: Option<String>,
    pub placeholder: Option<String>,
    pub url: Option<String>,
    pub command: Option<String>,
    pub parts: Option<Vec<TextPart>>,
}

/// 段落类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    Text,
    Button,
    Input,
    #[serde(rename = "form-input")]
    FormInput,
    #[serde(rename = "submit-button")]
    SubmitButton,
    #[serde(rename = "cmd-input")]
    CmdInput,
    #[serde(rename = "url-link")]
    UrlLink,
    Image,
}

/// 文本部分（用于颜色等样式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPart {
    pub text: String,
    pub color: Option<String>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub link: Option<String>,
}

/// MUD 输出解析器
pub struct MudOutputParser {
    /// 当前房间信息
    current_room: Option<RoomInfo>,
    /// 当前房间中的 NPC 列表
    room_npcs: Vec<NpcInfo>,
    /// 当前房间中的出口
    room_exits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcInfo {
    pub id: String,
    pub name: String,
    pub short: String,
}

impl MudOutputParser {
    pub fn new() -> Self {
        Self {
            current_room: None,
            room_npcs: vec![],
            room_exits: vec![],
        }
    }

    /// 解析 MUD 输出为 Vue 前端格式
    pub fn parse_output(&self, output: &str) -> Vec<MudLine> {
        let mut lines = vec![];

        for line in output.lines() {
            if line.trim().is_empty() {
                lines.push(MudLine {
                    r#type: "empty".to_string(),
                    segments: vec![],
                });
            } else {
                lines.push(self.parse_line(line));
            }
        }

        lines
    }

    /// 解析单行输出
    fn parse_line(&self, line: &str) -> MudLine {
        let mut segments = vec![];

        // 检测可点击的 NPC
        for npc in &self.room_npcs {
            if line.contains(&npc.name) || line.contains(&npc.short) {
                // 找到 NPC，创建可点击按钮
                let parts: Vec<&str> = line.splitn(2, &npc.name).collect();
                if parts.len() == 2 {
                    // 前面的文本
                    if !parts[0].is_empty() {
                        segments.push(MudSegment {
                            r#type: SegmentType::Text,
                            text: Some(parts[0].to_string()),
                            parts: Some(vec![TextPart {
                                text: parts[0].to_string(),
                                color: None,
                                bold: None,
                                underline: None,
                                link: None,
                            }]),
                            ..Default::default()
                        });
                    }
                    // NPC 按钮
                    segments.push(MudSegment {
                        r#type: SegmentType::Button,
                        label: Some(npc.name.clone()),
                        cmd: Some(format!("look {}", npc.id)),
                        class: Some("npc-link".to_string()),
                        ..Default::default()
                    });
                    // 后面的文本
                    if !parts[1].is_empty() {
                        segments.push(MudSegment {
                            r#type: SegmentType::Text,
                            text: Some(parts[1].to_string()),
                            parts: Some(vec![TextPart {
                                text: parts[1].to_string(),
                                color: None,
                                bold: None,
                                underline: None,
                                link: None,
                            }]),
                            ..Default::default()
                        });
                    }
                    return MudLine {
                        r#type: "line".to_string(),
                        segments,
                    };
                }
            }
        }

        // 检测出口
        for exit in &self.room_exits {
            if line.contains(&format!("{}:", exit)) || line.contains(&format!("（{}）", exit)) {
                segments.push(MudSegment {
                    r#type: SegmentType::Button,
                    label: Some(exit.clone()),
                    cmd: Some(format!("go {}", exit)),
                    class: Some("exit-link".to_string()),
                    ..Default::default()
                });
                return MudLine {
                    r#type: "line".to_string(),
                    segments,
                };
            }
        }

        // 默认：纯文本
        segments.push(MudSegment {
            r#type: SegmentType::Text,
            text: Some(line.to_string()),
            parts: Some(vec![TextPart {
                text: line.to_string(),
                color: None,
                bold: None,
                underline: None,
                link: None,
            }]),
            ..Default::default()
        });

        MudLine {
            r#type: "line".to_string(),
            segments,
        }
    }

    /// 解析房间信息（从 look 命令的输出）
    pub fn parse_room_info(&mut self, output: &str) {
        // 简单解析：查找房间名称和描述
        let lines: Vec<&str> = output.lines().collect();

        if let Some(first_line) = lines.first() {
            if first_line.contains("│") {
                // 假设是房间名称行
                let name = first_line.replace("│", "").trim().to_string();
                self.current_room = Some(RoomInfo {
                    id: "unknown".to_string(),
                    name: name.clone(),
                    short: name.clone(),
                    long: output.to_string(),
                });
            }
        }

        // 解析出口
        for line in &lines {
            if line.contains("出口:") || line.contains("Exits:") {
                // 提取出口方向
                let exits = ["north", "south", "east", "west", "up", "down", "西北", "西南", "东北", "东南"];
                for exit in &exits {
                    if line.contains(exit) || line.contains(&exit.chars().next().unwrap_or(' ').to_string()) {
                        self.room_exits.push(exit.to_string());
                    }
                }
            }

            // 解析 NPC
            if line.contains("这里") && (line.contains("有") || line.contains("站着")) {
                // 简单的 NPC 检测
                let npcs = self.extract_npcs_from_line(line);
                self.room_npcs.extend(npcs);
            }
        }
    }

    /// 从行中提取 NPC
    fn extract_npcs_from_line(&self, line: &str) -> Vec<NpcInfo> {
        // 简单实现：假设 NPC 格式为 "一个[NPC名称]"
        let mut npcs = vec![];

        // 常见 NPC 关键词
        let npc_keywords = ["老人", "商人", "铁匠", "守卫", "村民", "怪物", "敌人"];
        for keyword in &npc_keywords {
            if line.contains(keyword) {
                npcs.push(NpcInfo {
                    id: keyword.to_string(),
                    name: keyword.to_string(),
                    short: keyword.to_string(),
                });
            }
        }

        npcs
    }

    /// 更新房间信息（从后端数据）
    pub fn update_room(&mut self, room_data: &RoomData) {
        self.current_room = Some(RoomInfo {
            id: room_data.id.clone(),
            name: room_data.name.clone(),
            short: room_data.short.clone(),
            long: room_data.long.clone(),
        });
        self.room_npcs = room_data.npcs.clone();
        self.room_exits = room_data.exits.clone();
    }

    /// 生成房间 JSON 输出（用于 Vue 前端）
    pub fn generate_room_json(&self) -> Vec<MudLine> {
        let mut lines = vec![];

        if let Some(room) = &self.current_room {
            // 房间名称
            lines.push(MudLine {
                r#type: "line".to_string(),
                segments: vec![MudSegment {
                    r#type: SegmentType::Text,
                    text: Some(room.name.clone()),
                    parts: Some(vec![TextPart {
                        text: room.name.clone(),
                        color: Some("#00ff00".to_string()),
                        bold: Some(true),
                        underline: None,
                        link: None,
                    }]),
                    ..Default::default()
                }],
            });

            // 房间描述
            lines.push(MudLine {
                r#type: "empty".to_string(),
                segments: vec![],
            });

            for desc_line in room.long.lines() {
                lines.push(MudLine {
                    r#type: "line".to_string(),
                    segments: vec![MudSegment {
                        r#type: SegmentType::Text,
                        text: Some(desc_line.to_string()),
                        parts: Some(vec![TextPart {
                            text: desc_line.to_string(),
                            color: Some("#cccccc".to_string()),
                            bold: None,
                            underline: None,
                            link: None,
                        }]),
                        ..Default::default()
                    }],
                });
            }

            // 出口
            if !self.room_exits.is_empty() {
                lines.push(MudLine {
                    r#type: "empty".to_string(),
                    segments: vec![],
                });

                let exit_text = format!("出口: {}", self.room_exits.join(" "));
                lines.push(MudLine {
                    r#type: "line".to_string(),
                    segments: vec![MudSegment {
                        r#type: SegmentType::Text,
                        text: Some(exit_text.clone()),
                        parts: Some(vec![TextPart {
                            text: exit_text,
                            color: Some("#ffff00".to_string()),
                            bold: None,
                            underline: None,
                            link: None,
                        }]),
                        ..Default::default()
                    }],
                });

                // 可点击的出口按钮
                let mut exit_segments = vec![];
                for exit in &self.room_exits {
                    exit_segments.push(MudSegment {
                        r#type: SegmentType::Button,
                        label: Some(exit.clone()),
                        cmd: Some(format!("go {}", exit)),
                        class: Some("exit-btn".to_string()),
                        ..Default::default()
                    });
                }
                lines.push(MudLine {
                    r#type: "line".to_string(),
                    segments: exit_segments,
                });
            }

            // NPC
            if !self.room_npcs.is_empty() {
                lines.push(MudLine {
                    r#type: "empty".to_string(),
                    segments: vec![],
                });

                lines.push(MudLine {
                    r#type: "line".to_string(),
                    segments: vec![MudSegment {
                        r#type: SegmentType::Text,
                        text: Some("这里的人物:".to_string()),
                        parts: Some(vec![TextPart {
                            text: "这里的人物:".to_string(),
                            color: Some("#ff9900".to_string()),
                            bold: None,
                            underline: None,
                            link: None,
                        }]),
                        ..Default::default()
                    }],
                });

                let mut npc_segments = vec![];
                for npc in &self.room_npcs {
                    npc_segments.push(MudSegment {
                        r#type: SegmentType::Button,
                        label: Some(format!("★ {}", npc.short)),
                        cmd: Some(format!("look {}", npc.id)),
                        class: Some("npc-btn".to_string()),
                        ..Default::default()
                    });
                }
                lines.push(MudLine {
                    r#type: "line".to_string(),
                    segments: npc_segments,
                });
            }
        }

        lines
    }
}

/// 房间数据（从后端获取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomData {
    pub id: String,
    pub name: String,
    pub short: String,
    pub long: String,
    pub npcs: Vec<NpcInfo>,
    pub exits: Vec<String>,
}

impl Default for MudSegment {
    fn default() -> Self {
        Self {
            r#type: SegmentType::Text,
            text: None,
            label: None,
            cmd: None,
            class: None,
            src: None,
            alt: None,
            name: None,
            default: None,
            is_password: None,
            cmd_prefix: None,
            width: None,
            input_type: None,
            placeholder: None,
            url: None,
            command: None,
            parts: None,
        }
    }
}

impl Default for NpcInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            short: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutputResponse {
    pub status: String,
    pub mud_lines: Vec<MudLine>,
    pub room_info: Option<RoomData>,
    pub player_stats: Option<PlayerStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub hp: i32,
    pub hp_max: i32,
    pub spirit: i32,
    pub spirit_max: i32,
    pub potential: i32,
    pub potential_max: i32,
    pub neili: i32,
    pub neili_max: i32,
    pub exp: i64,
    pub level: i32,
    pub name_cn: Option<String>,
    pub autofight: bool,
}

/// 将纯文本输出转换为 JSON 格式
pub fn text_to_json_output(output: String) -> JsonOutputResponse {
    let parser = MudOutputParser::new();
    let mud_lines = parser.parse_output(&output);

    JsonOutputResponse {
        status: "success".to_string(),
        mud_lines,
        room_info: None,
        player_stats: None,
    }
}
