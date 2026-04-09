// gamenv/http_api/utils.rs - HTTP API 工具函数
// 对应 txpike9/gamenv/single/daemons/http_api/utils.pike

use serde::{Deserialize, Serialize};

/// 隐藏命令 (对应 hide_command)
pub fn hide_command(command: &str) -> String {
    // 将命令替换为占位符，用于日志记录
    // 例如: password "secret" -> password "******"
    if is_sensitive_command(command) {
        return format!("{}, ***REDACTED***",
            command.split_whitespace().next().unwrap_or("unknown")
        );
    }
    command.to_string()
}

/// 判断是否为敏感命令
fn is_sensitive_command(command: &str) -> bool {
    let cmd = command.split_whitespace().next().unwrap_or("");
    matches!(cmd,
        "password" | "passwd" | "login" |
        "set_password" | "changepass" |
        "secret" | "token"
    )
}

/// 清理 HTML 特殊字符
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// 验证用户名格式
pub fn validate_username(username: &str) -> bool {
    if username.len() < 3 || username.len() > 16 {
        return false;
    }
    username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// 生成会话 ID
pub fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

/// 解析颜色代码 (对应 txpike9 的 § 颜色代码)
/// 完整的颜色映射，包含数字、小写字母、大写字母和特殊代码
pub fn parse_color_codes(text: &str) -> String {
    // 数字代码 (0-9)
    let colors = [
        ("§0", "<span style='color:#000000'>"),      // 黑色-默认
        ("§1", "<span style='color:#FF0000;font-weight:bold'>"),  // 红色-夏季节气
        ("§2", "<span style='color:#00AA00;font-weight:bold'>"),  // 绿色-优秀
        ("§3", "<span style='color:#0066CC;font-weight:bold'>"),  // 蓝色-稀有
        ("§4", "<span style='color:#00AAFF'>"),      // 青色-冬季节气
        ("§5", "<span style='color:#8B00FF;font-weight:bold'>"),  // 紫色-史诗
        ("§6", "<span style='color:#FF8C00;font-weight:bold'>"),  // 金色-传说
        ("§7", "<span style='color:#666666'>"),      // 白色-普通
        ("§8", "<span style='color:#888888'>"),      // 灰色-劣质
        ("§9", "<span style='color:#CCCCCC'>"),      // 浅灰
        // 小写字母 (a-f)
        ("§a", "<span style='color:#00CC00'>"),      // 亮绿色
        ("§b", "<span style='color:#FF00FF'>"),      // 紫红色
        ("§c", "<span style='color:#FF3366'>"),      // 粉红色
        ("§d", "<span style='color:#FF6699'>"),      // 亮粉红
        ("§e", "<span style='color:#CC8800'>"),      // 土黄色-标题
        ("§f", "<span style='color:#333333'>"),      // 白色-默认
        // 大写字母 (A-F, Y)
        ("§A", "<span style='color:#00FF00;font-weight:bold'>"),  // 亮绿色-增强
        ("§B", "<span style='color:#0099FF'>"),      // 亮蓝色
        ("§C", "<span style='color:#FF0000;font-weight:bold'>"),  // 鲜红色-稀有标记
        ("§D", "<span style='color:#FF1493'>"),      // 深粉红
        ("§E", "<span style='color:#FFD700;font-weight:bold'>"),  // 金色-增强
        ("§F", "<span style='color:#FFFFFF'>"),      // 纯白色
        ("§Y", "<span style='color:#FFFF00'>"),      // 黄色
        ("§G", "<span style='color:#00FF00'>"),      // 绿色(Good)
        ("§R", "<span style='color:#FF0000'>"),      // 红色(Red)
        ("§H", "<span style='color:#FFD700;font-weight:bold'>"),  // 金色/高亮(Highlight)
        ("§N", "<span style='color:#FFFFFF'>"),      // 白色(Normal)
        ("§W", "<span style='color:#FFFFFF'>"),      // 白色(White)
        ("§B", "<span style='color:#0099FF'>"),      // 蓝色(Blue)
        ("§M", "<span style='color:#FF00FF'>"),      // 紫色(Magenta)
        ("§C", "<span style='color:#00FFFF'>"),      // 青色(Cyan)
        ("§X", "<span style='color:#FF00FF'>"),      // 紫红色
        // 特殊代码
        ("§r", "</span>"),   // 重置-小写
        ("§g", "<span class='ink-wash-gradient'>"),  // 水墨渐变
    ];

    let mut result = text.to_string();
    for (code, replacement) in &colors {
        result = result.replace(code, replacement);
    }
    // 处理 §R (大写R) 作为重置
    result = result.replace("§R", "</span>");
    result
}

/// 解析颜色代码为 TextPart 列表（用于 JSON 格式输出）
pub fn parse_color_codes_to_parts(text: &str) -> Vec<TextPart> {
    let mut parts = Vec::new();
    let mut current_text = String::new();
    let mut current_color: Option<String> = None;
    let mut current_bold: Option<bool> = None;
    let chars: Vec<char> = text.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        // 检查颜色代码 §X
        if i + 1 < chars.len() && chars[i] == '§' {
            // 保存之前的文本
            if !current_text.is_empty() {
                parts.push(TextPart {
                    text: current_text.clone(),
                    color: current_color.clone(),
                    bold: current_bold,
                    underline: None,
                    link: None,
                });
                current_text.clear();
            }

            // 获取颜色代码字符
            let color_char = chars[i + 1];
            let color = match color_char {
                // 数字代码
                '0' => Some("#000000".to_string()),
                '1' => Some("#FF0000".to_string()),
                '2' => Some("#00AA00".to_string()),
                '3' => Some("#0066CC".to_string()),
                '4' => Some("#00AAFF".to_string()),
                '5' => Some("#8B00FF".to_string()),
                '6' => Some("#FF8C00".to_string()),
                '7' => Some("#666666".to_string()),
                '8' => Some("#888888".to_string()),
                '9' => Some("#CCCCCC".to_string()),
                // 小写字母
                'a' => Some("#00CC00".to_string()),
                'b' => Some("#FF00FF".to_string()),
                'c' => Some("#FF3366".to_string()),
                'd' => Some("#FF6699".to_string()),
                'e' => Some("#CC8800".to_string()),
                'f' => Some("#333333".to_string()),
                // 大写字母
                'A' => Some("#00FF00".to_string()),
                'B' => Some("#0099FF".to_string()),
                'C' => Some("#FF0000".to_string()),
                'D' => Some("#FF1493".to_string()),
                'E' => Some("#FFD700".to_string()),
                'F' => Some("#FFFFFF".to_string()),
                'Y' => Some("#FFFF00".to_string()),
                'G' => Some("#00FF00".to_string()),
                'H' => Some("#FFD700".to_string()),
                'N' | 'W' => Some("#FFFFFF".to_string()),
                'M' | 'X' => Some("#FF00FF".to_string()),
                'r' | 'R' => None,  // 重置颜色
                _ => {
                    // 未知代码，当作普通文本
                    current_text.push('§');
                    i += 1;
                    continue;
                }
            };

            // 更新当前颜色和粗体
            match color_char {
                'r' | 'R' => {
                    current_color = None;
                    current_bold = None;
                }
                '1' | '2' | '3' | '5' | '6' | 'A' | 'C' | 'E' | 'H' => {
                    current_color = color;
                    current_bold = Some(true);
                }
                _ => {
                    current_color = color;
                }
            }

            i += 2;  // 跳过 §X
        } else {
            current_text.push(chars[i]);
            i += 1;
        }
    }

    // 保存最后一段文本
    if !current_text.is_empty() {
        parts.push(TextPart {
            text: current_text,
            color: current_color,
            bold: current_bold,
            underline: None,
            link: None,
        });
    }

    if parts.is_empty() {
        parts.push(TextPart {
            text: text.to_string(),
            color: None,
            bold: None,
            underline: None,
            link: None,
        });
    }

    parts
}

/// TextPart 结构 - 用于带颜色的文本片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPart {
    pub text: String,
    pub color: Option<String>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
    pub link: Option<String>,
}

/// 验证 TXD Token 格式
pub fn validate_txd_format(txd: &str) -> bool {
    if !txd.contains('~') {
        return false;
    }
    let parts: Vec<&str> = txd.splitn(2, '~').collect();
    parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hide_command() {
        assert_eq!(hide_command("password secret123"), "password, ***REDACTED***");
        assert_eq!(hide_command("look"), "look");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_parse_color_codes() {
        let result = parse_color_codes("§cRed text§r normal");
        assert!(result.contains("color:#FF5555"));
    }
}
