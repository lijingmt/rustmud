// gamenv/user.rs - 用户对象
// 对应 txpike9/gamenv/clone/user.pike

use crate::core::*;
use crate::pikenv::config::CONFIG;

/// 用户对象 (对应 /gamenv/clone/user)
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
    pub fn save(&self) -> Result<()> {
        let user_path = format!("{}/gamenv/u/{}.json", CONFIG.root, self.name);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(user_path, json)?;
        Ok(())
    }

    /// 恢复用户数据 (对应 restore_object)
    pub fn load(&mut self) -> Result<()> {
        let user_path = format!("{}/gamenv/u/{}.json", CONFIG.root, self.name);
        if std::path::Path::new(&user_path).exists() {
            let json = std::fs::read_to_string(user_path)?;
            let loaded: User = serde_json::from_str(&json)?;
            // 复制属性
            self.hp = loaded.hp;
            self.hp_max = loaded.hp_max;
            self.level = loaded.level;
            self.exp = loaded.exp;
            // ...
        }
        Ok(())
    }

    /// 发送提示符 (对应 write_prompt())
    pub fn write_prompt(&self) -> String {
        format!("> ")
    }
}

impl serde::Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("User", 10)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("name_cn", &self.name_cn)?;
        state.serialize_field("level", &self.level)?;
        state.serialize_field("exp", &self.exp)?;
        state.serialize_field("hp", &self.hp)?;
        state.serialize_field("hp_max", &self.hp_max)?;
        state.serialize_field("qi", &self.qi)?;
        state.serialize_field("qi_max", &self.qi_max)?;
        state.serialize_field("money", &self.money)?;
        state.serialize_field("room_id", &self.room_id)?;
        state.end()
    }
}
