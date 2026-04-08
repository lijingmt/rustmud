// pikenv/pikenv.rs - 主服务器入口
// 对应 txpike9/pikenv/pikenv.pike

use crate::core::{MudError, Result};
use crate::pikenv::config::CONFIG;
use crate::pikenv::conn::accept_connection;
use crate::pikenv::gc_manager::GcManager;
use crate::pikenv::efuns::EFUNSD;
use tokio::net::TcpListener;
use tracing::{info, error, warn};

/// Pikenv 主服务器
pub struct PikenvServer {
    config: &'static crate::pikenv::config::Config,
}

impl PikenvServer {
    pub fn new() -> Self {
        Self {
            config: &CONFIG,
        }
    }

    /// 启动服务器 (对应 pikenv.pike 的 main())
    pub async fn run(&self) -> Result<()> {
        // 初始化日志
        self.init_logging()?;

        // 确保目录存在
        self.config.ensure_directories()?;

        // 启动 GC 管理器
        let gc_manager = GcManager::default();
        gc_manager.spawn();

        // 打印启动信息
        info!("==========================================");
        info!("RustMUD - 1:1 Port of txpike9");
        info!("==========================================");
        info!("GAME_AREA: {}", self.config.game_area);
        info!("ROOT: {}", self.config.root);
        info!("Listening on: {}:{}", self.config.ip, self.config.port);
        info!("==========================================");

        // 启动 TCP 监听
        let addr = format!("{}:{}", self.config.ip, self.config.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| MudError::RuntimeError(format!("Failed to bind {}: {}", addr, e)))?;

        info!("Server started successfully");

        // 接受连接循环 (对应 accept_callback)
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from: {}", addr);

                    // 创建用户对象工厂
                    let user_factory = || {
                        // TODO: 加载用户对象 (对应 master()->connect())
                        crate::core::GObject::new(
                            "guest".to_string(),
                            "/gamenv/clone/user".to_string(),
                        )
                    };

                    // 处理连接 (对应 CONN(ob, u()))
                    tokio::spawn(async move {
                        if let Err(e) = accept_connection(stream, user_factory).await {
                            error!("Connection error: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {:?}", e);
                }
            }
        }
    }

    /// 初始化日志系统
    fn init_logging(&self) -> Result<()> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

        let log_path = format!("{}/log/error.{}", self.config.root, self.config.log_prefix);
        let debug_log_path = format!("{}/log/stderr.{}", self.config.root, self.config.log_prefix);

        // 创建日志目录
        std::fs::create_dir_all(format!("{}/log", self.config.root))?;

        // 初始化 tracing
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_target(false)
            )
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
            ))
            .init();

        Ok(())
    }
}

impl Default for PikenvServer {
    fn default() -> Self {
        Self::new()
    }
}

/// 主入口函数 (对应 pikenv.pike 的 main())
#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let config = &CONFIG;

    // 检查命令行参数
    let args: Vec<String> = std::env::args().collect();

    // 处理命令行参数 (对应 pikenv.pike 的 Getopt)
    let mut server = PikenvServer::new();

    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            "-v" | "--version" => {
                println!("RustMUD - 1:1 Port of txpike9");
                println!("Version 0.1.0");
                return Ok(());
            }
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    // TODO: 设置端口
                }
            }
            _ => {}
        }
    }

    // 启动服务器
    server.run().await
}

/// 打印使用说明 (对应 usage())
fn print_usage() {
    println!("RustMUD - A Rust MUD engine, 1:1 port of txpike9");
    println!();
    println!("Usage: pikenv [OPTIONS] <mudlib directory> [script.pike] ...");
    println!();
    println!("Options:");
    println!("  -i, --ip=IP           Set binding IP (default: 0.0.0.0)");
    println!("  -p, --port=PORT       Set binding port (default: 9999)");
    println!("  -m, --master=MASTER   Set mudlib master path");
    println!("  -l, --logprefix=PREFIX Set log file prefix");
    println!("  -v, --version         Display version");
    println!("  -h, --help            Print this help");
    println!();
    println!("Environment variables:");
    println!("  GAME_AREA             Game area ID (default: tx01)");
    println!("  ROOT                  Mudlib root directory");
    println!("  PORT                  Listening port (default: 9999)");
    println!("  IP                    Binding IP (default: 0.0.0.0)");
}
