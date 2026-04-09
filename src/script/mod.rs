// script/mod.rs - Rhai 脚本引擎集成（简化版）
// 对应 txpike9 的动态代码执行能力

use rhai::{Engine, EvalAltResult};
use crate::core::MudError;
use std::sync::Mutex;

/// 创建一个新的脚本引擎
pub fn create_engine() -> Engine {
    let mut engine = Engine::new();

    // === 注册游戏 API 供脚本调用 ===

    // 玩家相关
    engine.register_fn("send_msg", |_player_id: i64, _msg: &str| {
        tracing::debug!("Script: send_msg called");
    });

    engine.register_fn("query_name", |player_id: i64| -> String {
        format!("Player_{}", player_id)
    });

    engine.register_fn("query_hp", |_player_id: i64| -> i64 {
        100
    });

    engine.register_fn("add_hp", |_player_id: i64, _amount: i64| {
        tracing::debug!("Script: add_hp called");
    });

    engine.register_fn("sub_hp", |_player_id: i64, _amount: i64| {
        tracing::debug!("Script: sub_hp called");
    });

    // 战斗相关
    engine.register_fn("damage", |_target_id: i64, amount: i64| -> i64 {
        amount
    });

    engine.register_fn("random", |min: i64, max: i64| -> i64 {
        use rand::Rng;
        rand::thread_rng().gen_range(min..=max)
    });

    engine.register_fn("random_percent", || -> i64 {
        use rand::Rng;
        rand::thread_rng().gen_range(1..=100)
    });

    // 装备/物品相关
    engine.register_fn("equip_name", |item_id: i64| -> String {
        format!("Item_{}", item_id)
    });

    engine.register_fn("equip_attack", |_item_id: i64| -> i64 {
        100
    });

    engine.register_fn("equip_defense", |_item_id: i64| -> i64 {
        50
    });

    // 日志
    engine.register_fn("log", |msg: &str| {
        tracing::info!("Script log: {}", msg);
    });

    engine.register_fn("debug", |msg: &str| {
        tracing::debug!("Script debug: {}", msg);
    });

    // 时间相关
    engine.register_fn("time", || -> i64 {
        chrono::Utc::now().timestamp()
    });

    engine
}

/// 全局引擎（单线程使用）
static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

/// 初始化全局引擎
fn init_engine() {
    let mut engine = ENGINE.lock().unwrap();
    if engine.is_none() {
        *engine = Some(create_engine());
    }
}

/// 获取全局引擎
pub fn get_engine() -> Result<Engine, MudError> {
    init_engine();
    // 注意：这里返回一个新的引擎克隆，因为 Engine 不可跨线程共享
    // 在实际使用中，应该在需要时调用 create_engine()
    Ok(create_engine())
}

/// 执行脚本字符串
pub fn eval<T: Clone + Send + Sync + 'static>(script: &str) -> Result<T, MudError>
where
    T: for<'a> TryFrom<rhai::Dynamic, Error = Box<EvalAltResult>>,
{
    let engine = create_engine();
    engine.eval(script)
        .map_err(|e| MudError::RuntimeError(e.to_string()))
}

/// 脚本管理器（只存储脚本字符串）
pub struct ScriptManager {
    equip_scripts: std::collections::HashMap<String, String>,
    room_scripts: std::collections::HashMap<String, String>,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self {
            equip_scripts: std::collections::HashMap::new(),
            room_scripts: std::collections::HashMap::new(),
        }
    }

    /// 加载装备脚本
    pub fn load_equip_script(&mut self, path: String, script: String) -> Result<(), MudError> {
        self.equip_scripts.insert(path, script);
        Ok(())
    }

    /// 加载房间脚本
    pub fn load_room_script(&mut self, path: String, script: String) -> Result<(), MudError> {
        self.room_scripts.insert(path, script);
        Ok(())
    }

    /// 获取装备脚本
    pub fn get_equip_script(&self, path: &str) -> Option<String> {
        self.equip_scripts.get(path).cloned()
    }

    /// 获取房间脚本
    pub fn get_room_script(&self, path: &str) -> Option<String> {
        self.room_scripts.get(path).cloned()
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局脚本管理器实例
pub static SCRIPT_MANAGER: once_cell::sync::Lazy<Mutex<ScriptManager>> =
    once_cell::sync::Lazy::new(|| Mutex::new(ScriptManager::new()));
