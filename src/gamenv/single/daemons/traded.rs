// gamenv/single/daemons/traded.rs - 玩家交易系统守护进程
// 对应 txpike9/gamenv/single/daemons/traded.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

/// 交易状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TradeStatus {
    /// 等待确认
    Waiting,
    /// 已确认
    Confirmed,
    /// 已取消
    Cancelled,
    /// 已完成
    Completed,
}

/// 交易物品
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeItem {
    /// 物品ID
    pub item_id: String,
    /// 物品名称
    pub item_name: String,
    /// 数量
    pub count: i32,
    /// 品质
    pub quality: i32,
}

/// 交易槽位
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeSlot {
    /// 物品列表
    pub items: Vec<TradeItem>,
    /// 金币
    pub gold: u64,
    /// 是否已确认
    pub confirmed: bool,
}

impl Default for TradeSlot {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            gold: 0,
            confirmed: false,
        }
    }
}

/// 交易会话
#[derive(Clone, Debug)]
pub struct TradeSession {
    /// 交易ID
    pub trade_id: String,
    /// 玩家1 ID
    pub player1_id: String,
    /// 玩家1 名称
    pub player1_name: String,
    /// 玩家2 ID
    pub player2_id: String,
    /// 玩家2 名称
    pub player2_name: String,
    /// 玩家1 槽位
    pub player1_slot: TradeSlot,
    /// 玩家2 槽位
    pub player2_slot: TradeSlot,
    /// 交易状态
    pub status: TradeStatus,
    /// 创建时间
    pub created_at: i64,
    /// 最后更新时间
    pub updated_at: i64,
}

