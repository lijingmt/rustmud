// gamenv/efuns.rs - 核心MUD操作
// 对应 txpike9/pikenv/efuns.pike
//
// 实现MUD的基本对象操作:
// - move_object: 移动对象
// - environment: 获取对象所在环境
// - all_inventory: 获取容器内的所有对象
// - present: 查找对象
// - destruct: 销毁对象
// - clone: 克隆对象

use crate::core::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 对象类型
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Player,
    Npc,
    Item,
    Room,
}

/// 对象引用 (弱引用，避免循环引用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjRef {
    pub id: String,
    pub obj_type: String,
}

impl ObjRef {
    pub fn new(id: String, obj_type: String) -> Self {
        Self { id, obj_type }
    }
}

/// 世界状态 - 跟踪所有对象的位置和关系
///
/// 对应 txpike9 efuns.pike 中的:
/// - inv_map: mapping(object:array(object)) - 容器内的对象
/// - env_map: mapping(object:object) - 对象所在的环境
pub struct WorldState {
    /// 环境映射: 对象ID -> 所在容器的对象ID
    env_map: HashMap<String, String>,
    /// 库存映射: 容器ID -> 包含的对象ID列表
    inv_map: HashMap<String, Vec<String>>,
    /// 对象类型映射
    obj_types: HashMap<String, ObjectType>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            env_map: HashMap::new(),
            inv_map: HashMap::new(),
            obj_types: HashMap::new(),
        }
    }

    /// 注册对象类型
    pub fn register_object_type(&mut self, id: String, obj_type: ObjectType) {
        self.obj_types.insert(id, obj_type);
    }

    /// 移动对象到新环境
    ///
    /// 对应 txpike9 的 move_object(dest)
    pub fn move_object(&mut self, obj_id: &str, dest_id: &str) -> Result<(), String> {
        // 1. 从旧环境中移除
        if let Some(old_env) = self.env_map.get(obj_id) {
            if let Some(inv) = self.inv_map.get_mut(old_env) {
                inv.retain(|id| id != obj_id);
                if inv.is_empty() {
                    self.inv_map.remove(old_env);
                }
            }
        }

        // 2. 添加到新环境
        self.env_map.insert(obj_id.to_string(), dest_id.to_string());
        self.inv_map
            .entry(dest_id.to_string())
            .or_insert_with(Vec::new)
            .push(obj_id.to_string());

        Ok(())
    }

    /// 移除对象 (从当前位置移除，但不销毁)
    pub fn remove_from_environment(&mut self, obj_id: &str) {
        if let Some(old_env) = self.env_map.remove(obj_id) {
            if let Some(inv) = self.inv_map.get_mut(&old_env) {
                inv.retain(|id| id != obj_id);
                if inv.is_empty() {
                    self.inv_map.remove(&old_env);
                }
            }
        }
    }

    /// 完全销毁对象
    ///
    /// 对应 txpike9 的 destruct()
    pub fn destruct(&mut self, obj_id: &str) {
        // 1. 先从环境中移除
        self.remove_from_environment(obj_id);

        // 2. 清理该对象内的所有物品
        if let Some(inv) = self.inv_map.remove(obj_id) {
            for contained_id in inv {
                // 递归销毁容器内的对象，或将它们移动到当前环境
                self.destruct(&contained_id);
            }
        }

        // 3. 清理类型信息
        self.obj_types.remove(obj_id);
    }

    /// 获取对象所在环境
    ///
    /// 对应 txpike9 的 environment()
    pub fn environment(&self, obj_id: &str) -> Option<&str> {
        self.env_map.get(obj_id).map(|s| s.as_str())
    }

    /// 获取容器内的所有对象
    ///
    /// 对应 txpike9 的 all_inventory()
    pub fn all_inventory(&self, container_id: &str) -> Vec<&str> {
        self.inv_map
            .get(container_id)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// 在指定位置查找对象
    ///
    /// 对应 txpike9 的 present()
    pub fn present(&self, obj_id: &str, container_id: &str) -> bool {
        if let Some(inv) = self.inv_map.get(container_id) {
            inv.iter().any(|id| id == obj_id)
        } else {
            false
        }
    }

    /// 按类型查找对象
    pub fn find_by_type(&self, container_id: &str, obj_type: ObjectType) -> Vec<String> {
        self.inv_map
            .get(container_id)
            .map(|inv| {
                inv.iter()
                    .filter(|id| self.obj_types.get(*id) == Some(&obj_type))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取对象类型
    pub fn get_object_type(&self, obj_id: &str) -> Option<&ObjectType> {
        self.obj_types.get(obj_id)
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局世界状态
pub static WORLD_STATE: tokio::sync::OnceCell<Arc<RwLock<WorldState>>> =
    tokio::sync::OnceCell::const_new();

/// 初始化世界状态
pub async fn init_world_state() {
    WORLD_STATE
        .set(Arc::new(RwLock::new(WorldState::new())))
        .ok();
}

/// 获取世界状态
pub async fn get_world_state() -> Arc<RwLock<WorldState>> {
    WORLD_STATE
        .get()
        .expect("World state not initialized")
        .clone()
}

// ==================== Efun 函数 ====================

/// 移动对象到目标位置
///
/// 对应 txpike9: move_object(dest)
pub async fn move_object(obj_id: &str, dest_id: &str) -> Result<(), String> {
    let state = get_world_state().await;
    let mut state = state.write().await;
    state.move_object(obj_id, dest_id)
}

/// 获取对象所在环境
///
/// 对应 txpike9: environment()
pub async fn environment(obj_id: &str) -> Option<String> {
    let state = get_world_state().await;
    let state = state.read().await;
    state.environment(obj_id).map(|s| s.to_string())
}

/// 获取容器内的所有对象
///
/// 对应 txpike9: all_inventory()
pub async fn all_inventory(container_id: &str) -> Vec<String> {
    let state = get_world_state().await;
    let state = state.read().await;
    state.all_inventory(container_id)
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// 检查对象是否在指定位置
///
/// 对应 txpike9: present()
pub async fn present(obj_id: &str, container_id: &str) -> bool {
    let state = get_world_state().await;
    let state = state.read().await;
    state.present(obj_id, container_id)
}

/// 销毁对象
///
/// 对应 txpike9: destruct()
pub async fn destruct(obj_id: &str) {
    let state = get_world_state().await;
    let mut state = state.write().await;
    state.destruct(obj_id);
}

/// 注册对象到世界状态
pub async fn register_object(obj_id: String, obj_type: ObjectType, env_id: Option<String>) {
    let state = get_world_state().await;
    let mut state = state.write().await;

    state.register_object_type(obj_id.clone(), obj_type);

    if let Some(env) = env_id {
        state.move_object(&obj_id, &env).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_move_object() {
        init_world_state().await;

        // 注册测试对象
        register_object("room1".to_string(), ObjectType::Room, None).await;
        register_object("room2".to_string(), ObjectType::Room, None).await;
        register_object("item1".to_string(), ObjectType::Item, Some("room1".to_string())).await;

        // 检查初始位置
        assert_eq!(environment("item1").await, Some("room1".to_string()));

        // 移动到新位置
        move_object("item1", "room2").await.unwrap();

        // 验证移动成功
        assert_eq!(environment("item1").await, Some("room2".to_string()));

        // 检查库存
        let inv1 = all_inventory("room1").await;
        let inv2 = all_inventory("room2").await;
        assert!(inv1.is_empty());
        assert_eq!(inv2.len(), 1);
        assert_eq!(inv2[0], "item1");
    }

    #[tokio::test]
    async fn test_destruct() {
        init_world_state().await;

        register_object("room1".to_string(), ObjectType::Room, None).await;
        register_object("item1".to_string(), ObjectType::Item, Some("room1".to_string())).await;

        // 销毁对象
        destruct("item1").await;

        // 验证对象已消失
        assert_eq!(environment("item1").await, None);
        assert!(all_inventory("room1").await.is_empty());
    }
}
