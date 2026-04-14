// gamenv/traits/composition.rs - 特性组合系统
// 对应 txpike9 的 inherit 多继承机制
// 使用 Rust 实现动态特性组合

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use async_trait::async_trait;
use std::sync::Arc;

/// 特性 trait - 所有动态特性都需要实现此 trait
///
/// 类似 txpike9 中的 inherit 机制，但使用运行时组合而非编译时继承
pub trait Feature: Any + Send + Sync + fmt::Debug {
    /// 特性名称（唯一标识符）
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// 特性描述
    fn description(&self) -> &str {
        ""
    }

    /// 转换为 Any 以支持 downcasting
    fn as_any(&self) -> &dyn Any;

    /// 转换为 Any (mutable) 以支持 downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 特性初始化回调
    fn on_add(&mut self, _entity_id: &str) {}

    /// 特性移除回调
    fn on_remove(&mut self, _entity_id: &str) {}

    /// 克隆特性
    fn clone_box(&self) -> Box<dyn Feature>;
}

/// 特性容器 - 管理对象的所有动态特性
///
/// 类似 txpike9 中的对象通过 inherit 获得多个特性
/// 这里通过 FeatureContainer 在运行时组合多个特性
pub struct FeatureContainer {
    features: HashMap<TypeId, Box<dyn Feature>>,
}

impl Default for FeatureContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureContainer {
    /// 创建新的特性容器
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }

    /// 添加特性
    ///
    /// 类似 txpike9 中的 `inherit FEATURE_NAME;`
    pub fn add<F: Feature + 'static + Clone>(&mut self, feature: F) {
        let type_id = TypeId::of::<F>();
        self.features.insert(type_id, Box::new(feature));
    }

    /// 添加动态特性（通过 Box<dyn Feature>）
    pub fn add_box(&mut self, feature: Box<dyn Feature>) {
        let type_id = (*feature).type_id();
        self.features.insert(type_id, feature);
    }

    /// 移除特性
    pub fn remove<F: Feature + 'static>(&mut self, entity_id: &str) -> bool {
        let type_id = TypeId::of::<F>();

        // 调用移除回调
        if let Some(feature) = self.features.get_mut(&type_id) {
            feature.as_any_mut();
            feature.on_remove(entity_id);
        }

        self.features.remove(&type_id).is_some()
    }

    /// 检查是否有某个特性
    ///
    /// 类似 txpike9 中的 `if (ob->is("npc"))`
    pub fn has<F: Feature + 'static>(&self) -> bool {
        self.features.contains_key(&TypeId::of::<F>())
    }

    /// 检查是否有某个特性（通过 TypeId）
    pub fn has_type_id(&self, type_id: TypeId) -> bool {
        self.features.contains_key(&type_id)
    }

    /// 获取特性（不可变引用）
    pub fn get<F: Feature + 'static>(&self) -> Option<&F> {
        let type_id = TypeId::of::<F>();
        self.features.get(&type_id)
            .and_then(|f| f.as_any().downcast_ref::<F>())
    }

    /// 获取特性（可变引用）
    pub fn get_mut<F: Feature + 'static>(&mut self) -> Option<&mut F> {
        let type_id = TypeId::of::<F>();
        if let Some(feature) = self.features.get_mut(&type_id) {
            feature.as_any_mut().downcast_mut::<F>()
        } else {
            None
        }
    }

    /// 列出所有特性名称
    pub fn list_features(&self) -> Vec<String> {
        self.features.values()
            .map(|f| f.name().to_string())
            .collect()
    }

    /// 获取特性数量
    pub fn count(&self) -> usize {
        self.features.len()
    }

    /// 清空所有特性
    pub fn clear(&mut self) {
        self.features.clear();
    }
}

impl fmt::Debug for FeatureContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FeatureContainer")
            .field("feature_count", &self.features.len())
            .field("features", &self.list_features())
            .finish()
    }
}

// ============================================================================
// 具体特性实现 - 对应 txpike9 中的各个 inherit 特性
// ============================================================================

