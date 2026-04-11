// gamenv/clone/npc.rs - NPC模板系统
// 对应 txpike9/gamenv/clone/npc/ 目录
//
// NPC模板用于创建可克隆的NPC实例

use crate::gamenv::entities::npc::{DialogueOption, Npc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// NPC模板 - 定义NPC的基础属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcTemplate {
    /// 模板ID (唯一标识)
    pub id: String,
    /// NPC名称
    pub name: String,
    /// 中文名称
    pub name_cn: String,
    /// 描述
    pub description: String,
    /// 等级
    pub level: u32,
    /// 生命值
    pub hp: i32,
    /// 最大生命值
    pub hp_max: i32,
    /// 攻击力
    pub attack: i32,
    /// 防御力
    pub defense: i32,
    /// 速度
    pub speed: i32,
    /// 对话选项
    pub dialogues: Vec<DialogueOption>,
    /// 初始房间ID
    pub start_room: Option<String>,
    /// 是否可攻击
    pub attackable: bool,
    /// 扩展属性
    pub extra_data: HashMap<String, serde_json::Value>,
}

impl NpcTemplate {
    /// 创建新NPC模板
    pub fn new(id: String, name: String, name_cn: String) -> Self {
        Self {
            id,
            name,
            name_cn,
            description: String::new(),
            level: 1,
            hp: 100,
            hp_max: 100,
            attack: 10,
            defense: 5,
            speed: 10,
            dialogues: Vec::new(),
            start_room: None,
            attackable: false,
            extra_data: HashMap::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// 设置等级
    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    /// 设置属性
    pub fn with_stats(mut self, hp: i32, attack: i32, defense: i32) -> Self {
        self.hp = hp;
        self.hp_max = hp;
        self.attack = attack;
        self.defense = defense;
        self
    }

    /// 添加对话选项
    pub fn with_dialogue(mut self, topic: String, response: String) -> Self {
        self.dialogues.push(DialogueOption { topic, response });
        self
    }

    /// 设置初始房间
    pub fn with_start_room(mut self, room: String) -> Self {
        self.start_room = Some(room);
        self
    }

    /// 设置可攻击
    pub fn with_attackable(mut self, attackable: bool) -> Self {
        self.attackable = attackable;
        self
    }

    /// 从模板创建NPC实例
    pub fn instantiate(&self) -> Npc {
        let mut npc = Npc::new(
            format!("{}_{}", self.id, chrono::Utc::now().timestamp_millis()),
            self.id.clone(),
            self.name.clone(),
            self.name_cn.clone(),
        );
        npc.character.level = self.level;
        npc.character.hp = self.hp;
        npc.character.hp_max = self.hp_max;
        npc.character.attack = self.attack;
        npc.character.defense = self.defense;
        npc.character.speed = self.speed;
        npc.dialogue_data = self.dialogues.clone();
        if let Some(ref room) = self.start_room {
            npc.set_room(room.clone());
        }
        npc
    }
}

/// NPC模板注册表
pub struct NpcTemplateRegistry {
    templates: HashMap<String, Arc<NpcTemplate>>,
}

impl NpcTemplateRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// 注册模板
    pub fn register(&mut self, template: NpcTemplate) {
        let id = template.id.clone();
        self.templates.insert(id, Arc::new(template));
    }

    /// 获取模板
    pub fn get(&self, id: &str) -> Option<Arc<NpcTemplate>> {
        self.templates.get(id).cloned()
    }

    /// 通过模板创建NPC
    pub fn create_npc(&self, template_id: &str) -> Option<Npc> {
        self.get(template_id).map(|t| t.instantiate())
    }

    /// 列出所有模板ID
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
}

impl Default for NpcTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局NPC模板注册表
pub static NPC_TEMPLATE_REGISTRY: tokio::sync::OnceCell<Arc<RwLock<NpcTemplateRegistry>>> = tokio::sync::OnceCell::const_new();

/// 初始化NPC模板注册表
pub async fn init_npc_templates() {
    let registry = Arc::new(RwLock::new(NpcTemplateRegistry::new()));
    let mut reg = registry.write().await;

    // 注册新手村村民
    reg.register(NpcTemplate::new(
        "npc/villager".to_string(),
        "villager".to_string(),
        "村民".to_string(),
    )
    .with_description("一个普通的村民，看起来很和善。\\n")
    .with_level(1)
    .with_stats(50, 5, 2)
    .with_start_room("xinshoucun/changetang".to_string())
    .with_dialogue(
        "你好".to_string(),
        "你好，欢迎来到新手村！".to_string(),
    )
    .with_dialogue(
        "任务".to_string(),
        "去村长那里看看吧，他可能有任务给你。".to_string(),
    ));

    // 注册新手村村长
    reg.register(NpcTemplate::new(
        "npc/village_head".to_string(),
        "village_head".to_string(),
        "村长".to_string(),
    )
    .with_description("一位慈祥的老人，负责管理新手村的事务。\\n")
    .with_level(10)
    .with_stats(200, 20, 10)
    .with_start_room("xinshoucun/changetang".to_string())
    .with_dialogue(
        "你好".to_string(),
        "年轻人，欢迎来到这个世界！".to_string(),
    )
    .with_dialogue(
        "历练".to_string(),
        "你可以去野外打怪升级，注意安全！".to_string(),
    ));

    // 注册神秘老人 (对话NPC)
    reg.register(NpcTemplate::new(
        "npc/mysterious_elder".to_string(),
        "mysterious_elder".to_string(),
        "神秘老人".to_string(),
    )
    .with_description("一位白发苍苍的老人，眼神深邃，似乎隐藏着什么秘密。\\n")
    .with_level(99)
    .with_stats(9999, 999, 999)
    .with_start_room("xinshoucun/changetang".to_string())
    .with_dialogue(
        "你好".to_string(),
        "年轻人，我看你骨骼惊奇，是练武的好材料...".to_string(),
    )
    .with_dialogue(
        "秘密".to_string(),
        "有些事情你现在不需要知道...".to_string(),
    ));

    // 注册可攻击的怪物
    reg.register(NpcTemplate::new(
        "mob/wolf".to_string(),
        "wolf".to_string(),
        "野狼".to_string(),
    )
    .with_description("一只凶猛的野狼，眼神凶狠。\\n")
    .with_level(3)
    .with_stats(80, 15, 5)
    .with_attackable(true));

    drop(reg);
    NPC_TEMPLATE_REGISTRY.set(registry).ok().unwrap();
}

/// 获取NPC模板注册表
pub async fn get_npc_registry() -> Arc<RwLock<NpcTemplateRegistry>> {
    NPC_TEMPLATE_REGISTRY.get()
        .expect("NPC template registry not initialized")
        .clone()
}

/// 通过模板ID创建NPC
pub async fn create_npc_from_template(template_id: &str) -> Option<Npc> {
    let registry = get_npc_registry().await;
    let reg = registry.read().await;
    reg.create_npc(template_id)
}