impl TradeSession {
    /// 创建新交易
    pub fn new(
        trade_id: String,
        player1_id: String,
        player1_name: String,
        player2_id: String,
        player2_name: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            trade_id,
            player1_id,
            player1_name,
            player2_id,
            player2_name,
            player1_slot: TradeSlot::default(),
            player2_slot: TradeSlot::default(),
            status: TradeStatus::Waiting,
            created_at: now,
            updated_at: now,
        }
    }

    /// 是否在交易中
    pub fn is_active(&self) -> bool {
        matches!(self.status, TradeStatus::Waiting | TradeStatus::Confirmed)
    }

    /// 获取对手ID
    pub fn get_opponent(&self, player_id: &str) -> Option<&str> {
        if player_id == self.player1_id {
            Some(&self.player2_id)
        } else if player_id == self.player2_id {
            Some(&self.player1_id)
        } else {
            None
        }
    }

    /// 获取玩家槽位
    pub fn get_slot(&self, player_id: &str) -> Option<&TradeSlot> {
        if player_id == self.player1_id {
            Some(&self.player1_slot)
        } else if player_id == self.player2_id {
            Some(&self.player2_slot)
        } else {
            None
        }
    }

    /// 获取可变槽位
    pub fn get_slot_mut(&mut self, player_id: &str) -> Option<&mut TradeSlot> {
        if player_id == self.player1_id {
            Some(&mut self.player1_slot)
        } else if player_id == self.player2_id {
            Some(&mut self.player2_slot)
        } else {
            None
        }
    }

    /// 添加物品
    pub fn add_item(&mut self, player_id: &str, item: TradeItem) -> Result<()> {
        let slot = self.get_slot_mut(player_id)
            .ok_or_else(|| MudError::NotFound("玩家不在交易中".to_string()))?;

        if slot.confirmed {
            return Err(MudError::RuntimeError("已确认，不能修改".to_string()));
        }

        slot.items.push(item);
        self.updated_at = chrono::Utc::now().timestamp();
        Ok(())
    }

    /// 移除物品
    pub fn remove_item(&mut self, player_id: &str, index: usize) -> Result<TradeItem> {
        let slot = self.get_slot_mut(player_id)
            .ok_or_else(|| MudError::NotFound("玩家不在交易中".to_string()))?;

        if slot.confirmed {
            return Err(MudError::RuntimeError("已确认，不能修改".to_string()));
        }

        if index >= slot.items.len() {
            return Err(MudError::NotFound("物品不存在".to_string()));
        }

        let item = slot.items.remove(index);
        self.updated_at = chrono::Utc::now().timestamp();
        Ok(item)
    }

    /// 设置金币
    pub fn set_gold(&mut self, player_id: &str, gold: u64) -> Result<()> {
        let slot = self.get_slot_mut(player_id)
            .ok_or_else(|| MudError::NotFound("玩家不在交易中".to_string()))?;

        if slot.confirmed {
            return Err(MudError::RuntimeError("已确认，不能修改".to_string()));
        }

        slot.gold = gold;
        self.updated_at = chrono::Utc::now().timestamp();
        Ok(())
    }

    /// 确认交易
    pub fn confirm(&mut self, player_id: &str) -> Result<bool> {
        let slot = self.get_slot_mut(player_id)
            .ok_or_else(|| MudError::NotFound("玩家不在交易中".to_string()))?;

        slot.confirmed = true;
        self.updated_at = chrono::Utc::now().timestamp();

        // 检查是否双方都确认
        if self.player1_slot.confirmed && self.player2_slot.confirmed {
            self.status = TradeStatus::Confirmed;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 取消确认
    pub fn unconfirm(&mut self, player_id: &str) -> Result<()> {
        // 先检查状态，避免借用冲突
        if self.status == TradeStatus::Confirmed {
            return Err(MudError::RuntimeError("交易已锁定，不能取消".to_string()));
        }

        let slot = self.get_slot_mut(player_id)
            .ok_or_else(|| MudError::NotFound("玩家不在交易中".to_string()))?;

        slot.confirmed = false;
        self.updated_at = chrono::Utc::now().timestamp();
        Ok(())
    }

    /// 是否可以完成
    pub fn can_complete(&self) -> bool {
        self.player1_slot.confirmed && self.player2_slot.confirmed
    }

    /// 格式化交易信息
    pub fn format_info(&self, player_id: &str) -> String {
        let is_player1 = player_id == self.player1_id;
        let my_slot = if is_player1 { &self.player1_slot } else { &self.player2_slot };
        let other_slot = if is_player1 { &self.player2_slot } else { &self.player1_slot };
        let other_name = if is_player1 { &self.player2_name } else { &self.player1_name };

        let mut output = format!(
            "§H=== 与 {} 的交易 ===§N\n\
             状态: {}\n\
             我的物品:\n",
            other_name,
            match self.status {
                TradeStatus::Waiting => "等待确认",
                TradeStatus::Confirmed => "§G已锁定§N",
                TradeStatus::Cancelled => "§R已取消§N",
                TradeStatus::Completed => "§Y已完成§N",
            }
        );

        if my_slot.items.is_empty() {
            output.push_str("  (空)\n");
        } else {
            for (i, item) in my_slot.items.iter().enumerate() {
                output.push_str(&format!("  [{}] {} x{}\n", i + 1, item.item_name, item.count));
            }
        }

        output.push_str(&format!("金币: {}\n", my_slot.gold));
        output.push_str(&format!("确认状态: {}\n\n", if my_slot.confirmed { "§G已确认§N" } else { "§Y未确认§N" }));

        output.push_str("对方物品:\n");
        if other_slot.items.is_empty() {
            output.push_str("  (空)\n");
        } else {
            for (i, item) in other_slot.items.iter().enumerate() {
                output.push_str(&format!("  [{}] {} x{}\n", i + 1, item.item_name, item.count));
            }
        }

        output.push_str(&format!("金币: {}\n", other_slot.gold));
        output.push_str(&format!("确认状态: {}", if other_slot.confirmed { "§G已确认§N" } else { "§Y未确认§N" }));

        output
    }
}

/// 交易守护进程
pub struct TradeDaemon {
    /// 活跃的交易
    active_trades: HashMap<String, TradeSession>,
    /// 玩家到交易的映射
    player_trades: HashMap<String, String>,
    /// 交易超时时间（秒）
    timeout: i64,
}

impl TradeDaemon {
    /// 创建新的交易守护进程
    pub fn new() -> Self {
        Self {
            active_trades: HashMap::new(),
            player_trades: HashMap::new(),
            timeout: 300, // 5分钟
        }
    }

    /// 发起交易
    pub fn request_trade(
        &mut self,
        player1_id: String,
        player1_name: String,
        player2_id: String,
        player2_name: String,
    ) -> Result<String> {
        // 检查玩家状态
        if self.player_trades.contains_key(&player1_id) {
            return Err(MudError::RuntimeError("你已在交易中".to_string()));
        }

        if self.player_trades.contains_key(&player2_id) {
            return Err(MudError::RuntimeError("对方已在交易中".to_string()));
        }

        let trade_id = format!("trade_{}_{}",
            chrono::Utc::now().timestamp_nanos(),
            rand::random::<u32>()
        );

        let trade = TradeSession::new(
            trade_id.clone(),
            player1_id.clone(),
            player1_name,
            player2_id.clone(),
            player2_name,
        );

        self.active_trades.insert(trade_id.clone(), trade);
        self.player_trades.insert(player1_id, trade_id.clone());
        self.player_trades.insert(player2_id, trade_id.clone());

        Ok(trade_id)
    }

    /// 取消交易
    pub fn cancel_trade(&mut self, player_id: &str) -> Result<()> {
        let trade_id = self.player_trades.get(player_id)
            .ok_or_else(|| MudError::NotFound("未在交易中".to_string()))?
            .clone();

        if let Some(trade) = self.active_trades.get_mut(&trade_id) {
            trade.status = TradeStatus::Cancelled;
        }

        self.cleanup_trade(&trade_id);
        Ok(())
    }

    /// 添加物品到交易
    pub fn add_item(&mut self, player_id: &str, item: TradeItem) -> Result<()> {
        let trade_id = self.player_trades.get(player_id)
            .ok_or_else(|| MudError::NotFound("未在交易中".to_string()))?
            .clone();

        if let Some(trade) = self.active_trades.get_mut(&trade_id) {
            trade.add_item(player_id, item)
        } else {
            Err(MudError::NotFound("交易不存在".to_string()))
        }
    }

    /// 移除物品
    pub fn remove_item(&mut self, player_id: &str, index: usize) -> Result<TradeItem> {
        let trade_id = self.player_trades.get(player_id)
            .ok_or_else(|| MudError::NotFound("未在交易中".to_string()))?
            .clone();

        if let Some(trade) = self.active_trades.get_mut(&trade_id) {
            trade.remove_item(player_id, index)
        } else {
            Err(MudError::NotFound("交易不存在".to_string()))
        }
    }

    /// 设置金币
    pub fn set_gold(&mut self, player_id: &str, gold: u64) -> Result<()> {
        let trade_id = self.player_trades.get(player_id)
            .ok_or_else(|| MudError::NotFound("未在交易中".to_string()))?
            .clone();

        if let Some(trade) = self.active_trades.get_mut(&trade_id) {
            trade.set_gold(player_id, gold)
        } else {
            Err(MudError::NotFound("交易不存在".to_string()))
        }
    }

    /// 确认交易
    pub fn confirm_trade(&mut self, player_id: &str) -> Result<bool> {
        let trade_id = self.player_trades.get(player_id)
            .ok_or_else(|| MudError::NotFound("未在交易中".to_string()))?
            .clone();

        if let Some(trade) = self.active_trades.get_mut(&trade_id) {
            trade.confirm(player_id)
        } else {
            Err(MudError::NotFound("交易不存在".to_string()))
        }
    }

    /// 取消确认
    pub fn unconfirm_trade(&mut self, player_id: &str) -> Result<()> {
        let trade_id = self.player_trades.get(player_id)
            .ok_or_else(|| MudError::NotFound("未在交易中".to_string()))?
            .clone();

        if let Some(trade) = self.active_trades.get_mut(&trade_id) {
            trade.unconfirm(player_id)
        } else {
            Err(MudError::NotFound("交易不存在".to_string()))
        }
    }

    /// 完成交易
    pub fn complete_trade(&mut self, player_id: &str) -> Result<TradeResult> {
        let trade_id = self.player_trades.get(player_id)
            .ok_or_else(|| MudError::NotFound("未在交易中".to_string()))?
            .clone();

        let trade = self.active_trades.get(&trade_id)
            .ok_or_else(|| MudError::NotFound("交易不存在".to_string()))?;

        if !trade.can_complete() {
            return Err(MudError::RuntimeError("双方都未确认".to_string()));
        }

        // 获取交易结果
        let result = TradeResult {
            player1_id: trade.player1_id.clone(),
            player1_items: trade.player2_slot.items.clone(),
            player1_gold: trade.player2_slot.gold,
            player2_id: trade.player2_id.clone(),
            player2_items: trade.player1_slot.items.clone(),
            player2_gold: trade.player1_slot.gold,
        };

        // 标记完成
        if let Some(trade) = self.active_trades.get_mut(&trade_id) {
            trade.status = TradeStatus::Completed;
        }

        self.cleanup_trade(&trade_id);
        Ok(result)
    }

    /// 获取交易信息
    pub fn get_trade_info(&self, player_id: &str) -> Option<String> {
        let trade_id = self.player_trades.get(player_id)?;
        let trade = self.active_trades.get(trade_id)?;
        Some(trade.format_info(player_id))
    }

    /// 获取交易对手
    pub fn get_opponent(&self, player_id: &str) -> Option<String> {
        let trade_id = self.player_trades.get(player_id)?;
        let trade = self.active_trades.get(trade_id)?;
        trade.get_opponent(player_id).map(|s| s.to_string())
    }

    /// 清理交易
    fn cleanup_trade(&mut self, trade_id: &str) {
        if let Some(trade) = self.active_trades.remove(trade_id) {
            self.player_trades.remove(&trade.player1_id);
            self.player_trades.remove(&trade.player2_id);
        }
    }

    /// 清理过期交易
    pub fn cleanup_expired(&mut self) -> usize {
        let now = chrono::Utc::now().timestamp();
        let mut expired = Vec::new();

        for (id, trade) in &self.active_trades {
            if now - trade.updated_at > self.timeout {
                expired.push(id.clone());
            }
        }

        for id in expired {
            if let Some(mut trade) = self.active_trades.remove(&id) {
                trade.status = TradeStatus::Cancelled;
                self.player_trades.remove(&trade.player1_id);
                self.player_trades.remove(&trade.player2_id);
            }
        }

        self.active_trades.len()
    }

    /// 检查玩家是否在交易中
    pub fn is_in_trade(&self, player_id: &str) -> bool {
        self.player_trades.contains_key(player_id)
    }
}

/// 交易结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeResult {
    /// 玩家1 ID
    pub player1_id: String,
    /// 玩家1 获得的物品
    pub player1_items: Vec<TradeItem>,
    /// 玩家1 获得的金币
    pub player1_gold: u64,
    /// 玩家2 ID
    pub player2_id: String,
    /// 玩家2 获得的物品
    pub player2_items: Vec<TradeItem>,
    /// 玩家2 获得的金币
    pub player2_gold: u64,
}

impl Default for TradeDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局交易守护进程
pub static TRADED: std::sync::OnceLock<TokioRwLock<TradeDaemon>> = std::sync::OnceLock::new();

/// 获取交易守护进程
pub fn get_traded() -> &'static TokioRwLock<TradeDaemon> {
    TRADED.get_or_init(|| TokioRwLock::new(TradeDaemon::default()))
}
