// gamenv/output/mud.rs - MUD文本格式化
// 对应 txpike9 的颜色代码和格式化输出

use serde_json::Value as JsonValue;

/// MUD颜色代码映射
/// 对应 txpike9 中的颜色系统
#[derive(Debug, Clone, Copy)]
pub enum MudColor {
    /// 黑色 - §N (实际上是不带颜色)
    Black,
    /// 红色 - §R
    Red,
    /// 绿色 - §G
    Green,
    /// 黄色 - §Y
    Yellow,
    /// 蓝色 - §B
    Blue,
    /// 品红 - §M
    Magenta,
    /// 青色 - §C
    Cyan,
    /// 白色 - §W
    White,
    /// 高亮黑色 - §b
    DarkGray,
    /// 高亮红色 - §r
    BrightRed,
    /// 高亮绿色 - §g
    BrightGreen,
    /// 高亮黄色 - §y
    BrightYellow,
    /// 高亮蓝色 - §u
    BrightBlue,
    /// 高亮品红 - §m
    BrightMagenta,
    /// 高亮青色 - §c
    BrightCyan,
    /// 高亮白色 - §w
    BrightWhite,
    /// 无颜色 (默认) - §N
    Normal,
}

impl MudColor {
    /// 获取颜色代码
    pub fn code(self) -> &'static str {
        match self {
            MudColor::Black => "§k",
            MudColor::Red => "§R",
            MudColor::Green => "§G",
            MudColor::Yellow => "§Y",
            MudColor::Blue => "§B",
            MudColor::Magenta => "§M",
            MudColor::Cyan => "§C",
            MudColor::White => "§W",
            MudColor::DarkGray => "§b",
            MudColor::BrightRed => "§r",
            MudColor::BrightGreen => "§g",
            MudColor::BrightYellow => "§y",
            MudColor::BrightBlue => "§u",
            MudColor::BrightMagenta => "§m",
            MudColor::BrightCyan => "§c",
            MudColor::BrightWhite => "§w",
            MudColor::Normal => "§N",
        }
    }

    /// 获取HTML颜色代码 (用于Web前端)
    pub fn html_color(self) -> &'static str {
        match self {
            MudColor::Black => "#000000",
            MudColor::Red => "#FF0000",
            MudColor::Green => "#00FF00",
            MudColor::Yellow => "#FFFF00",
            MudColor::Blue => "#0000FF",
            MudColor::Magenta => "#FF00FF",
            MudColor::Cyan => "#00FFFF",
            MudColor::White => "#FFFFFF",
            MudColor::DarkGray => "#808080",
            MudColor::BrightRed => "#FF8080",
            MudColor::BrightGreen => "#80FF80",
            MudColor::BrightYellow => "#FFFF80",
            MudColor::BrightBlue => "#8080FF",
            MudColor::BrightMagenta => "#FF80FF",
            MudColor::BrightCyan => "#80FFFF",
            MudColor::BrightWhite => "#FFFFFF",
            MudColor::Normal => "#FFFFFF",
        }
    }
}

/// MUD文本格式化器
pub struct MudFormatter;

impl MudFormatter {
    /// 给文本添加颜色
    pub fn color(text: &str, color: MudColor) -> String {
        format!("{}{}{}", color.code(), text, MudColor::Normal.code())
    }

    /// 红色文本 (常用于错误、危险)
    pub fn red(text: &str) -> String {
        Self::color(text, MudColor::Red)
    }

    /// 绿色文本 (常用于成功信息)
    pub fn green(text: &str) -> String {
        Self::color(text, MudColor::Green)
    }

    /// 黄色文本 (常用于警告)
    pub fn yellow(text: &str) -> String {
        Self::color(text, MudColor::Yellow)
    }

    /// 蓝色文本 (常用于信息)
    pub fn blue(text: &str) -> String {
        Self::color(text, MudColor::Blue)
    }

    /// 高亮文本
    pub fn highlight(text: &str) -> String {
        Self::color(text, MudColor::BrightYellow)
    }

    /// 系统消息
    pub fn system(text: &str) -> String {
        Self::color(text, MudColor::BrightCyan)
    }

    /// 重要信息
    pub fn important(text: &str) -> String {
        Self::color(text, MudColor::BrightRed)
    }

    /// 创建带超链接的文本 (MUD格式)
    /// 格式: [显示文本:命令]
    pub fn link(text: &str, command: &str) -> String {
        format!("[{}:{}]", text, command)
    }

    /// 创建按钮 (MUD格式)
    pub fn button(text: &str, command: &str) -> String {
        Self::link(text, command)
    }

    /// 格式化房间标题
    pub fn room_title(text: &str) -> String {
        Self::color(text, MudColor::BrightYellow)
    }

    /// 格式化NPC名称
    pub fn npc_name(text: &str) -> String {
        Self::color(text, MudColor::BrightGreen)
    }

    /// 格式化玩家名称
    pub fn player_name(text: &str) -> String {
        Self::color(text, MudColor::BrightCyan)
    }

    /// 格式化物品名称
    pub fn item_name(text: &str) -> String {
        Self::color(text, MudColor::BrightMagenta)
    }

    /// 格式化方向按钮
    pub fn direction_button(dir: &str, room: &str) -> String {
        Self::button(dir, room)
    }

    /// 格式化战斗信息
    pub fn combat_info(text: &str) -> String {
        Self::color(text, MudColor::BrightRed)
    }

    /// 格式化属性显示
    pub fn stat_label(label: &str) -> String {
        Self::color(label, MudColor::Cyan)
    }

    /// 换行
    pub fn newline() -> String {
        "\\n".to_string()
    }

