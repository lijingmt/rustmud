// gamenv/single/daemons/maild.rs - 邮件系统守护进程
// 对应 txpike9 的邮件系统

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 邮件类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MailType {
    /// 普通邮件
    Normal,
    /// 系统邮件
    System,
    /// 奖励邮件
    Reward,
    /// 战报
    BattleReport,
}

/// 邮件附件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MailAttachment {
    /// 物品ID
    pub item_id: String,
    /// 物品名称
    pub item_name: String,
    /// 数量
    pub count: i32,
}

/// 邮件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mail {
    /// 邮件ID
    pub id: String,
    /// 发件人ID
    pub sender_id: String,
    /// 发件人名称
    pub sender_name: String,
    /// 收件人ID
    pub receiver_id: String,
    /// 邮件类型
    pub mail_type: MailType,
    /// 标题
    pub title: String,
    /// 内容
    pub content: String,
    /// 金币附件
    pub gold: u64,
    /// 物品附件
    pub items: Vec<MailAttachment>,
    /// 是否已读
    pub read: bool,
    /// 是否已领取附件
    pub claimed: bool,
    /// 发送时间
    pub sent_at: i64,
    /// 过期时间
    pub expire_at: i64,
}

impl Mail {
    /// 创建新邮件
    pub fn new(
        sender_id: String,
        sender_name: String,
        receiver_id: String,
        mail_type: MailType,
        title: String,
        content: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: format!("mail_{}_{}", now, rand::random::<u32>()),
            sender_id,
            sender_name,
            receiver_id,
            mail_type,
            title,
            content,
            gold: 0,
            items: vec![],
            read: false,
            claimed: false,
            sent_at: now,
            expire_at: now + (30 * 24 * 60 * 60), // 30天
        }
    }

    /// 是否已过期
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expire_at
    }

    /// 是否有附件
    pub fn has_attachment(&self) -> bool {
        self.gold > 0 || !self.items.is_empty()
    }

    /// 格式化邮件列表项
    pub fn format_list_item(&self) -> String {
        let type_icon = match self.mail_type {
            MailType::Normal => "",
            MailType::System => "§C[系统]§N ",
            MailType::Reward => "§Y[奖励]§N ",
            MailType::BattleReport => "§R[战报]§N ",
        };

        let status = if !self.read {
            "§G[未读]§N"
        } else if self.has_attachment() && !self.claimed {
            "§Y[附件]§N"
        } else {
            ""
        };

        format!("{}{} {} - {}", type_icon, status, self.sender_name, self.title)
    }

    /// 格式化邮件详情
    pub fn format_detail(&self) -> String {
        let mut output = format!(
            "§H=== 邮件详情 ===§N\n\
             发件人: {}\n\
             时间: {}\n\
             标题: {}\n\
             \n{}\n",
            self.sender_name,
            chrono::DateTime::from_timestamp(self.sent_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "未知".to_string()),
            self.title,
            self.content
        );

        if self.has_attachment() {
            output.push_str("\n§H--- 附件 ---§N\n");
            if self.gold > 0 {
                output.push_str(&format!("金币: {}\n", self.gold));
            }
            for item in &self.items {
                output.push_str(&format!("{} x{}\n", item.item_name, item.count));
            }
        }

        output
    }
}

/// 邮件守护进程
pub struct MailDaemon {
    /// 所有邮件
    mails: HashMap<String, Mail>,
    /// 收件箱索引
    inbox: HashMap<String, Vec<String>>,
    /// 发件箱索引
    sentbox: HashMap<String, Vec<String>>,
}

impl MailDaemon {
    /// 创建新的邮件守护进程
    pub fn new() -> Self {
        Self {
            mails: HashMap::new(),
            inbox: HashMap::new(),
            sentbox: HashMap::new(),
        }
    }

    /// 发送邮件
    pub fn send_mail(&mut self, mail: Mail) -> Result<()> {
        let mail_id = mail.id.clone();
        let receiver_id = mail.receiver_id.clone();

        // 添加到邮件列表
        self.mails.insert(mail_id.clone(), mail.clone());

        // 添加到收件箱
        self.inbox
            .entry(receiver_id)
            .or_insert_with(Vec::new)
            .push(mail_id.clone());

        // 添加到发件箱
        self.sentbox
            .entry(mail.sender_id.clone())
            .or_insert_with(Vec::new)
            .push(mail_id);

        Ok(())
    }

