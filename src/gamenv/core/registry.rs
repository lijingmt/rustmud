// gamenv/core/registry.rs - 通用注册表抽象
// 所有游戏对象的注册都基于这个抽象

use std::collections::HashMap;
use std::sync::Arc;
use std::any::{Any, TypeId};

/// 通用注册表trait
pub trait Registry<K, V>: Send + Sync {
    fn register(&mut self, key: K, value: V) -> Option<V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn contains_key(&self, key: &K) -> bool;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn keys(&self) -> Vec<&K>;
    fn values(&self) -> Vec<&V>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

/// 基础HashMap实现的注册表
#[derive(Default)]
pub struct HashMapRegistry<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    inner: HashMap<K, V>,
}

impl<K, V> Registry<K, V> for HashMapRegistry<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn register(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }

    fn keys(&self) -> Vec<&K> {
        self.inner.keys().collect()
    }

    fn values(&self) -> Vec<&V> {
        self.inner.values().collect()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> HashMapRegistry<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

/// 多类型注册表 - 可以存储不同类型的值
pub struct TypedRegistry {
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for TypedRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedRegistry {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, value: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.data.get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data.get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }
}

/// 可观测的注册表 - 支持变更通知
pub trait Observable {
    type Event;
    fn subscribe(&self) -> Box<dyn Iterator<Item = Self::Event>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_map_registry() {
        let mut registry = HashMapRegistry::new();
        registry.register("cmd1".to_string(), "handler1".to_string());

        assert!(registry.contains_key(&"cmd1".to_string()));
        assert_eq!(registry.get(&"cmd1".to_string()), Some(&"handler1".to_string()));
    }

    #[test]
    fn test_typed_registry() {
        let mut registry = TypedRegistry::new();
        registry.insert(42i32);
        registry.insert("hello".to_string());

        assert_eq!(registry.get::<i32>(), Some(&42));
        assert_eq!(registry.get::<String>(), Some(&"hello".to_string()));
    }
}
