// gamenv/http_api/command_queue.rs - 命令队列系统
// 对应 txpike9/gamenv/single/daemons/http_api/command_queue.pike

use crate::core::*;
use crate::gamenv::http_api::virtual_conn::VirtualConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot};

/// 命令队列项
struct CommandItem {
    command: String,
    response_tx: oneshot::Sender<Result<String, String>>,
}

/// 用户命令队列
struct UserQueue {
    userid: String,
    pending: Vec<CommandItem>,
    is_processing: bool,
}

impl UserQueue {
    fn new(userid: String) -> Self {
        Self {
            userid,
            pending: Vec::new(),
            is_processing: false,
        }
    }
}

/// 命令队列管理器
pub struct CommandQueue {
    queues: HashMap<String, UserQueue>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }

    /// 入队命令并等待结果 (对应 enqueue_and_wait)
    pub async fn enqueue_and_wait(
        &mut self,
        userid: String,
        command: String,
        _vconn: VirtualConnection,
    ) -> Result<String, String> {
        let (tx, rx) = oneshot::channel();

        // 获取或创建用户队列
        let queue = self.queues.entry(userid.clone()).or_insert_with(|| {
            UserQueue::new(userid)
        });

        // 添加命令到队列
        queue.pending.push(CommandItem {
            command,
            response_tx: tx,
        });

        // 如果没有正在处理，开始处理
        if !queue.is_processing {
            queue.is_processing = true;
            // TODO: 启动处理任务
        }

        // 等待结果
        match rx.await {
            Ok(result) => result,
            Err(_) => Err("Command cancelled".to_string()),
        }
    }

    /// 处理队列中的下一个命令
    pub async fn process_next(&mut self, userid: &str) -> Option<CommandItem> {
        if let Some(queue) = self.queues.get_mut(userid) {
            if let Some(item) = queue.pending.pop() {
                if queue.pending.is_empty() {
                    queue.is_processing = false;
                }
                return Some(item);
            }
        }
        None
    }

    /// 获取队列长度
    pub fn queue_length(&self, userid: &str) -> usize {
        self.queues.get(userid).map(|q| q.pending.len()).unwrap_or(0)
    }

    /// 清空用户队列
    pub fn clear_queue(&mut self, userid: &str) {
        if let Some(queue) = self.queues.get_mut(userid) {
            queue.pending.clear();
            queue.is_processing = false;
        }
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行单个命令 (内部函数)
pub async fn execute_command_internal(
    userid: String,
    command: String,
    vconn: &VirtualConnection,
) -> Result<String, String> {
    tracing::debug!("Executing command '{}' for user '{}'", command, userid);

    // TODO: 通过 efuns 系统执行命令
    // 1. 设置 this_player
    // 2. 调用 EFUNSD->command()
    // 3. 收集输出到 vconn.buffer

    let output = format!("Command executed: {}\n", command);
    vconn.write(&output).await;

    Ok(vconn.get_output().await)
}
