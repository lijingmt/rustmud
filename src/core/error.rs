// core/error.rs - 错误处理系统
// 对应 txpike9 的 Pike 错误处理

use thiserror::Error;

/// MUD 错误类型
#[derive(Error, Debug)]
pub enum MudError {
    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Program compile error: {0}")]
    CompileError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Timeout")]
    Timeout,
}

/// 错误处理器 (对应 Pike 的 handle_error)
pub struct ErrorHandler {
    log_file: Option<String>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            log_file: None,
        }
    }

    pub fn with_log(mut self, path: String) -> Self {
        self.log_file = Some(path);
        self
    }

    /// 处理错误 (对应 master()->handle_error())
    pub fn handle_error(&self, error: &MudError, header: Option<&str>) {
        let header = header.unwrap_or("ERROR");
        let log_msg = format!(
            "\n-----{}-----\n{}: {:?}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            header,
            error
        );

        eprintln!("{}", log_msg);

        if let Some(ref log_path) = self.log_file {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                use std::io::Write;
                let _ = file.write_all(log_msg.as_bytes());
            }
        }
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub type Result<T> = std::result::Result<T, MudError>;
