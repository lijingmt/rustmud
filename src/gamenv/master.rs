// gamenv/master.rs - 主控制器
// 对应 txpike9/gamenv/master.pike

use crate::core::*;
use crate::rustenv::config::CONFIG;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Master 对象 (对应 master.pike)
pub struct Master {
    /// 主机列表 (对应 hosts_list)
    pub hosts_list: Vec<String>,
    /// IP 地址
    pub ip: String,
    /// 端口
    pub port: u16,
    /// 视图模板 (对应 VIEWD)
    pub views: HashMap<String, ViewTemplate>,
}

/// 视图模板 (对应 VIEW)
#[derive(Debug, Clone)]
pub struct ViewTemplate {
    pub name: String,
    pub template: String,
}

impl ViewTemplate {
    pub fn new(name: String, template: String) -> Self {
        Self { name, template }
    }

    /// 渲染模板 (对应 VIEWD 的调用)
    pub fn render(&self, context: &HashMap<String, Value>) -> String {
        let mut result = self.template.clone();

        // 简单的模板替换 $(ob->method)
        for (key, value) in context {
            let placeholder = format!("$({})", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Int(i) => i.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => String::new(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        result
    }
}

impl Master {
    pub fn new() -> Self {
        Self {
            hosts_list: vec![format!("0.0.0.0:{}", CONFIG.port)],
            ip: CONFIG.ip.clone(),
            port: CONFIG.port,
            views: HashMap::new(),
        }
    }

    /// 初始化视图 (对应 master.pike 的 VIEWD 初始化)
    pub fn init_views(&mut self) {
        // /hp 视图
        self.views.insert(
            "/hp".to_string(),
            ViewTemplate::new("/hp".to_string(), String::from(
                "生命：$(jing)/$(jing_max)\n精神：$(shen)/$(shen_max)\n内力：$(qi)/$(qi_max)\n"
            ))
        );

        // /options 视图
        self.views.insert(
            "/options".to_string(),
            ViewTemplate::new("/options".to_string(), String::from(
                "[我的帮派:mybang][身体情况:hp][技能设定:skills][修炼内力:exercise]\n"
            ))
        );

        // 更多视图...
    }

    /// 对应 connect() - 创建用户对象
    pub fn connect(&self) -> Result<GObject> {
        // 对应 CHINAQUEST_USER
        let user_path = format!("{}/gamenv/clone/user", CONFIG.root);
        Ok(Arc::new(RwLock::new(ObjectInner::new("guest".to_string(), user_path))))
    }

    /// 对应 cast_to_program()
    pub fn cast_to_program(&self, pname: &str) -> Result<Program> {
        // TODO: 实现程序加载
        Err(MudError::ObjectNotFound(pname.to_string()))
    }

    /// 对应 cast_to_object()
    pub fn cast_to_object(&self, oname: &str) -> Result<GObject> {
        // TODO: 实现对象加载
        Ok(Arc::new(RwLock::new(ObjectInner::new(oname.to_string(), oname.to_string()))))
    }

    /// 处理错误 (对应 handle_error())
    pub fn handle_error(&self, error: &MudError) {
        tracing::error!("Master handled error: {:?}", error);
    }
}

impl Default for Master {
    fn default() -> Self {
        let mut master = Self::new();
        master.init_views();
        master
    }
}

/// 全局 Master 实例
pub static MASTER: once_cell::sync::Lazy<Master> =
    once_cell::sync::Lazy::new(Default::default);

/// 初始化 Daemons (对应 master.pike 的 daemon 初始化)
pub async fn init_daemons() {
    let config = &CONFIG;
    let daemons_path = format!("{}/gamenv/single/daemons", config.root);

    // 确保目录存在
    std::fs::create_dir_all(&daemons_path).ok();

    // TODO: 加载所有 daemons
    tracing::info!("Loading daemons from: {}", daemons_path);
}
