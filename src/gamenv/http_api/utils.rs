// gamenv/http_api/utils.rs - HTTP API 工具函数
// 对应 txpike9/gamenv/single/daemons/http_api/utils.pike

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
pub fn parse_color_codes(text: &str) -> String {
    // 颜色代码映射
    let colors = [
        ("§0", "#000000"), // 黑色
        ("§1", "#0000AA"), // 深蓝
        ("§2", "#00AA00"), // 深绿
        ("§3", "#00AAAA"), // 深青
        ("§4", "#AA0000"), // 深红
        ("§5", "#AA00AA"), // 深紫
        ("§6", "#FFAA00"), // 金色
        ("§7", "#AAAAAA"), // 灰色
        ("§8", "#555555"), // 深灰
        ("§9", "#5555FF"), // 蓝色
        ("§a", "#55FF55"), // 绿色
        ("§b", "#55FFFF"), // 青色
        ("§c", "#FF5555"), // 红色
        ("§d", "#FF55FF"), // 紫色
        ("§e", "#FFFF55"), // 黄色
        ("§f", "#FFFFFF"), // 白色
        ("§r", ""),         // 重置
    ];

    let mut result = text.to_string();
    for (code, color) in &colors {
        if *color == "" {
            result = result.replace(code, "</span>");
        } else {
            result = result.replace(code, &format!("<span style=\"color:{}\">", color));
        }
    }
    result
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
