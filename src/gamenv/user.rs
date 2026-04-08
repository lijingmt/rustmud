// gamenv/user.rs - 用户对象
// 对应 txpike9/gamenv/clone/user.pike

use crate::core::*;
use crate::pikenv::config::CONFIG;
use crate::pikenv::pike_save::{parse_pike_save_file, PikeValue, get_user_save_path, user_file_exists};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 用户对象 (对应 /gamenv/clone/user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: ObjectId,
    pub name: String,
    pub name_cn: String,
    pub level: u32,
    pub exp: u64,
    pub hp: i32,
    pub hp_max: i32,
    pub qi: i32,       // 内力
    pub qi_max: i32,
    pub shen: i32,     // 精神
    pub shen_max: i32,
    pub potential: i32, // 潜能
    pub money: u64,
    pub room_id: Option<String>,
    // txpike9 兼容字段
    pub password: Option<String>,
    pub login_time: Option<i64>,
    pub online_time: Option<i64>,
    pub first_login: Option<i64>,
    pub userip: Option<String>,
    // 扩展数据 (兼容 txpike9 的 data 字段)
    pub extra_data: serde_json::Value,
}

impl User {
    /// 创建新用户
    pub fn new(name: String) -> Self {
        Self {
            id: ObjectId::new(),
            name: name.clone(),
            name_cn: name,
            level: 1,
            exp: 0,
            hp: 100,
            hp_max: 100,
            qi: 50,
            qi_max: 50,
            shen: 50,
            shen_max: 50,
            potential: 100,
            money: 0,
            room_id: None,
            password: None,
            login_time: None,
            online_time: None,
            first_login: None,
            userip: None,
            extra_data: serde_json::json!({}),
        }
    }

    /// 登录处理 (对应 logon())
    pub async fn logon(&mut self) -> Result<String> {
        // TODO: 实现登录流程
        Ok("欢迎使用 RustMUD！\n".to_string())
    }

    /// 移动到房间
    pub fn move_to(&mut self, room_id: String) {
        self.room_id = Some(room_id);
    }

    /// 保存用户数据 (对应 save_object)
    /// 同时保存 JSON 格式 (RustMUD) 和 Pike 格式 (txpike9 兼容)
    pub fn save(&self) -> Result<()> {
        // 保存 JSON 格式
        let user_dir = format!("{}/gamenv/u", CONFIG.root);
        std::fs::create_dir_all(&user_dir)?;
        let user_path = format!("{}/{}.json", user_dir, self.name);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(user_path, json)?;

        // 保存 Pike 格式 (txpike9 兼容)
        self.save_pike_format()?;

        Ok(())
    }

    /// 保存为 Pike save_object 格式 (txpike9 兼容)
    fn save_pike_format(&self) -> Result<()> {
        let pike_path = get_user_save_path(&CONFIG.root, &self.name);
        let dir = Path::new(&pike_path).parent().unwrap();
        std::fs::create_dir_all(dir)?;

        let mut content = format!("#~/gamenv/clone/user.pike\n");
        content.push_str(&format!("name \"{}\"\n", self.name));
        content.push_str(&format!("name_newbei \"{}\"\n", self.name_cn));
        content.push_str(&format!("level {}\n", self.level));
        content.push_str(&format!("exp {}\n", self.exp));
        content.push_str(&format!("hp {}\n", self.hp));
        content.push_str(&format!("hp_max {}\n", self.hp_max));
        content.push_str(&format!("qi {}\n", self.qi));
        content.push_str(&format!("qi_max {}\n", self.qi_max));
        content.push_str(&format!("shen {}\n", self.shen));
        content.push_str(&format!("shen_max {}\n", self.shen_max));
        content.push_str(&format!("potential {}\n", self.potential));
        content.push_str(&format!("money {}\n", self.money));

        if let Some(ref pwd) = self.password {
            content.push_str(&format!("password \"{}\"\n", pwd));
        }
        if let Some(login_time) = self.login_time {
            content.push_str(&format!("login_time {}\n", login_time));
        }
        if let Some(online_time) = self.online_time {
            content.push_str(&format!("online_time {}\n", online_time));
        }

        // 空数据字段
        content.push_str("msgs ([])\n");
        content.push_str("inbox ({})\n");
        content.push_str("inventory_data ({})\n");
        content.push_str("skill_data ({})\n");
        content.push_str("data ({})\n");

        std::fs::write(pike_path, content)?;
        Ok(())
    }

