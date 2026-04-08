// pikenv/config.rs - 配置系统
// 对应 txpike9 的环境变量配置

use std::env;

/// 全局配置 (对应 .include/sys_config.h)
#[derive(Debug, Clone)]
pub struct Config {
    /// SROOT - pikenv 根目录
    pub sroot: String,
    /// ROOT - mudlib 根目录
    pub root: String,
    /// GAME_AREA - 游戏区号
    pub game_area: String,
    /// 监听端口
    pub port: u16,
    /// 监听 IP
    pub ip: String,
    /// 日志文件前缀
    pub log_prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let game_area = env::var("GAME_AREA")
            .unwrap_or_else(|_| "tx01".to_string());

        // 确保 GAME_AREA 格式正确
        let game_area = if !game_area.starts_with("tx") {
            format!("tx{}", game_area)
        } else {
            game_area
        };

        let root = env::var("ROOT")
            .unwrap_or_else(|_| {
                env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "/usr/local/games/rustmud".to_string())
            });

        Self {
            sroot: env::var("SROOT")
                .unwrap_or_else(|_| root.clone()),
            root: root.clone(),
            game_area,
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(9999),
            ip: env::var("IP")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            log_prefix: env::var("LOG_PREFIX")
                .unwrap_or_else(|_| "9999".to_string()),
        }
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        Self::new()
    }

    /// 检查并创建必要目录
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(format!("{}/log", self.root))?;
        std::fs::create_dir_all(format!("{}/gamenv/u", self.root))?;
        std::fs::create_dir_all(format!("{}/gamenv/single/daemons", self.root))?;
        Ok(())
    }
}

/// 全局配置实例
pub static CONFIG: once_cell::sync::Lazy<Config> =
    once_cell::sync::Lazy::new(Config::new);
