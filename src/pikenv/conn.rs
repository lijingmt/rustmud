// pikenv/conn.rs - 连接处理
// 对应 txpike9/pikenv/conn.pike

use crate::core::{GObject, ObjectId, Value, MudError, Result};
use crate::pikenv::efuns::EfunManager;
use crate::pikenv::connd::CONND;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::sync::Arc;
use crate::core::object::ObjectInner;

/// 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connected,
    Closing,
    Closed,
}

/// 用户连接 (对应 CONN 类)
pub struct Connection {
    /// TCP 流
    stream: Arc<tokio::sync::Mutex<TcpStream>>,
    /// 用户对象
    user: Option<GObject>,
    /// 输入缓冲区
    input_buffer: Arc<tokio::sync::Mutex<String>>,
    /// 输出缓冲区
    output_buffer: Arc<tokio::sync::Mutex<Vec<u8>>>,
    /// 连接状态
    state: Arc<tokio::sync::Mutex<ConnectionState>>,
    /// 输入回调 (对应 on_input)
    input_callback: Arc<tokio::sync::Mutex<Option<InputCallback>>>,
}

/// 输入回调类型
pub type InputCallback = Box<dyn Fn(GObject, String) + Send + Sync>;

impl Connection {
    /// 创建新连接 (对应 conn.pike 的 create())
    pub fn new(stream: TcpStream, user: GObject) -> Self {
        Self {
            stream: Arc::new(tokio::sync::Mutex::new(stream)),
            user: Some(user),
            input_buffer: Arc::new(tokio::sync::Mutex::new(String::new())),
            output_buffer: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            state: Arc::new(tokio::sync::Mutex::new(ConnectionState::Connected)),
            input_callback: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// 写入数据 (对应 write())
    pub async fn write(&self, data: &str) -> Result<()> {
        let state = self.state.lock().await;
        if *state == ConnectionState::Closed {
            return Ok(());
        }
        drop(state);

        let mut output = self.output_buffer.lock().await;
        output.extend_from_slice(data.as_bytes());
        self.flush().await?;
        Ok(())
    }

    /// 刷新输出缓冲区 (对应 write_callback)
    pub async fn flush(&self) -> Result<()> {
        let mut output = self.output_buffer.lock().await;
        if !output.is_empty() {
            let mut stream = self.stream.lock().await;
            stream.write_all(&output.clone()).await?;
            stream.flush().await?;
            output.clear();
        }
        Ok(())
    }

    /// 设置输入回调 (对应 input_to())
    pub async fn set_input_callback(&self, callback: InputCallback) {
        let mut cb = self.input_callback.lock().await;
        *cb = Some(callback);
    }

    /// 处理输入数据 (对应 read_callback)
    pub async fn handle_input(&self, data: String) -> Result<()> {
        let mut input = self.input_buffer.lock().await;
        input.push_str(&data);

        // 按行分割处理
        let lines: Vec<String> = input
            .split('\n')
            .map(|s| s.trim_end_matches('\r').to_string())
            .collect();

        // 保存未完成的行
        if let Some(last) = lines.last() {
            let has_newline = data.ends_with('\n');
            if !has_newline && !lines.is_empty() {
                *input = lines.last().unwrap().clone();
            } else {
                input.clear();
            }
        }

        // 处理完整的行
        for line in lines.iter().filter(|l| !l.is_empty()) {
            self.process_line(line.clone()).await?;
        }

        Ok(())
    }

    /// 处理单行输入
    async fn process_line(&self, line: String) -> Result<()> {
        let user = self.user.as_ref().ok_or(MudError::ConnectionClosed)?;

        // 检查是否有 input_to 回调
        {
            let mut cb = self.input_callback.lock().await;
            if let Some(callback) = cb.take() {
                callback(user.clone(), line);
                return Ok(());
            }
        }

        // 正常命令处理
        // 1. process_input 过滤
        // 2. 命令执行
        // 3. write_prompt

        let efuns = EfunManager::instance();
        let command = if line == "0" { "look".to_string() } else { line };

        // TODO: 调用命令系统
        tracing::debug!("Processing command: {}", command);

        Ok(())
    }

    /// 关闭连接 (对应 close())
    pub async fn close(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        if *state == ConnectionState::Closed {
            return Ok(());
        }
        *state = ConnectionState::Closing;

        // 触发 net_dead
        if let Some(ref user) = self.user {
            // TODO: 调用 user->net_dead()
        }

        drop(state);
        self.try_close().await
    }

    /// 尝试关闭 (对应 tryclose)
    async fn try_close(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        *state = ConnectionState::Closed;

        // 关闭 TCP 连接
        let mut stream = self.stream.lock().await;
        stream.shutdown().await?;

        Ok(())
    }
}

/// 接受连接并处理 (对应 accept_callback)
pub async fn accept_connection(
    stream: TcpStream,
    user_factory: impl Fn() -> GObject,
) -> Result<Arc<Connection>> {
    let user = user_factory();
    let conn = Arc::new(Connection::new(stream, user.clone()));

    // 注册到 CONND
    let user_id = {
        let inner = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(user.inner.read())
        });
        inner.id
    };
    CONND.set_conn(user_id, conn.clone());

    // 调用 logon
    // TODO: user.call_method("logon", vec![]).await?;

    Ok(conn)
}