    /// 恢复用户数据 (对应 restore_object)
    /// 优先尝试从 txpike9 格式加载，如果不存在则从 JSON 格式加载
    pub fn load(&mut self) -> Result<bool> {
        let name = self.name.clone();

        // 首先尝试从 txpike9 格式加载
        let pike_path = get_user_save_path(&CONFIG.root, &name);
        if Path::new(&pike_path).exists() {
            return self.load_from_pike(&pike_path);
        }

        // 然后尝试从 JSON 格式加载
        let json_path = format!("{}/gamenv/u/{}.json", CONFIG.root, name);
        if Path::new(&json_path).exists() {
            return self.load_from_json(&json_path);
        }

        Ok(false)
    }

    /// 从 txpike9 Pike 格式加载用户数据
    fn load_from_pike(&mut self, path: &str) -> Result<bool> {
        let save_data = parse_pike_save_file(path)?;

        // 解析基本字段
        if let Some(PikeValue::String(name)) = save_data.variables.get("name") {
            self.name = name.clone();
        }
        if let Some(PikeValue::String(name_cn)) = save_data.variables.get("name_newbei") {
            self.name_cn = name_cn.clone();
        }
        if let Some(level) = save_data.variables.get("level").and_then(|v| v.as_int()) {
            self.level = level as u32;
        }
        if let Some(exp) = save_data.variables.get("exp").and_then(|v| v.as_int()) {
            self.exp = exp as u64;
        }
        if let Some(hp) = save_data.variables.get("hp").and_then(|v| v.as_int()) {
            self.hp = hp as i32;
        }
        if let Some(hp_max) = save_data.variables.get("hp_max").and_then(|v| v.as_int()) {
            self.hp_max = hp_max as i32;
        }
        if let Some(qi) = save_data.variables.get("qi").and_then(|v| v.as_int()) {
            self.qi = qi as i32;
        }
        if let Some(qi_max) = save_data.variables.get("qi_max").and_then(|v| v.as_int()) {
            self.qi_max = qi_max as i32;
        }
        if let Some(shen) = save_data.variables.get("shen").and_then(|v| v.as_int()) {
            self.shen = shen as i32;
        }
        if let Some(shen_max) = save_data.variables.get("shen_max").and_then(|v| v.as_int()) {
            self.shen_max = shen_max as i32;
        }
        if let Some(potential) = save_data.variables.get("potential").and_then(|v| v.as_int()) {
            self.potential = potential as i32;
        }
        if let Some(money) = save_data.variables.get("money").and_then(|v| v.as_int()) {
            self.money = money as u64;
        }
        if let Some(PikeValue::String(password)) = save_data.variables.get("password") {
            self.password = Some(password.clone());
        }
        if let Some(login_time) = save_data.variables.get("login_time").and_then(|v| v.as_int()) {
            self.login_time = Some(login_time);
        }
        if let Some(online_time) = save_data.variables.get("online_time").and_then(|v| v.as_int()) {
            self.online_time = Some(online_time);
        }
        if let Some(first_login) = save_data.variables.get("first_login").and_then(|v| v.as_int()) {
            self.first_login = Some(first_login);
        }
        if let Some(PikeValue::String(userip)) = save_data.variables.get("userip") {
            self.userip = Some(userip.clone());
        }

        tracing::info!("Loaded user {} from txpike9 format", self.name);
        Ok(true)
    }

    /// 从 JSON 格式加载用户数据
    fn load_from_json(&mut self, path: &str) -> Result<bool> {
        let json = std::fs::read_to_string(path)?;
        let loaded: User = serde_json::from_str(&json)?;
        // 复制属性
        self.id = loaded.id;
        self.name = loaded.name;
        self.name_cn = loaded.name_cn;
        self.level = loaded.level;
        self.exp = loaded.exp;
        self.hp = loaded.hp;
        self.hp_max = loaded.hp_max;
        self.qi = loaded.qi;
        self.qi_max = loaded.qi_max;
        self.shen = loaded.shen;
        self.shen_max = loaded.shen_max;
        self.potential = loaded.potential;
        self.money = loaded.money;
        self.room_id = loaded.room_id;
        self.password = loaded.password;
        self.login_time = loaded.login_time;
        self.online_time = loaded.online_time;
        self.first_login = loaded.first_login;
        self.userip = loaded.userip;
        self.extra_data = loaded.extra_data;

        tracing::info!("Loaded user {} from JSON format", self.name);
        Ok(true)
    }

    /// 检查用户文件是否存在
    pub fn exists(&self) -> bool {
        user_file_exists(&CONFIG.root, &self.name)
    }

    /// 发送提示符 (对应 write_prompt())
    pub fn write_prompt(&self) -> String {
        format!("> ")
    }
}
