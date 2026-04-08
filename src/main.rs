// main.rs - RustMUD 主入口
// 对应 txpike9/pikenv/pikenv.pike 的启动入口

mod core;
mod pikenv;
mod gamenv;

use pikenv::pikenv::PikenvServer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 启动 pikenv 服务器
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let server = PikenvServer::new();
        match server.run().await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Server error: {:?}", e);
                Err(Box::new(e) as Box<dyn std::error::Error>)
            }
        }
    })
}
