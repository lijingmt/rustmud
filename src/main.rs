// main.rs - RustMUD 主入口
// 对应 txpike9/pikenv/pikenv.pike 的启动入口

mod core;
mod rustenv;
mod gamenv;

use rustenv::rustenv::RustenvServer;
use gamenv::http_api;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 启动 rustenv 服务器
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
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

        // 等待两个服务器
        tokio::select! {
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