    /// 获取收件箱
    pub fn get_inbox(&self, userid: &str) -> Vec<&Mail> {
        if let Some(mail_ids) = self.inbox.get(userid) {
            let mut mails = Vec::new();
            for mail_id in mail_ids {
                if let Some(mail) = self.mails.get(mail_id) {
                    if !mail.is_expired() {
                        mails.push(mail);
                    }
                }
            }
            // 按时间倒序
            mails.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));
            mails
        } else {
            Vec::new()
        }
    }

    /// 获取发件箱
    pub fn get_sentbox(&self, userid: &str) -> Vec<&Mail> {
        if let Some(mail_ids) = self.sentbox.get(userid) {
            let mut mails = Vec::new();
            for mail_id in mail_ids {
                if let Some(mail) = self.mails.get(mail_id) {
                    mails.push(mail);
                }
            }
            mails.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));
            mails
        } else {
            Vec::new()
        }
    }

    /// 获取邮件
    pub fn get_mail(&self, mail_id: &str) -> Option<&Mail> {
        self.mails.get(mail_id)
    }

    /// 标记为已读
    pub fn mark_read(&mut self, mail_id: &str) -> Result<()> {
        let mail = self.mails.get_mut(mail_id)
            .ok_or_else(|| MudError::NotFound("邮件不存在".to_string()))?;

        mail.read = true;
        Ok(())
    }

    /// 领取附件
    pub fn claim_attachment(&mut self, mail_id: &str) -> Result<(u64, Vec<MailAttachment>)> {
        let mail = self.mails.get_mut(mail_id)
            .ok_or_else(|| MudError::NotFound("邮件不存在".to_string()))?;

        if !mail.has_attachment() {
            return Err(MudError::RuntimeError("没有附件".to_string()));
        }

        if mail.claimed {
            return Err(MudError::RuntimeError("附件已领取".to_string()));
        }

        mail.claimed = true;
        Ok((mail.gold, mail.items.clone()))
    }

    /// 删除邮件
    pub fn delete_mail(&mut self, mail_id: &str, userid: &str) -> Result<()> {
        let mail = self.mails.get(mail_id)
            .ok_or_else(|| MudError::NotFound("邮件不存在".to_string()))?;

        // 检查权限
        if mail.receiver_id != userid && mail.sender_id != userid {
            return Err(MudError::PermissionDenied);
        }

        // 从索引中移除
        if let Some(ids) = self.inbox.get_mut(&mail.receiver_id) {
            ids.retain(|id| id != mail_id);
        }
        if let Some(ids) = self.sentbox.get_mut(&mail.sender_id) {
            ids.retain(|id| id != mail_id);
        }

        self.mails.remove(mail_id);
        Ok(())
    }

    /// 清理过期邮件
    pub fn cleanup_expired(&mut self) -> usize {
        let mut expired_ids = Vec::new();

        for (mail_id, mail) in &self.mails {
            if mail.is_expired() {
                expired_ids.push(mail_id.clone());
            }
        }

        for mail_id in &expired_ids {
            if let Some(mail) = self.mails.remove(mail_id) {
                // 从索引中移除
                if let Some(ids) = self.inbox.get_mut(&mail.receiver_id) {
                    ids.retain(|id| id != mail_id);
                }
                if let Some(ids) = self.sentbox.get_mut(&mail.sender_id) {
                    ids.retain(|id| id != mail_id);
                }
            }
        }

        expired_ids.len()
    }

    /// 发送系统邮件
    pub fn send_system_mail(
        &mut self,
        receiver_id: String,
        title: String,
        content: String,
        gold: u64,
        items: Vec<MailAttachment>,
    ) {
        let mail = Mail {
            id: format!("sys_{}_{}", chrono::Utc::now().timestamp(), rand::random::<u32>()),
            sender_id: "system".to_string(),
            sender_name: "系统".to_string(),
            receiver_id,
            mail_type: MailType::System,
            title,
            content,
            gold,
            items,
            read: false,
            claimed: false,
            sent_at: chrono::Utc::now().timestamp(),
            expire_at: chrono::Utc::now().timestamp() + (30 * 24 * 60 * 60),
        };

        let _ = self.send_mail(mail);
    }

    /// 格式化收件箱列表
    pub fn format_inbox(&self, userid: &str) -> String {
        let mails = self.get_inbox(userid);

        let mut output = format!("§H=== 收件箱 ({}封) ===§N\n", mails.len());

        if mails.is_empty() {
            output.push_str("收件箱为空。\n");
        } else {
            for mail in mails {
                output.push_str(&format!("  {}\n", mail.format_list_item()));
            }
        }

        output
    }

    /// 获取未读邮件数
    pub fn get_unread_count(&self, userid: &str) -> usize {
        if let Some(mail_ids) = self.inbox.get(userid) {
            mail_ids.iter()
                .filter_map(|id| self.mails.get(id))
                .filter(|mail| !mail.read && !mail.is_expired())
                .count()
        } else {
            0
        }
    }
}

impl Default for MailDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局邮件守护进程
pub static MAILD: std::sync::OnceLock<RwLock<MailDaemon>> = std::sync::OnceLock::new();

/// 获取邮件守护进程
pub fn get_maild() -> &'static RwLock<MailDaemon> {
    MAILD.get_or_init(|| RwLock::new(MailDaemon::default()))
}
