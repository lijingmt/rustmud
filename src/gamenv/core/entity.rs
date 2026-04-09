// gamenv/core/entity.rs - 实体抽象
// 所有游戏对象（NPC、物品、玩家）都实现Entity trait

use serde::{Deserialize, Serialize};
use std::any::Any;

/// 实体类型枚举
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Npc,
    Item,
    Player,
    Room,
    Exit,
    Monster,
}

/// 实体基础trait - 所有游戏对象的基础
///
/// 注意：为了支持 dyn Entity，所有方法返回 String 而非 &str
pub trait Entity: Any + Send + Sync {
    /// 获取实体ID
    fn id(&self) -> String;

    /// 获取实体名称
    fn name(&self) -> String;

    /// 获取实体类型
    fn entity_type(&self) -> EntityType;

    /// 获取短描述
    fn short_desc(&self) -> String {
        self.name()
    }

    /// 获取长描述
    fn long_desc(&self) -> String {
        self.short_desc()
    }

    /// 转换为Any以便downcast
    fn as_any(&self) -> &dyn Any;
}

/// 可交互实体trait
pub trait Interactable: Entity {
    /// 获取可用的交互动作
    fn interactions(&self) -> Vec<Interaction>;

    /// 检查是否支持某个交互
    fn can_interact(&self, action: &str) -> bool {
        self.interactions().iter().any(|i| i.action == action)
    }

    /// 执行交互
    fn interact(&self, action: &str, context: &InteractionContext) -> InteractionResult;
}

/// 可战斗实体trait
pub trait Combatant: Entity {
    fn level(&self) -> i32;
    fn hp(&self) -> i32;
    fn max_hp(&self) -> i32;
    fn attack_power(&self) -> i32;
    fn defense(&self) -> i32;
    fn is_alive(&self) -> bool { self.hp() > 0 }
}

/// 可交易实体trait（商人、商店NPC）
pub trait Merchant: Entity {
    fn shop_id(&self) -> Option<String>;
    fn can_trade(&self) -> bool {
        self.shop_id().is_some()
    }
}

/// 交互动作
#[derive(Clone, Debug)]
pub struct Interaction {
    pub action: String,
    pub label: String,
    pub description: String,
    pub requires_combat: bool,
    pub style: ButtonStyle,
}

/// 交互上下文
#[derive(Clone, Debug)]
pub struct InteractionContext {
    pub player_id: String,
    pub room_id: String,
    pub timestamp: i64,
}

/// 交互结果
#[derive(Clone, Debug)]
pub enum InteractionResult {
    Success(String),
    Failure(String),
    Combat(String),
    Dialog(Vec<DialogOption>),
    Redirect(String),
}

/// 对话选项
#[derive(Clone, Debug)]
pub struct DialogOption {
    pub id: String,
    pub text: String,
    pub action: Option<String>,
}

/// 按钮样式
#[derive(Clone, Debug)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Danger,
    Warning,
    Info,
    Success,
}

impl ButtonStyle {
    pub fn as_str(&self) -> &str {
        match self {
            ButtonStyle::Primary => "btn-primary",
            ButtonStyle::Secondary => "btn-secondary",
            ButtonStyle::Danger => "btn-danger",
            ButtonStyle::Warning => "btn-warning",
            ButtonStyle::Info => "btn-info",
            ButtonStyle::Success => "btn-success",
        }
    }
}

/// 实体组件trait - 用于组件化设计
pub trait Component: Any + Send + Sync {
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// 实体容器 - 支持组件系统
pub struct EntityComponents {
    components: std::collections::HashMap<String, Box<dyn Component>>,
}

impl EntityComponents {
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
        }
    }

    pub fn add<C: Component + 'static>(&mut self, component: C) {
        self.components.insert(
            std::any::type_name::<C>().to_string(),
            Box::new(component),
        );
    }

    pub fn get<C: Component + 'static>(&self) -> Option<&C> {
        self.components
            .get(std::any::type_name::<C>())
            .and_then(|c| c.as_any().downcast_ref::<C>())
    }

    pub fn get_mut<C: Component + 'static>(&mut self) -> Option<&mut C> {
        self.components
            .get_mut(std::any::type_name::<C>())
            .and_then(|c| c.as_any_mut().downcast_mut::<C>())
    }
}

impl Default for EntityComponents {
    fn default() -> Self {
        Self::new()
    }
}

/// 实体过滤器
#[derive(Clone, Default)]
pub struct EntityFilter {
    pub entity_type: Option<EntityType>,
    pub name_contains: Option<String>,
    pub min_level: Option<i32>,
    pub max_level: Option<i32>,
    pub has_interaction: Option<String>,
}

impl EntityFilter {
    pub fn matches(&self, entity: &dyn Entity) -> bool {
        if let Some(et) = &self.entity_type {
            if entity.entity_type() != *et {
                return false;
            }
        }

        if let Some(pattern) = &self.name_contains {
            if !entity.name().contains(pattern) {
                return false;
            }
        }

        true
    }
}
