// gamenv/single/daemons/friendd.rs - 社交系统守护进程
// 对应 txpike9/gamenv/single/daemons/friendd.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 好友关系状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FriendStatus {
    /// 正常
    Normal,
    /// 已屏蔽
    Blocked,
    /// 已删除
    Deleted,
}

/// 好友关系
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FriendRelation {
    /// 玩家ID
    pub player_id: String,
    /// 好友ID
    pub friend_id: String,
    /// 好友名称
    pub friend_name: String,
    /// 状态
    pub status: FriendStatus,
    /// 好友度
    pub intimacy: i32,
    /// 添加时间
    pub added_at: i64,
    /// 最后在线时间
    pub last_online: Option<i64>,
    /// 备注
    pub remark: String,
}

/// 好友申请
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FriendRequest {
    /// 申请ID
    pub request_id: String,
    /// 申请人ID
    pub applicant_id: String,
    /// 申请人名称
    pub applicant_name: String,
    /// 目标玩家ID
    pub target_id: String,
    /// 申请消息
    pub message: String,
    /// 申请时间
    pub created_at: i64,
    /// 是否已过期
    pub expired: bool,
}

/// 社交守护进程
pub struct FriendDaemon {
    /// 好友关系映射 (player_id -> friend_id -> relation)
    friends: HashMap<String, HashMap<String, FriendRelation>>,
    /// 好友申请 (target_id -> requests)
    pending_requests: HashMap<String, Vec<FriendRequest>>,
    /// 黑名单
    blacklist: HashMap<String, Vec<String>>,
}

impl FriendDaemon {
    /// 创建新的社交守护进程
    pub fn new() -> Self {
        Self {
            friends: HashMap::new(),
            pending_requests: HashMap::new(),
            blacklist: HashMap::new(),
        }
    }

    /// 发送好友申请
    pub fn send_request(
        &mut self,
        applicant_id: String,
        applicant_name: String,
        target_id: String,
        message: String,
    ) -> Result<()> {
        // 不能添加自己
        if applicant_id == target_id {
            return Err(MudError::RuntimeError("不能添加自己为好友".to_string()));
        }

        // 检查是否已经是好友
        if let Some(friends) = self.friends.get(&applicant_id) {
            if friends.contains_key(&target_id) {
                return Err(MudError::RuntimeError("对方已经是你的好友".to_string()));
            }
        }

        // 检查是否在黑名单中
        if let Some(blocked) = self.blacklist.get(&target_id) {
            if blocked.contains(&applicant_id) {
                return Err(MudError::RuntimeError("对方已将你加入黑名单".to_string()));
            }
        }

        // 检查是否已有待处理申请
        if let Some(requests) = self.pending_requests.get(&target_id) {
            for req in requests {
                if req.applicant_id == applicant_id && !req.expired {
                    return Err(MudError::RuntimeError("已有待处理的好友申请".to_string()));
                }
            }
        }

        let request = FriendRequest {
            request_id: format!("friend_req_{}_{}",
                applicant_id,
                chrono::Utc::now().timestamp_nanos()
            ),
            applicant_id,
            applicant_name,
            target_id: target_id.clone(),
            message,
            created_at: chrono::Utc::now().timestamp(),
            expired: false,
        };

        self.pending_requests
            .entry(target_id)
            .or_insert_with(Vec::new)
            .push(request);

        Ok(())
    }

