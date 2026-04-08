// gamenv/http_api/config.rs - HTTP API 配置
// 对应 txpike9/gamenv/single/daemons/http_api/config.pike

/// HTTP API 配置
#[derive(Debug, Clone)]
pub struct HttpApiConfig {
    /// HTTP 端口
    pub http_port: u16,
    /// WebSocket 端口
    pub ws_port: u16,
    /// 命令隐藏启用
    pub command_hide_enabled: bool,
    /// 速率限制
    pub rate_limit: usize,
    /// 连接超时 (秒)
    pub connection_timeout: u64,
}

impl Default for HttpApiConfig {
    fn default() -> Self {
        Self {
            http_port: 8081,  // Changed from 8080 to avoid conflict with Tomcat
            ws_port: 8081,    // Changed from 8080 to avoid conflict with Tomcat
            command_hide_enabled: true,
            rate_limit: 100, // 每分钟100个请求
            connection_timeout: 1800, // 30分钟
        }
    }
}

impl HttpApiConfig {
    pub fn from_env() -> Self {
        Self {
            http_port: std::env::var("HTTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8081),
            ws_port: std::env::var("WS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8081),
            ..Default::default()
        }
    }
}
