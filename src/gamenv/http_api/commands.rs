// gamenv/http_api/commands.rs - 命令注册和分发系统
// 允许模块化地添加新命令

use std::collections::HashMap;

/// 命令类别
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandCategory {
    Movement,     // 移动类
    Interaction,  // 交互类
    Combat,       // 战斗类
    Social,       // 社交类
    System,       // 系统类
    Info,         // 信息类
}

/// 命令元数据
#[derive(Clone)]
pub struct CommandMeta {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub category: CommandCategory,
}

impl CommandMeta {
    pub fn new(name: &str, description: &str, category: CommandCategory) -> Self {
        Self {
            name: name.to_string(),
            aliases: vec![],
            description: description.to_string(),
            category,
        }
    }

    pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// 命令注册表（只存储元数据，实际处理在execute_game_command中）
pub struct CommandRegistry {
    commands: HashMap<String, CommandMeta>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
        };
        registry.register_all();
        registry
    }

    /// 注册命令
    pub fn register(&mut self, meta: CommandMeta) {
        // 注册主命令名
        self.commands.insert(meta.name.clone(), meta.clone());

        // 注册别名
        for alias in &meta.aliases {
            self.commands.insert(alias.clone(), meta.clone());
        }
    }

    /// 检查命令是否存在
    pub fn exists(&self, cmd: &str) -> bool {
        self.commands.contains_key(cmd)
    }

    /// 获取命令元数据
    pub fn get(&self, cmd: &str) -> Option<&CommandMeta> {
        self.commands.get(cmd)
    }

    /// 按类别获取所有命令
    pub fn by_category(&self, category: CommandCategory) -> Vec<&CommandMeta> {
        let mut seen = std::collections::HashSet::new();
        let mut result = vec![];

        for (_name, meta) in &self.commands {
            if meta.category == category && !seen.contains(&meta.name) {
                seen.insert(meta.name.clone());
                result.push(meta);
            }
        }

        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// 获取所有主要命令（不含别名）
    pub fn all_commands(&self) -> Vec<&CommandMeta> {
        let mut seen = std::collections::HashSet::new();
        let mut result = vec![];

        for (_name, meta) in &self.commands {
            if !seen.contains(&meta.name) {
                seen.insert(meta.name.clone());
                result.push(meta);
            }
        }

        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// 注册所有内置命令
    fn register_all(&mut self) {
        // 移动命令
        self.register(CommandMeta::new("north", "向北移动", CommandCategory::Movement)
            .with_aliases(&["n"]));
        self.register(CommandMeta::new("south", "向南移动", CommandCategory::Movement)
            .with_aliases(&["s"]));
        self.register(CommandMeta::new("east", "向东移动", CommandCategory::Movement)
            .with_aliases(&["e"]));
        self.register(CommandMeta::new("west", "向西移动", CommandCategory::Movement)
            .with_aliases(&["w"]));
        self.register(CommandMeta::new("up", "向上移动", CommandCategory::Movement)
            .with_aliases(&["u"]));
        self.register(CommandMeta::new("down", "向下移动", CommandCategory::Movement)
            .with_aliases(&["d"]));

        // 交互命令
        self.register(CommandMeta::new("look", "查看周围环境或目标", CommandCategory::Interaction)
            .with_aliases(&["l"]));
        self.register(CommandMeta::new("talk", "与NPC对话", CommandCategory::Interaction));
        self.register(CommandMeta::new("ask", "选择对话选项", CommandCategory::Interaction));

        // 战斗命令
        self.register(CommandMeta::new("kill", "攻击怪物", CommandCategory::Combat)
            .with_aliases(&["attack"]));
        self.register(CommandMeta::new("pk", "发起PK挑战", CommandCategory::Combat));

        // 社交命令
        self.register(CommandMeta::new("say", "说话", CommandCategory::Social));
        self.register(CommandMeta::new("tell", "悄悄话", CommandCategory::Social));
        self.register(CommandMeta::new("who", "查看在线玩家", CommandCategory::Social));

        // 系统命令
        self.register(CommandMeta::new("help", "显示帮助", CommandCategory::System));
        self.register(CommandMeta::new("save", "保存进度", CommandCategory::System));

        // 信息命令
        self.register(CommandMeta::new("score", "查看状态", CommandCategory::Info));
        self.register(CommandMeta::new("inventory", "查看背包", CommandCategory::Info)
            .with_aliases(&["i", "inv"]));
        self.register(CommandMeta::new("equipment", "查看装备", CommandCategory::Info)
            .with_aliases(&["eq"]));
        self.register(CommandMeta::new("skills", "查看技能", CommandCategory::Info));
        self.register(CommandMeta::new("quest", "查看任务", CommandCategory::Info));
        self.register(CommandMeta::new("shop", "打开商店", CommandCategory::Info));
    }

    /// 生成帮助文本
    pub fn help_text(&self) -> String {
        let mut output = String::from("§H可用命令:§N\n");

        for category in &[
            CommandCategory::Movement,
            CommandCategory::Interaction,
            CommandCategory::Combat,
            CommandCategory::Info,
            CommandCategory::Social,
            CommandCategory::System,
        ] {
            let category_name = match category {
                CommandCategory::Movement => "【基础】",
                CommandCategory::Interaction => "【互动】",
                CommandCategory::Combat => "【战斗】",
                CommandCategory::Info => "【角色】",
                CommandCategory::Social => "【社交】",
                CommandCategory::System => "【系统】",
            };

            let commands = self.by_category(*category);
            if !commands.is_empty() {
                output.push_str(&format!("§C{}§N\n", category_name));
                for cmd in commands {
                    let aliases = if cmd.aliases.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", cmd.aliases.join(", "))
                    };
                    output.push_str(&format!("  {}{} - {}\n", cmd.name, aliases, cmd.description));
                }
                output.push('\n');
            }
        }

        output
    }
}

/// 全局命令注册表
lazy_static::lazy_static! {
    pub static ref COMMAND_REGISTRY: std::sync::Mutex<CommandRegistry> = {
        std::sync::Mutex::new(CommandRegistry::new())
    };
}

/// 便捷函数：注册新命令
pub fn register_command(meta: CommandMeta) {
    COMMAND_REGISTRY.lock().unwrap().register(meta);
}

/// 便捷函数：检查命令是否存在
pub fn command_exists(cmd: &str) -> bool {
    COMMAND_REGISTRY.lock().unwrap().exists(cmd)
}