    /// 分隔线 (简化版，不带 =====)
    pub fn separator() -> String {
        "\\n".to_string()
    }

    /// 将MUD颜色代码转换为HTML span标签
    pub fn to_html(text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        let mut current_color = MudColor::Normal;

        while let Some(c) = chars.next() {
            if c == '§' {
                if let Some(&color_char) = chars.peek() {
                    chars.next(); // 消费颜色字符
                    let new_color = match color_char {
                        'k' => MudColor::Black,
                        'R' => MudColor::Red,
                        'G' => MudColor::Green,
                        'Y' => MudColor::Yellow,
                        'B' => MudColor::Blue,
                        'M' => MudColor::Magenta,
                        'C' => MudColor::Cyan,
                        'W' => MudColor::White,
                        'b' => MudColor::DarkGray,
                        'r' => MudColor::BrightRed,
                        'g' => MudColor::BrightGreen,
                        'y' => MudColor::BrightYellow,
                        'u' => MudColor::BrightBlue,
                        'm' => MudColor::BrightMagenta,
                        'c' => MudColor::BrightCyan,
                        'w' => MudColor::BrightWhite,
                        'N' | _ => MudColor::Normal,
                    };

                    if new_color != MudColor::Normal {
                        if current_color != MudColor::Normal {
                            result.push_str("</span>");
                        }
                        result.push_str(&format!("<span style=\"color:{}\">", new_color.html_color()));
                        current_color = new_color;
                    } else {
                        if current_color != MudColor::Normal {
                            result.push_str("</span>");
                            current_color = MudColor::Normal;
                        }
                    }
                }
            } else {
                result.push(c);
            }
        }

        if current_color != MudColor::Normal {
            result.push_str("</span>");
        }

        result
    }

    /// 转换链接格式 [text:cmd] 为 HTML
    pub fn links_to_html(text: &str) -> String {
        // 简单的正则替换: [text:cmd] -> <a href="#" onclick="sendCmd('cmd')">text</a>
        let re = regex::Regex::new(r"\[([^\]:]+):([^\]]+)\]").unwrap();
        re.replace_all(text, "<a href=\"#\" onclick=\"sendCmd('$2')\">$1</a>").to_string()
    }
}

/// 格式化输出构建器
pub struct OutputBuilder {
    content: String,
}

impl OutputBuilder {
    /// 创建新的输出构建器
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    /// 添加文本
    pub fn text(mut self, text: &str) -> Self {
        self.content.push_str(text);
        self
    }

    /// 添加带颜色的文本
    pub fn colored(mut self, text: &str, color: MudColor) -> Self {
        self.content.push_str(&MudFormatter::color(text, color));
        self
    }

    /// 添加红色文本
    pub fn red(mut self, text: &str) -> Self {
        self.content.push_str(&MudFormatter::red(text));
        self
    }

    /// 添加绿色文本
    pub fn green(mut self, text: &str) -> Self {
        self.content.push_str(&MudFormatter::green(text));
        self
    }

    /// 添加黄色文本
    pub fn yellow(mut self, text: &str) -> Self {
        self.content.push_str(&MudFormatter::yellow(text));
        self
    }

    /// 添加蓝色文本
    pub fn blue(mut self, text: &str) -> Self {
        self.content.push_str(&MudFormatter::blue(text));
        self
    }

    /// 添加高亮文本
    pub fn highlight(mut self, text: &str) -> Self {
        self.content.push_str(&MudFormatter::highlight(text));
        self
    }

    /// 添加链接/按钮
    pub fn link(mut self, text: &str, command: &str) -> Self {
        self.content.push_str(&MudFormatter::link(text, command));
        self
    }

    /// 添加换行
    pub fn newline(mut self) -> Self {
        self.content.push_str("\\n");
        self
    }

    /// 添加多个换行
    pub fn newlines(mut self, count: usize) -> Self {
        for _ in 0..count {
            self.content.push_str("\\n");
        }
        self
    }

    /// 构建最终输出
    pub fn build(self) -> String {
        self.content
    }
}

impl Default for OutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 从JSON格式转换为MUD格式
pub fn json_to_mud(json: &JsonValue) -> String {
    if let Some(text) = json.as_str() {
        return text.to_string();
    }

    // 如果是对象，检查是否有text字段
    if let Some(obj) = json.as_object() {
        if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
            let mut result = text.to_string();

            // 处理颜色
            if let Some(color) = obj.get("color").and_then(|v| v.as_str()) {
                result = match color {
                    "red" => MudFormatter::red(&result),
                    "green" => MudFormatter::green(&result),
                    "yellow" => MudFormatter::yellow(&result),
                    "blue" => MudFormatter::blue(&result),
                    "highlight" => MudFormatter::highlight(&result),
                    _ => result,
                };
            }

            return result;
        }
    }

    json.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_codes() {
        assert_eq!(MudColor::Red.code(), "§R");
        assert_eq!(MudColor::Green.code(), "§G");
        assert_eq!(MudColor::Normal.code(), "§N");
    }

    #[test]
    fn test_colored_text() {
        let red = MudFormatter::red("Error");
        assert!(red.contains("§R"));
        assert!(red.contains("Error"));
        assert!(red.ends_with("§N"));
    }

    #[test]
    fn test_link() {
        let link = MudFormatter::link("Look", "look");
        assert_eq!(link, "[Look:look]");
    }

    #[test]
    fn test_output_builder() {
        let output = OutputBuilder::new()
            .text("Hello ")
            .red("World")
            .newline()
            .build();
        assert!(output.contains("Hello "));
        assert!(output.contains("§R"));
    }
}