    /// 获取待处理申请
    pub fn get_pending_requests(&self, player_id: &str) -> Vec<&FriendRequest> {
        if let Some(requests) = self.pending_requests.get(player_id) {
            requests.iter()
                .filter(|r| !r.expired)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 接受好友申请
    pub fn accept_request(&mut self, request_id: &str, player_id: &str, player_name: String) -> Result<String> {
        // 查找申请
        let mut found_request = None;
        if let Some(requests) = self.pending_requests.get_mut(player_id) {
            for req in requests.iter_mut() {
                if req.request_id == request_id && !req.expired {
                    req.expired = true;
                    found_request = Some(req.clone());
                    break;
                }
            }
        }

        let request = found_request
            .ok_or_else(|| MudError::NotFound("好友申请不存在或已过期".to_string()))?;

        // 创建双向好友关系
        let now = chrono::Utc::now().timestamp();

        let relation1 = FriendRelation {
            player_id: player_id.to_string(),
            friend_id: request.applicant_id.clone(),
            friend_name: request.applicant_name.clone(),
            status: FriendStatus::Normal,
            intimacy: 0,
            added_at: now,
            last_online: None,
            remark: String::new(),
        };

        let relation2 = FriendRelation {
            player_id: request.applicant_id.clone(),
            friend_id: player_id.to_string(),
            friend_name: player_name.clone(),
            status: FriendStatus::Normal,
            intimacy: 0,
            added_at: now,
            last_online: None,
            remark: String::new(),
        };

        self.friends
            .entry(player_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(request.applicant_id.clone(), relation1);

        self.friends
            .entry(request.applicant_id.clone())
            .or_insert_with(HashMap::new)
            .insert(player_id.to_string(), relation2);

        Ok(request.applicant_id)
    }

    /// 拒绝好友申请
    pub fn reject_request(&mut self, request_id: &str, player_id: &str) -> Result<()> {
        if let Some(requests) = self.pending_requests.get_mut(player_id) {
            for req in requests.iter_mut() {
                if req.request_id == request_id && !req.expired {
                    req.expired = true;
                    return Ok(());
                }
            }
        }
        Err(MudError::NotFound("好友申请不存在或已过期".to_string()))
    }

    /// 获取好友列表
    pub fn get_friends(&self, player_id: &str) -> Vec<&FriendRelation> {
        if let Some(friends) = self.friends.get(player_id) {
            friends.values()
                .filter(|f| f.status == FriendStatus::Normal)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取在线好友
    pub fn get_online_friends(&self, player_id: &str, online_check: impl Fn(&str) -> bool) -> Vec<&FriendRelation> {
        self.get_friends(player_id)
            .into_iter()
            .filter(|f| online_check(&f.friend_id))
            .collect()
    }

    /// 删除好友
    pub fn remove_friend(&mut self, player_id: &str, friend_id: &str) -> Result<()> {
        // 删除单向关系
        if let Some(friends) = self.friends.get_mut(player_id) {
            friends.remove(friend_id);
        }

        // 删除对方的关系
        if let Some(friends) = self.friends.get_mut(friend_id) {
            friends.remove(player_id);
        }

        Ok(())
    }

    /// 添加到黑名单
    pub fn add_to_blacklist(&mut self, player_id: &str, target_id: &str) -> Result<()> {
        // 先删除好友关系
        self.remove_friend(player_id, target_id)?;

        self.blacklist
            .entry(player_id.to_string())
            .or_insert_with(Vec::new)
            .push(target_id.to_string());

        Ok(())
    }

    /// 从黑名单移除
    pub fn remove_from_blacklist(&mut self, player_id: &str, target_id: &str) -> Result<()> {
        if let Some(list) = self.blacklist.get_mut(player_id) {
            list.retain(|id| id != target_id);
            Ok(())
        } else {
            Err(MudError::NotFound("黑名单中没有该玩家".to_string()))
        }
    }

    /// 获取黑名单
    pub fn get_blacklist(&self, player_id: &str) -> Vec<&str> {
        if let Some(list) = self.blacklist.get(player_id) {
            list.iter().map(|s| s.as_str()).collect()
        } else {
            Vec::new()
        }
    }

    /// 是否被屏蔽
    pub fn is_blocked(&self, player_id: &str, target_id: &str) -> bool {
        if let Some(list) = self.blacklist.get(player_id) {
            list.contains(&target_id.to_string())
        } else {
            false
        }
    }

    /// 检查是否是好友
    pub fn is_friend(&self, player_id: &str, target_id: &str) -> bool {
        if let Some(friends) = self.friends.get(player_id) {
            friends.get(target_id)
                .map_or(false, |f| f.status == FriendStatus::Normal)
        } else {
            false
        }
    }

    /// 更新好友度
    pub fn add_intimacy(&mut self, player_id: &str, friend_id: &str, amount: i32) {
        if let Some(friends) = self.friends.get_mut(player_id) {
            if let Some(relation) = friends.get_mut(friend_id) {
                relation.intimacy = (relation.intimacy + amount).max(0).min(100);
            }
        }
    }

    /// 获取好友度
    pub fn get_intimacy(&self, player_id: &str, friend_id: &str) -> i32 {
        if let Some(friends) = self.friends.get(player_id) {
            friends.get(friend_id)
                .map_or(0, |f| f.intimacy)
        } else {
            0
        }
    }

    /// 设置备注
    pub fn set_remark(&mut self, player_id: &str, friend_id: &str, remark: String) -> Result<()> {
        if let Some(friends) = self.friends.get_mut(player_id) {
            if let Some(relation) = friends.get_mut(friend_id) {
                relation.remark = remark;
                return Ok(());
            }
        }
        Err(MudError::NotFound("好友不存在".to_string()))
    }

    /// 更新最后在线时间
    pub fn update_last_online(&mut self, player_id: &str) {
        let now = chrono::Utc::now().timestamp();

        // 更新所有好友的last_online
        for (pid, friends) in &mut self.friends {
            if let Some(relation) = friends.get_mut(player_id) {
                relation.last_online = Some(now);
            }
        }
    }

    /// 清理过期申请
    pub fn cleanup_expired_requests(&mut self) -> usize {
        let now = chrono::Utc::now().timestamp();
        let expiry_time = 7 * 24 * 60 * 60; // 7天
        let mut cleaned = 0;

        for requests in self.pending_requests.values_mut() {
            for req in requests.iter_mut() {
                if !req.expired && (now - req.created_at) > expiry_time {
                    req.expired = true;
                    cleaned += 1;
                }
            }
        }

        cleaned
    }

    /// 格式化好友列表
    pub fn format_friend_list(&self, player_id: &str) -> String {
        let friends = self.get_friends(player_id);

        let mut output = format!("§H=== 好友列表 ({}人) ===§N\n", friends.len());

        if friends.is_empty() {
            output.push_str("你还没有好友。\n");
        } else {
            for friend in friends {
                let online = if let Some(last) = friend.last_online {
                    let elapsed = chrono::Utc::now().timestamp() - last;
                    if elapsed < 300 {
                        "§G[在线]§N"
                    } else {
                        let mins = elapsed / 60;
                        if mins < 60 {
                            &format!("[{}分钟前]", mins)
                        } else {
                            let hours = mins / 60;
                            &format!("[{}小时前]", hours)
                        }
                    }
                } else {
                    "[离线]"
                };

                let intimacy_level = if friend.intimacy >= 80 {
                    "§Y挚友§N"
                } else if friend.intimacy >= 50 {
                    "§C好友§N"
                } else if friend.intimacy >= 20 {
                    "§B相识§N"
                } else {
                    "§X陌生§N"
                };

                output.push_str(&format!(
                    "  {} {} - {} {}\n",
                    online,
                    friend.friend_name,
                    intimacy_level,
                    if friend.remark.is_empty() {
                        String::new()
                    } else {
                        format!("({})", friend.remark)
                    }
                ));
            }
        }

        output
    }

    /// 格式化待处理申请列表
    pub fn format_pending_requests(&self, player_id: &str) -> String {
        let requests = self.get_pending_requests(player_id);

        let mut output = format!("§H=== 好友申请 ({}条) ===§N\n", requests.len());

        if requests.is_empty() {
            output.push_str("暂无待处理的申请。\n");
        } else {
            for req in requests {
                let time = chrono::DateTime::from_timestamp(req.created_at, 0)
                    .map(|dt| dt.format("%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "未知".to_string());

                output.push_str(&format!(
                    "  {} [{}] {}\n    \"{}\"\n",
                    req.applicant_name,
                    time,
                    if req.message.is_empty() { "无留言" } else { &req.message },
                    req.request_id
                ));
            }
        }

        output
    }
}

impl Default for FriendDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局社交守护进程
pub static FRIENDD: std::sync::OnceLock<RwLock<FriendDaemon>> = std::sync::OnceLock::new();

/// 获取社交守护进程
pub fn get_friendd() -> &'static RwLock<FriendDaemon> {
    FRIENDD.get_or_init(|| RwLock::new(FriendDaemon::default()))
}
