// core/object.rs - 对象系统
// 对应 Pike 的 object 类型，实现 save_object/restore_object

use crate::core::{ObjectId, Value, Mapping, MudError, Result, GObject};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// 对象基类 (对应 Pike 的 object 类型)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInner {
    pub id: ObjectId,
    pub name: String,
    pub program_path: String,
    pub variables: Mapping,
    pub created: i64,
    pub last_modified: i64,
}

impl ObjectInner {
    /// 创建新对象
    pub fn new(name: String, program_path: String) -> Self {
        Self {
            id: ObjectId::new(),
            name,
            program_path,
            variables: Mapping::new(),
            created: chrono::Utc::now().timestamp(),
            last_modified: chrono::Utc::now().timestamp(),
        }
    }

    /// 对应 Pike 的 save_object() - 保存对象状态到文件
    pub fn save_object(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, self)
            .map_err(|e| MudError::SerializationError(e.to_string()))?;
        tracing::debug!("Saved object {} to {}", self.name, path);
        Ok(())
    }

    /// 对应 Pike 的 restore_object() - 从文件恢复对象状态
    pub fn restore_object(&mut self, path: &str) -> Result<()> {
        if !Path::new(path).exists() {
            return Err(MudError::ObjectNotFound(path.to_string()));
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let restored: ObjectInner = bincode::deserialize_from(reader)
            .map_err(|e| MudError::SerializationError(e.to_string()))?;

        self.variables = restored.variables;
        self.last_modified = chrono::Utc::now().timestamp();
        tracing::debug!("Restored object {} from {}", self.name, path);
        Ok(())
    }

    /// 设置变量值
    pub fn set_variable(&mut self, key: &str, value: Value) {
        self.variables.insert(key, value);
        self.last_modified = chrono::Utc::now().timestamp();
    }

    /// 获取变量值
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }

    /// 删除变量
    pub fn delete_variable(&mut self, key: &str) -> Option<Value> {
        self.last_modified = chrono::Utc::now().timestamp();
        self.variables.delete(key)
    }
}

/// Extension trait for GObject (Arc<RwLock<ObjectInner>>)
pub trait GObjectExt {
    /// 获取对象 ID
    async fn id(&self) -> ObjectId;

    /// 保存对象
    async fn save_object(&self, path: String) -> Result<()>;

    /// 恢复对象
    async fn restore_object(&self, path: String) -> Result<()>;

    /// 调用对象方法 (对应 Pike 的 ob->method())
    async fn call_method(&self, method: &str, args: Vec<Value>) -> Result<Value>;
}

impl GObjectExt for GObject {
    /// 获取对象 ID
    async fn id(&self) -> ObjectId {
        self.read().await.id
    }

    /// 保存对象
    async fn save_object(&self, path: String) -> Result<()> {
        let inner = self.read().await;
        inner.save_object(&path)
    }

    /// 恢复对象
    async fn restore_object(&self, path: String) -> Result<()> {
        let mut inner = self.write().await;
        inner.restore_object(&path)
    }

    /// 调用对象方法 (对应 Pike 的 ob->method())
    async fn call_method(&self, method: &str, args: Vec<Value>) -> Result<Value> {
        // 这里会通过程序系统动态调用方法
        // 简化版本：只记录调用
        tracing::debug!("Calling method {} on object {:?}", method, self.read().await.id);
        Ok(Value::Void)
    }
}

/// 对象管理器 (对应 efuns.pike 中的对象管理)
pub struct ObjectManager {
    objects: dashmap::DashMap<ObjectId, GObject>,
    by_name: dashmap::DashMap<String, ObjectId>,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            objects: dashmap::DashMap::new(),
            by_name: dashmap::DashMap::new(),
        }
    }

    /// 注册对象
    pub fn register(&self, obj: GObject) {
        let id = {
            let inner = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(obj.read())
            });
            inner.id
        };
        let name = {
            let inner = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(obj.read())
            });
            inner.name.clone()
        };
        self.objects.insert(id, obj.clone());
        self.by_name.insert(name, id);
    }

    /// 查找对象 (对应 Pike 的 load_object / find_object)
    pub fn find(&self, id: ObjectId) -> Option<GObject> {
        self.objects.get(&id).map(|v| v.clone())
    }

    /// 按名称查找对象
    pub fn find_by_name(&self, name: &str) -> Option<GObject> {
        self.by_name.get(name).and_then(|id| self.objects.get(&id).map(|v| v.clone()))
    }

    /// 删除对象 (对应 Pike 的 destruct)
    pub fn destruct(&self, id: ObjectId) -> bool {
        // 首先获取对象名称
        let name = self.objects.get(&id).and_then(|obj| {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let inner = obj.read().await;
                    Some(inner.name.clone())
                })
            })
        });

        // 删除对象
        let removed = self.objects.remove(&id).is_some();

        // 删除名称映射
        if let Some(name) = name {
            self.by_name.remove(&name);
        }

        removed
    }

    /// 获取所有对象数量
    pub fn count(&self) -> usize {
        self.objects.len()
    }
}

impl Default for ObjectManager {
    fn default() -> Self {
        Self::new()
    }
}

// 对应 Pike 的 destruct()
pub async fn destruct(obj: &GObject) -> Result<()> {
    use crate::core::object::GObjectExt;
    // 清理对象资源
    tracing::debug!("Destructing object {:?}", obj.id().await);
    Ok(())
}
