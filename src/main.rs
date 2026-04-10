// main.rs - RustMUD 主入口
// 对应 txpike9/pikenv/pikenv.pike 的启动入口

mod core;
mod rustenv;
mod gamenv;

use rustenv::rustenv::RustenvServer;
use gamenv::http_api;
use gamenv::quest::QUESTD;
use gamenv::single::daemons::pkd::{PkDaemon, get_pkd};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 启动 rustenv 服务器
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // 初始化任务系统
        let root_dir = std::env::var("ROOT").unwrap_or_else(|_| "/usr/local/games/rust".to_string());
        let data_dir = format!("{}/data", root_dir);
        if let Err(e) = QUESTD.initialize(&data_dir).await {
            eprintln!("Failed to initialize quest system: {:?}", e);
        }

        // 启动 PKD 心跳任务（处理NPC自动战斗）
        let pkd = get_pkd().await;
        let heartbeat_handle = tokio::spawn(async {
            tracing::info!("PKD heartbeat task started");
            PkDaemon::start_heartbeat_task(pkd).await;
        });

        // 启动 HTTP API 服务器
        let http_handle = tokio::spawn(async {
            let router = http_api::create_router();
            let addr = "0.0.0.0:8081";
            tracing::info!("HTTP API listening on {}", addr);
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, router).await.unwrap();
        });

        // 启动游戏服务器
        let game_handle = tokio::spawn(async {
            let server = RustenvServer::new();
            if let Err(e) = server.run().await {
                eprintln!("Game server error: {:?}", e);
            }
        });

        // 等待任务
        tokio::select! {
            _ = heartbeat_handle => {
                eprintln!("PKD heartbeat task stopped");
                Ok(())
            }
            _ = http_handle => {
                eprintln!("HTTP API server stopped");
                Ok(())
            }
            _ = game_handle => {
                eprintln!("Game server stopped");
                Ok(())
            }
        }
    })
}
