// gamenv/http_api/auth.rs - TXD 认证系统
// 对应 txpike9/gamenv/single/daemons/http_api/auth.pike

use serde::{Deserialize, Serialize};

/// TXD 认证管理器
pub struct AuthManager {
    /// 认证缓存
    auth_cache: std::collections::HashMap<String, AuthData>,
}

/// 认证数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthData {
    pub userid: String,
    pub player_name: String,
    pub authenticated: bool,
    pub login_time: i64,
}

/// 解码后的 TXD Token
#[derive(Debug, Clone)]
pub struct DecodedTxd {
    pub userid: String,
    pub password: String,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            auth_cache: std::collections::HashMap::new(),
        }
    }

    /// 生成 TXD Token (对应 generate_txd)
    /// TXD 格式: 编码后的userid~编码后的password
    pub fn generate_txd(&self, userid: &str, password: Option<&str>) -> String {
        let uid = self.encode_userid(userid);
        let pwd = match password {
            Some(p) => self.encode_password(p),
            None => "dummy".to_string(),
        };
        format!("{}~{}", uid, pwd)
    }

    /// 编码 userid
    fn encode_userid(&self, userid: &str) -> String {
        let mut uid = String::new();
        for (i, c) in userid.chars().enumerate() {
            let tp = c as u8;
            if i % 2 == 0 {
                // 偶数位: +2，特殊处理 y(121)->%7B
                if tp == 121 {
                    uid.push_str("%7B");
                } else if tp == 122 {
                    uid.push_str("%7C");
                } else {
                    uid.push((tp + 2) as char);
                }
            } else {
                // 奇数位: +1
                if tp == 122 {
                    uid.push_str("%7B");
                } else {
                    uid.push((tp + 1) as char);
                }
            }
        }
        uid
    }

    /// 编码 password
    fn encode_password(&self, password: &str) -> String {
        let mut pwd = String::new();
        for (i, c) in password.chars().enumerate() {
            let tp = c as u8;
            if i % 2 == 0 {
                // 偶数位: +1
                pwd.push((tp + 1) as char);
            } else {
                // 奇数位: +2，特殊处理 y(121)->%7C, z(122)->%7B
                if tp == 121 {
                    pwd.push_str("%7C");
                } else if tp == 122 {
                    pwd.push_str("%7B");
                } else {
                    pwd.push((tp + 2) as char);
                }
            }
        }
        pwd
    }

    /// 解码 TXD Token (对应 decode_txd)
    pub fn decode_txd(&self, txd: &str) -> Option<DecodedTxd> {
        if txd.is_empty() || txd == " " {
            return None;
        }

        // 查找 ~ 分隔符
        let pos = txd.find('~')?;
        let stru = &txd[..pos];
        let strp = &txd[pos + 1..];

        // 解码 userid
        let mut userid = String::new();
        for (i, c) in stru.chars().enumerate() {
            if i % 2 == 0 {
                // 偶数位: -2
                if c == '%' {
                    // 处理 URL 编码
                    userid.push(if let Some(rest) = stru[i..].strip_prefix("%7B") {
                        'y'
                    } else if let Some(rest) = stru[i..].strip_prefix("%7C") {
                        'z'
                    } else {
                        c
                    });
                } else {
                    let u = (c as u8).wrapping_sub(2);
                    userid.push(u as char);
                }
            } else {
                // 奇数位: -1
                if c == '%' {
                    // 处理 URL 编码
                    userid.push(if stru[i..].starts_with("%7B") {
                        'z'
                    } else {
                        c
                    });
                } else {
                    let u = (c as u8).wrapping_sub(1);
                    userid.push(u as char);
                }
            }
        }

        // 解码 password
        let mut password = String::new();
        for (i, c) in strp.chars().enumerate() {
            if i % 2 == 0 {
                // 偶数位: -1
                let p = (c as u8).wrapping_sub(1);
                password.push(p as char);
            } else {
                // 奇数位: -2
                if c == '%' {
                    // 处理 URL 编码
                    if strp[i..].starts_with("%7B") {
                        password.push('z');
                    } else if strp[i..].starts_with("%7C") {
                        password.push('y');
                    } else {
                        password.push(c);
                    }
                } else {
                    let p = (c as u8).wrapping_sub(2);
                    password.push(p as char);
                }
            }
        }

        Some(DecodedTxd { userid, password })
    }

    /// 验证 TXD Token
    pub fn verify_txd(&mut self, txd: &str) -> Option<AuthData> {
        let decoded = self.decode_txd(txd)?;

        // TODO: 查询数据库验证用户
        let auth_data = AuthData {
            userid: decoded.userid.clone(),
            player_name: decoded.userid.clone(),
            authenticated: true,
            login_time: chrono::Utc::now().timestamp(),
        };

        // 缓存认证数据
        self.auth_cache.insert(decoded.userid, auth_data.clone());
        Some(auth_data)
    }

    /// 获取缓存的认证数据
    pub fn get_auth_data(&self, userid: &str) -> Option<AuthData> {
        self.auth_cache.get(userid).cloned()
    }

    /// 清除认证缓存
    pub fn clear_cache(&mut self, userid: &str) {
        self.auth_cache.remove(userid);
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

// 全局认证管理器
pub static AUTH_MANAGER: std::sync::OnceLock<std::sync::Mutex<AuthManager>> =
    std::sync::OnceLock::new();

/// 获取认证管理器实例
pub fn get_auth_manager() -> &'static std::sync::Mutex<AuthManager> {
    AUTH_MANAGER.get_or_init(|| std::sync::Mutex::new(AuthManager::new()))
}

/// 处理认证请求
pub async fn handle_auth(
    txd: String,
    _state: &crate::gamenv::http_api::HttpApiState,
) -> Result<serde_json::Value, crate::gamenv::http_api::ApiError> {
    let auth_mgr = get_auth_manager();
    let mut mgr = auth_mgr.lock().map_err(|e| {
        crate::gamenv::http_api::ApiError::Internal(format!("Auth lock error: {}", e))
    })?;

    match mgr.verify_txd(&txd) {
        Some(auth_data) => Ok(serde_json::json!({
            "status": "success",
            "userid": auth_data.userid,
            "player_name": auth_data.player_name,
            "authenticated": auth_data.authenticated,
        })),
        None => Err(crate::gamenv::http_api::ApiError::AuthFailed),
    }
}
