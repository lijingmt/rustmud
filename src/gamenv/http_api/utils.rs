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
/// 颜色已针对黄色背景优化 (白色/黄色改为深色)
pub fn parse_color_codes(text: &str) -> String {
    // 数字代码 (0-9)
    let colors = [
        ("§0", "<span style='color:#000000'>"),      // 黑色-默认
        ("§1", "<span style='color:#CC0000;font-weight:bold'>"),  // 红色-夏季节气 (深红)
        ("§2", "<span style='color:#006600;font-weight:bold'>"),  // 绿色-优秀 (深绿)
        ("§3", "<span style='color:#004488;font-weight:bold'>"),  // 蓝色-稀有 (深蓝)
        ("§4", "<span style='color:#006699'>"),      // 青色-冬季节气 (深青)
        ("§5", "<span style='color:#660088;font-weight:bold'>"),  // 紫色-史诗 (深紫)
        ("§6", "<span style='color:#AA6600;font-weight:bold'>"),  // 金色-传说 (深金)
        ("§7", "<span style='color:#444444'>"),      // 灰色-普通
        ("§8", "<span style='color:#666666'>"),      // 灰色-劣质
        ("§9", "<span style='color:#888888'>"),      // 浅灰
        // 小写字母 (a-f)
        ("§a", "<span style='color:#008800'>"),      // 亮绿色 (深绿)
        ("§b", "<span style='color:#990099'>"),      // 紫红色 (深紫红)
        ("§c", "<span style='color:#AA2222'>"),      // 粉红色 (深粉)
        ("§d", "<span style='color:#AA4466'>"),      // 亮粉红 (深粉红)
        ("§e", "<span style='color:#885500'>"),      // 土黄色-标题 (深褐黄)
        ("§f", "<span style='color:#333333'>"),      // 深灰-默认
        // 大写字母 (A-F, Y)
        ("§A", "<span style='color:#008800;font-weight:bold'>"),  // 亮绿色-增强 (深绿)
        ("§B", "<span style='color:#0066AA'>"),      // 亮蓝色 (深蓝)
        ("§C", "<span style='color:#AA0000;font-weight:bold'>"),  // 鲜红色-稀有标记 (深红)
        ("§D", "<span style='color:#AA0055'>"),      // 深粉红 (深粉)
        ("§E", "<span style='color:#AA6600;font-weight:bold'>"),  // 金色-增强 (深金)
        ("§F", "<span style='color:#333333'>"),      // 深灰 (原白色)
        ("§Y", "<span style='color:#886600'>"),      // 深黄 (原黄色)
        ("§G", "<span style='color:#008800'>"),      // 绿色-深绿
        ("§R", "<span style='color:#AA0000'>"),      // 红色-深红
        ("§H", "<span style='color:#885500;font-weight:bold'>"),  // 金色/高亮-深褐黄
        ("§N", "<span style='color:#333333'>"),      // 深灰 (原白色)
        ("§W", "<span style='color:#333333'>"),      // 深灰 (原白色)
        ("§M", "<span style='color:#880088'>"),      // 紫色-深紫
        ("§X", "<span style='color:#880066'>"),      // 紫红色-深紫红
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
/// 颜色已针对黄色背景优化 (白色/黄色改为深色)
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
                // 数字代码 - 深色版本
                '0' => Some("#000000".to_string()),      // 黑色
                '1' => Some("#CC0000".to_string()),      // 深红
                '2' => Some("#006600".to_string()),      // 深绿
                '3' => Some("#004488".to_string()),      // 深蓝
                '4' => Some("#006699".to_string()),      // 深青
                '5' => Some("#660088".to_string()),      // 深紫
                '6' => Some("#AA6600".to_string()),      // 深金
                '7' => Some("#444444".to_string()),      // 深灰
                '8' => Some("#666666".to_string()),      // 灰色
                '9' => Some("#888888".to_string()),      // 浅灰
                // 小写字母 - 深色版本
                'a' => Some("#008800".to_string()),      // 深绿
                'b' => Some("#990099".to_string()),      // 深紫红
                'c' => Some("#AA2222".to_string()),      // 深粉
                'd' => Some("#AA4466".to_string()),      // 深粉红
                'e' => Some("#885500".to_string()),      // 深褐黄
                'f' => Some("#333333".to_string()),      // 深灰
                // 大写字母 - 深色版本
                'A' => Some("#008800".to_string()),      // 深绿
                'B' => Some("#0066AA".to_string()),      // 深蓝
                'C' => Some("#AA0000".to_string()),      // 深红
                'D' => Some("#AA0055".to_string()),      // 深粉
                'E' => Some("#AA6600".to_string()),      // 深金
                'F' => Some("#333333".to_string()),      // 深灰 (原白色)
                'Y' => Some("#886600".to_string()),      // 深黄 (原黄色)
                'G' => Some("#008800".to_string()),      // 深绿
                'H' => Some("#885500".to_string()),      // 深金
                'N' | 'W' => Some("#333333".to_string()), // 深灰 (原白色)
                'M' | 'X' => Some("#880088".to_string()), // 深紫
                'R' => Some("#AA0000".to_string()),      // 深红
                'r' => None,  // 重置颜色
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
