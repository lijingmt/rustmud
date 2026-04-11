// main.rs - RustMUD 主入口
// 对应 txpike9/pikenv/pikenv.pike 的启动入口

mod core;
mod rustenv;
mod gamenv;

use rustenv::rustenv::RustenvServer;
use gamenv::http_api;
use gamenv::quest::QUESTD;
use gamenv::single::daemons::pkd::{PkDaemon, get_pkd};
// 新架构：克隆模板注册表 + 世界状态
use gamenv::clone::{init_item_templates, init_npc_templates};
use gamenv::efuns::init_world_state;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 tracing（必须在所有 tokio::spawn 之前）
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
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

    // 启动 rustenv 服务器
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // 初始化任务系统
        let root_dir = std::env::var("ROOT").unwrap_or_else(|_| "/usr/local/games/rust".to_string());
        let data_dir = format!("{}/data", root_dir);
        if let Err(e) = QUESTD.initialize(&data_dir).await {
            eprintln!("Failed to initialize quest system: {:?}", e);
        }

        // 初始化克隆模板注册表 (新架构)
        init_item_templates().await;
        init_npc_templates().await;
        tracing::info!("Clone template registries initialized");

        // 初始化世界状态 (核心MUD操作: move_object, environment, destruct)
        init_world_state().await;
        tracing::info!("World state (efuns) initialized");

        // 获取 PKD 守护进程实例
        let pkd = get_pkd().await;

        // 启动三个任务，它们将同时运行
        tokio::spawn(async move {
            tracing::info!("PKD heartbeat task started");
            PkDaemon::start_heartbeat_task(pkd).await;
            eprintln!("PKD heartbeat task stopped unexpectedly");
        });

        tokio::spawn(async {
            let router = http_api::create_router();
            let addr = "0.0.0.0:8081";
            tracing::info!("HTTP API listening on {}", addr);
            if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
                if let Err(e) = axum::serve(listener, router).await {
                    eprintln!("HTTP API server error: {:?}", e);
                }
            }
        });

        tokio::spawn(async {
            let server = RustenvServer::new();
            if let Err(e) = server.run().await {
                eprintln!("Game server error: {:?}", e);
            }
        });

        // 主任务永远运行，等待 Ctrl+C 或其他信号
        // 在生产环境中，应该使用 tokio::signal::ctrl_c() 等
        tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
        Ok(())
    })
}