/// 战斗特性 - 对应 txpike9 中的 COMBAT 特性
#[derive(Clone, Debug)]
pub struct FightFeature {
    pub hp: i32,
    pub hp_max: i32,
    pub attack: i32,
    pub defense: i32,
    pub level: u32,
    pub exp: u64,
}

impl FightFeature {
    pub fn new(hp: i32, hp_max: i32, attack: i32, defense: i32) -> Self {
        Self {
            hp,
            hp_max,
            attack,
            defense,
            level: 1,
            exp: 0,
        }
    }

    pub fn hp_percent(&self) -> i32 {
        if self.hp_max == 0 { return 0; }
        (self.hp * 100 / self.hp_max).max(0).min(100)
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

impl Feature for FightFeature {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn description(&self) -> &str {
        "战斗特性 - 可进行战斗"
    }

    fn clone_box(&self) -> Box<dyn Feature> {
        Box::new(self.clone())
    }
}

/// 背包特性 - 对应 txpike9 中的 STORAGE 特性
#[derive(Clone, Debug)]
pub struct InventoryFeature {
    pub items: Vec<String>,
    pub capacity: usize,
    pub encumbrance: i32,
    pub max_encumbrance: i32,
}

impl InventoryFeature {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            capacity,
            encumbrance: 0,
            max_encumbrance: 1000,
        }
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    pub fn add_item(&mut self, item_id: String) -> Result<(), String> {
        if self.is_full() {
            return Err("背包已满".to_string());
        }
        self.items.push(item_id);
        Ok(())
    }

    pub fn remove_item(&mut self, item_id: &str) -> Result<String, String> {
        if let Some(pos) = self.items.iter().position(|x| x == item_id) {
            Ok(self.items.remove(pos))
        } else {
            Err("物品不存在".to_string())
        }
    }

    pub fn list_items(&self) -> String {
        if self.items.is_empty() {
            return "背包是空的。".to_string();
        }

        let mut output = "你的背包：\n".to_string();
        for (i, item_id) in self.items.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, item_id));
        }
        output
    }
}

impl Feature for InventoryFeature {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn description(&self) -> &str {
        "背包特性 - 可携带物品"
    }

    fn clone_box(&self) -> Box<dyn Feature> {
        Box::new(self.clone())
    }
}

/// 对话特性 - 对应 txpike9 中的 CONVERSATION 特性
#[derive(Clone, Debug)]
pub struct DialogFeature {
    pub topics: Vec<String>,
    pub dialog_id: String,
}

impl DialogFeature {
    pub fn new(dialog_id: String) -> Self {
        Self {
            topics: vec![
                "你好".to_string(),
                "任务".to_string(),
                "再见".to_string(),
            ],
            dialog_id,
        }
    }

    pub fn has_topic(&self, topic: &str) -> bool {
        self.topics.iter().any(|t| t.contains(topic))
    }
}

impl Feature for DialogFeature {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn description(&self) -> &str {
        "对话特性 - 可进行交互对话"
    }

    fn clone_box(&self) -> Box<dyn Feature> {
        Box::new(self.clone())
    }
}

/// 商店特性 - 对应 txpike9 中的 SHOP 特性
#[derive(Clone, Debug)]
pub struct ShopFeature {
    pub shop_id: String,
    pub buy_rate: f32,
    pub sell_rate: f32,
}

impl ShopFeature {
    pub fn new(shop_id: String) -> Self {
        Self {
            shop_id,
            buy_rate: 1.0,
            sell_rate: 0.5,
        }
    }
}

impl Feature for ShopFeature {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn description(&self) -> &str {
        "商店特性 - 可进行交易"
    }

    fn clone_box(&self) -> Box<dyn Feature> {
        Box::new(self.clone())
    }
}

/// 移动特性 - 对应 txpike9 中的 MOVABLE 特性
#[derive(Clone, Debug)]
pub struct MovableFeature {
    pub can_move: bool,
    pub speed: u32,
}

impl MovableFeature {
    pub fn new() -> Self {
        Self {
            can_move: true,
            speed: 100,
        }
    }

