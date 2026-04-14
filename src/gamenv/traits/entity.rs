// gamenv/traits/entity.rs - 实体包装器
// 将特性组合系统集成到游戏对象中
// 对应 txpike9 中的对象动态特性系统

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock as TokioRwLock;
use crate::gamenv::world::{Npc, Room};
use crate::gamenv::traits::composition::*;

/// 实体类型枚举
#[derive(Clone, Debug, PartialEq)]
pub enum EntityType {
    Npc,
    Player,
    Item,
    Room,
}

/// 游戏实体包装器 - 基础特性
pub struct GameEntity {
    /// 实体ID
    pub id: String,
    /// 实体类型
    pub entity_type: EntityType,
    /// 动态特性容器
    pub features: FeatureContainer,
}

impl GameEntity {
    /// 创建新的游戏实体
    pub fn new(id: String, entity_type: EntityType) -> Self {
        Self {
            id,
            entity_type,
            features: FeatureContainer::new(),
        }
    }

    /// 从NPC创建实体（自动添加战斗特性）
    pub fn from_npc(npc: &Npc) -> Self {
        let mut entity = Self::new(npc.id.clone(), EntityType::Npc);

        // 添加战斗特性
        entity.features.add(FightFeature::new(
            npc.hp,
            npc.hp_max,
            npc.attack,
            npc.defense,
        ));

        // 如果有商店，添加商店特性
        if let Some(shop_id) = &npc.shop {
            entity.features.add(ShopFeature::new(shop_id.clone()));
        }

        // 如果有对话，添加对话特性
        if !npc.dialogs.is_empty() {
            entity.features.add(DialogFeature::new(format!("dialog_{}", npc.id)));
        }

        // 添加移动特性（NPC默认可移动）
        entity.features.add(MovableFeature::new());

        entity
    }

    /// 检查实体是否有某个特性
    pub fn has_feature<F: Feature + 'static>(&self) -> bool {
        self.features.has::<F>()
    }

    /// 获取实体特性
    pub fn get_feature<F: Feature + 'static>(&self) -> Option<&F> {
        self.features.get::<F>()
    }

    /// 获取可变特性
    pub fn get_feature_mut<F: Feature + 'static>(&mut self) -> Option<&mut F> {
        self.features.get_mut::<F>()
    }

    /// 添加特性
    pub fn add_feature<F: Feature + 'static + Clone>(&mut self, feature: F) {
        self.features.add(feature);
    }

    /// 移除特性
    pub fn remove_feature<F: Feature + 'static>(&mut self) -> bool {
        self.features.remove::<F>(&self.id)
    }

    /// 是否可战斗
    pub fn can_fight(&self) -> bool {
        self.has_feature::<FightFeature>()
    }

    /// 是否可移动
    pub fn can_move(&self) -> bool {
        if let Some(movable) = self.get_feature::<MovableFeature>() {
            movable.can_move
        } else {
            false
        }
    }

    /// 是否有商店
    pub fn has_shop(&self) -> bool {
        self.has_feature::<ShopFeature>()
    }

    /// 是否可对话
    pub fn can_talk(&self) -> bool {
        self.has_feature::<DialogFeature>()
    }

    /// 获取HP百分比（如果有战斗特性）
    pub fn hp_percent(&self) -> i32 {
        self.get_feature::<FightFeature>()
            .map(|f| f.hp_percent())
            .unwrap_or(0)
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.get_feature::<FightFeature>()
            .map(|f| f.is_alive())
            .unwrap_or(true)
    }
}

/// 实体管理器 - 管理所有运行时实体
pub struct EntityManager {
    entities: TokioRwLock<HashMap<String, Arc<TokioRwLock<GameEntity>>>>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: TokioRwLock::new(HashMap::new()),
        }
    }

    /// 获取或创建实体
    pub async fn get_or_create_npc(&self, npc: &Npc) -> Arc<TokioRwLock<GameEntity>> {
        let entities = self.entities.read().await;

        if let Some(entity) = entities.get(&npc.id) {
            entity.clone()
        } else {
            drop(entities);
            let mut entities = self.entities.write().await;

            // 再次检查（double-check locking）
            if let Some(entity) = entities.get(&npc.id) {
                entity.clone()
            } else {
                let entity = Arc::new(TokioRwLock::new(GameEntity::from_npc(npc)));
                entities.insert(npc.id.clone(), entity.clone());
                entity
            }
        }
    }

    /// 移除实体
    pub async fn remove(&self, id: &str) -> bool {
        let mut entities = self.entities.write().await;
        entities.remove(id).is_some()
    }

    /// 获取实体
    pub async fn get(&self, id: &str) -> Option<Arc<TokioRwLock<GameEntity>>> {
        let entities = self.entities.read().await;
        entities.get(id).cloned()
    }

    /// 清空所有实体
    pub async fn clear(&self) {
        let mut entities = self.entities.write().await;
        entities.clear();
    }

    /// 获取实体数量
    pub async fn count(&self) -> usize {
        let entities = self.entities.read().await;
        entities.len()
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局实体管理器
static ENTITY_MANAGER: once_cell::sync::Lazy<Arc<EntityManager>> =
    once_cell::sync::Lazy::new(|| Arc::new(EntityManager::new()));

/// 获取全局实体管理器
pub fn get_entity_manager() -> Arc<EntityManager> {
    ENTITY_MANAGER.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_entity_creation() {
        let npc = Npc {
            id: "npc_001".to_string(),
            name: "测试NPC".to_string(),
            short: "测试".to_string(),
            long: "这是一个测试NPC".to_string(),
            level: 10,
            hp: 100,
            hp_max: 100,
            mp: 50,
            mp_max: 50,
            attack: 15,
            defense: 10,
            exp: 100,
            gold: 50,
            behavior: crate::gamenv::world::npc::NpcBehavior::Passive,
            dialogs: vec![],
            shop: None,
            loot: vec![],
        };

        let entity = GameEntity::from_npc(&npc);

        assert!(entity.can_fight());
        assert!(entity.can_move());
        assert_eq!(entity.hp_percent(), 100);
        assert!(entity.is_alive());
    }

    #[tokio::test]
    async fn test_entity_manager() {
        let npc = Npc {
            id: "npc_test".to_string(),
            name: "测试".to_string(),
            short: "测试".to_string(),
            long: "测试".to_string(),
            level: 1,
            hp: 50,
            hp_max: 50,
            mp: 10,
            mp_max: 10,
            attack: 5,
            defense: 3,
            exp: 10,
            gold: 5,
            behavior: crate::gamenv::world::npc::NpcBehavior::Passive,
            dialogs: vec![],
            shop: Some("shop_001".to_string()),
            loot: vec![],
        };

        let mgr = get_entity_manager();
        let entity = mgr.get_or_create_npc(&npc).await;

        // 验证特性
        let entity_read = entity.read().await;
        assert!(entity_read.can_fight());
        assert!(entity_read.has_shop());
    }
}
