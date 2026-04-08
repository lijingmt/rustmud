// gamenv/daemons.rs - Daemon 系统
// 对应 txpike9/gamenv/single/daemons/ 目录

use crate::core::*;
use std::collections::HashMap;

/// Daemon Trait
pub trait Daemon: Send + Sync {
    fn name(&self) -> &str;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}

/// Daemon 管理器 (对应 daemons 的加载和管理)
pub struct DaemonManager {
    daemons: HashMap<String, Box<dyn Daemon>>,
}

impl DaemonManager {
    pub fn new() -> Self {
        Self {
            daemons: HashMap::new(),
        }
    }

    /// 注册 Daemon
    pub fn register(&mut self, daemon: Box<dyn Daemon>) {
        let name = daemon.name().to_string();
        self.daemons.insert(name, daemon);
    }

    /// 启动所有 Daemons (对应 master.pike 的 daemon 初始化)
    pub fn start_all(&mut self) -> Result<()> {
        for (name, daemon) in self.daemons.iter_mut() {
            tracing::info!("Starting daemon: {}", name);
            daemon.start()?;
        }
        Ok(())
    }

    /// 获取 Daemon
    pub fn get(&self, name: &str) -> Option<&dyn Daemon> {
        self.daemons.get(name).map(|d| d.as_ref())
    }
}

impl Default for DaemonManager {
    fn default() -> Self {
        Self::new()
    }
}

// ========== 具体 Daemon 实现 ==========

/// 用户管理 Daemon (对应 userd.pike)
pub struct UserDaemon {
    name: String,
}

impl UserDaemon {
    pub fn new() -> Self {
        Self {
            name: "userd".to_string(),
        }
    }

    /// 处理用户登录 (对应 do_login)
    pub fn do_login(&self, user: &GObject) -> Result<()> {
        tracing::info!("User logged in: {:?}", user);
        Ok(())
    }
}

impl Daemon for UserDaemon {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&mut self) -> Result<()> {
        tracing::info!("UserDaemon started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

/// 副本管理 Daemon (对应 fbd.pike)
pub struct FbDaemon {
    name: String,
}

impl FbDaemon {
    pub fn new() -> Self {
        Self {
            name: "fbd".to_string(),
        }
    }
}

impl Daemon for FbDaemon {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&mut self) -> Result<()> {
        tracing::info!("FbDaemon started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

/// 排行榜 Daemon (对应 toptend.pike)
pub struct ToptendDaemon {
    name: String,
}

impl ToptendDaemon {
    pub fn new() -> Self {
        Self {
            name: "toptend".to_string(),
        }
    }
}

impl Daemon for ToptendDaemon {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&mut self) -> Result<()> {
        tracing::info!("ToptendDaemon started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}