    pub fn is_immobile(&self) -> bool {
        !self.can_move
    }
}

impl Feature for MovableFeature {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn description(&self) -> &str {
        "移动特性 - 可在地图中移动"
    }

    fn clone_box(&self) -> Box<dyn Feature> {
        Box::new(self.clone())
    }
}

// ============================================================================
// 辅助宏 - 简化特性使用
// ============================================================================

/// 宏：创建带特性的对象
///
/// 类似 txpike9 中的:
/// ```pike
/// inherit NPC;
/// inherit COMBAT;
/// inherit CONVERSATION;
/// ```
#[macro_export]
macro_rules! compose_features {
    ($container:expr, [$($feature:expr),* $(,)?]) => {
        {
            use $crate::gamenv::traits::composition::Feature;
            $container.add($feature);
        }
    };
}

/// 检查对象是否有某个特性的宏
///
/// 类似 txpike9 中的: `if (ob->is("npc"))`
#[macro_export]
macro_rules! has_feature {
    ($container:expr, $feature:ty) => {
        $container.has::<$feature>()
    };
}

/// 获取某个特性的宏
#[macro_export]
macro_rules! get_feature {
    ($container:expr, $feature:ty) => {
        $container.get::<$feature>()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_container() {
        let mut container = FeatureContainer::new();

        // 添加战斗特性
        container.add(FightFeature::new(100, 100, 10, 5));

        // 检查特性存在
        assert!(container.has::<FightFeature>());

        // 获取特性
        if let Some(fight) = container.get::<FightFeature>() {
            assert_eq!(fight.hp, 100);
            assert_eq!(fight.attack, 10);
        }

        // 移除特性
        container.remove::<FightFeature>("test_id");
        assert!(!container.has::<FightFeature>());
    }

    #[test]
    fn test_multiple_features() {
        let mut container = FeatureContainer::new();

        // 组合多个特性 - 类似 txpike9 的多继承
        container.add(FightFeature::new(100, 100, 10, 5));
        container.add(InventoryFeature::new(20));
        container.add(DialogFeature::new("dialog_001".to_string()));
        container.add(MovableFeature::new());

        // 验证所有特性都存在
        assert!(container.has::<FightFeature>());
        assert!(container.has::<InventoryFeature>());
        assert!(container.has::<DialogFeature>());
        assert!(container.has::<MovableFeature>());

        // 列出特性
        let features = container.list_features();
        assert_eq!(features.len(), 4);
    }

    #[test]
    fn test_inventory_operations() {
        let mut inv = InventoryFeature::new(5);

        // 添加物品
        assert!(inv.add_item("item_001".to_string()).is_ok());
        assert!(inv.add_item("item_002".to_string()).is_ok());
        assert_eq!(inv.items.len(), 2);

        // 背包满 - 添加 3,4,5 会成功，6 会失败
        for i in 3..=6 {
            inv.add_item(format!("item_{:03}", i)).ok();
        }
        // 最终有 5 个物品 (001, 002, 003, 004, 005)，006 添加失败
        assert_eq!(inv.items.len(), 5);
        assert!(inv.add_item("item_007".to_string()).is_err());

        // 列出物品
        let list = inv.list_items();
        assert!(list.contains("item_001"));

        // 移除物品
        assert!(inv.remove_item("item_001").is_ok());
        assert_eq!(inv.items.len(), 4);  // 移除后剩 4 个
    }

    #[test]
    fn test_fight_feature() {
        let fight = FightFeature::new(100, 100, 15, 5);

        assert_eq!(fight.hp, 100);
        assert_eq!(fight.hp_percent(), 100);
        assert!(fight.is_alive());

        // 受伤测试
        let fight = FightFeature::new(50, 100, 10, 5);
        assert_eq!(fight.hp_percent(), 50);
    }

    #[test]
    fn test_dialog_feature() {
        let dialog = DialogFeature::new("test_dialog".to_string());

        assert!(dialog.has_topic("你好"));
        assert!(dialog.has_topic("任务"));
        assert!(!dialog.has_topic("不存在"));
    }
}
