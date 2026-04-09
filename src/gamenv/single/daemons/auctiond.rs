// gamenv/single/daemons/auctiond.rs - 市场拍卖系统守护进程
// 对应 txpike9/gamenv/single/daemons/auctiond.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 拍品状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AuctionStatus {
    /// 拍卖中
    Active,
    /// 已成交
    Sold,
    /// 已流拍
    Failed,
    /// 已取消
    Cancelled,
}

/// 拍品类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AuctionItemType {
    /// 装备
    Equipment,
    /// 消耗品
    Consumable,
    /// 材料
    Material,
    /// 宠物
    Pet,
    /// 其他
    Other,
}

/// 拍品
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuctionItem {
    /// 拍品ID
    pub id: String,
    /// 物品ID
    pub item_id: String,
    /// 物品名称
    pub item_name: String,
    /// 物品类型
    pub item_type: AuctionItemType,
    /// 物品描述
    pub description: String,
    /// 卖家ID
    pub seller_id: String,
    /// 卖家名称
    pub seller_name: String,
    /// 起拍价
    pub starting_price: u64,
    /// 一口价
    pub buyout_price: Option<u64>,
    /// 当前最高出价
    pub current_bid: u64,
    /// 当前最高出价者
    pub current_bidder: Option<String>,
    /// 最低加价幅度
    pub min_increment: u64,
    /// 状态
    pub status: AuctionStatus,
    /// 开始时间
    pub start_time: i64,
    /// 结束时间
    pub end_time: i64,
    /// 品质等级
    pub quality: i32,
}

impl AuctionItem {
    /// 创建新拍品
    pub fn new(
        id: String,
        item_id: String,
        item_name: String,
        item_type: AuctionItemType,
        seller_id: String,
        seller_name: String,
        starting_price: u64,
        duration_hours: u64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        let min_increment = ((starting_price as f32 * 0.05) as u64).max(1);

        Self {
            id,
            item_id,
            item_name,
            item_type,
            description: String::new(),
            seller_id,
            seller_name,
            starting_price,
            buyout_price: None,
            current_bid: starting_price,
            current_bidder: None,
            min_increment,
            status: AuctionStatus::Active,
            start_time: now,
            end_time: now + (duration_hours * 3600) as i64,
            quality: 1,
        }
    }

    /// 是否已过期
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.end_time
    }

    /// 是否可以一口价
    pub fn can_buyout(&self) -> bool {
        self.status == AuctionStatus::Active && self.buyout_price.is_some()
    }

    /// 获取最低出价
    pub fn get_min_bid(&self) -> u64 {
        self.current_bid + self.min_increment
    }

    /// 格式化拍品信息
    pub fn format_info(&self) -> String {
        let type_name = match self.item_type {
            AuctionItemType::Equipment => "装备",
            AuctionItemType::Consumable => "消耗品",
            AuctionItemType::Material => "材料",
            AuctionItemType::Pet => "宠物",
            AuctionItemType::Other => "其他",
        };

        let status_text = match self.status {
            AuctionStatus::Active => "§G拍卖中§N",
            AuctionStatus::Sold => "§Y已成交§N",
            AuctionStatus::Failed => "§R已流拍§N",
            AuctionStatus::Cancelled => "§X已取消§N",
        };

        let remaining = self.end_time - chrono::Utc::now().timestamp();
        let hours = remaining / 3600;
        let mins = (remaining % 3600) / 60;

        let mut output = format!(
            "§H[{}]§N {} - Lv.{}\n\
             卖家: {}\n\
             状态: {} | 剩余: {}小时{}分\n\
             起拍: {} 金币",
            self.item_name,
            type_name,
            self.quality,
            self.seller_name,
            status_text,
            hours.max(0),
            mins.max(0),
            self.starting_price
        );

        if let Some(buyout) = self.buyout_price {
            output.push_str(&format!(" | 一口价: {} 金币", buyout));
        }

        if self.current_bid > self.starting_price {
            output.push_str(&format!(
                "\n当前出价: {} 金币 (出价者: {})",
                self.current_bid,
                self.current_bidder.as_deref().unwrap_or("未知")
            ));
        }

        output
    }

    /// 格式化列表项
    pub fn format_list_item(&self) -> String {
        let status = match self.status {
            AuctionStatus::Active => "§G[拍]§N",
            AuctionStatus::Sold => "§Y[售]§N",
            AuctionStatus::Failed => "§R[流]§N",
            AuctionStatus::Cancelled => "§X[消]§N",
        };

        format!(
            "{} {} - {}金币 ({})",
            status,
            self.item_name,
            self.current_bid,
            self.seller_name
        )
    }
}

/// 出价记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BidRecord {
    /// 出价者ID
    pub bidder_id: String,
    /// 出价者名称
    pub bidder_name: String,
    /// 出价金额
    pub amount: u64,
    /// 出价时间
    pub timestamp: i64,
}

/// 拍卖搜索条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuctionSearch {
    /// 物品类型
    pub item_type: Option<AuctionItemType>,
    /// 最低价格
    pub min_price: Option<u64>,
    /// 最高价格
    pub max_price: Option<u64>,
    /// 最低品质
    pub min_quality: Option<i32>,
    /// 卖家ID
    pub seller_id: Option<String>,
    /// 搜索关键词
    pub keyword: Option<String>,
}

impl Default for AuctionSearch {
    fn default() -> Self {
        Self {
            item_type: None,
            min_price: None,
            max_price: None,
            min_quality: None,
            seller_id: None,
            keyword: None,
        }
    }
}

/// 拍卖守护进程
pub struct AuctionDaemon {
    /// 所有拍品
    items: HashMap<String, AuctionItem>,
    /// 出价记录
    bid_history: HashMap<String, Vec<BidRecord>>,
    /// 玩家拍品索引
    seller_items: HashMap<String, Vec<String>>,
    /// 拍卖手续费率 (0.01 = 1%)
    fee_rate: f32,
    /// 最低上架时长（小时）
    min_duration: u64,
    /// 最高上架时长（小时）
    max_duration: u64,
}

impl AuctionDaemon {
    /// 创建新的拍卖守护进程
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            bid_history: HashMap::new(),
            seller_items: HashMap::new(),
            fee_rate: 0.05, // 5% 手续费
            min_duration: 1,
            max_duration: 72, // 最长3天
        }
    }

    /// 上架拍品
    pub fn list_item(
        &mut self,
        mut item: AuctionItem,
        seller_id: String,
    ) -> Result<()> {
        // 验证时长
        let duration = (item.end_time - item.start_time) as u64 / 3600;
        if duration < self.min_duration || duration > self.max_duration {
            return Err(MudError::RuntimeError(
                format!("拍卖时长必须在{}-{}小时之间", self.min_duration, self.max_duration)
            ));
        }

        // 验证价格
        if item.starting_price == 0 {
            return Err(MudError::RuntimeError("起拍价不能为0".to_string()));
        }

        item.id = format!("auction_{}_{}",
            seller_id,
            chrono::Utc::now().timestamp_nanos()
        );

        let item_id = item.id.clone();
        let seller_items_list = self.seller_items
            .entry(seller_id.clone())
            .or_insert_with(Vec::new);
        seller_items_list.push(item_id.clone());

        self.items.insert(item_id.clone(), item);
        Ok(())
    }

    /// 出价
    pub fn place_bid(
        &mut self,
        auction_id: &str,
        bidder_id: String,
        bidder_name: String,
        amount: u64,
    ) -> Result<()> {
        let item = self.items.get_mut(auction_id)
            .ok_or_else(|| MudError::NotFound("拍品不存在".to_string()))?;

        // 验证状态
        if item.status != AuctionStatus::Active {
            return Err(MudError::RuntimeError("拍品不在拍卖中".to_string()));
        }

        // 验证时间
        if item.is_expired() {
            item.status = AuctionStatus::Failed;
            return Err(MudError::RuntimeError("拍卖已结束".to_string()));
        }

        // 验证出价
        let min_bid = item.get_min_bid();
        if amount < min_bid {
            return Err(MudError::RuntimeError(
                format!("出价太低，最低出价为{}金币", min_bid)
            ));
        }

        // 不能自己出价
        if item.seller_id == bidder_id {
            return Err(MudError::RuntimeError("不能竞拍自己的物品".to_string()));
        }

        // 更新出价
        item.current_bid = amount;
        item.current_bidder = Some(bidder_id.clone());

        // 记录出价历史
        self.bid_history
            .entry(auction_id.to_string())
            .or_insert_with(Vec::new)
            .push(BidRecord {
                bidder_id,
                bidder_name,
                amount,
                timestamp: chrono::Utc::now().timestamp(),
            });

        Ok(())
    }

    /// 一口价购买
    pub fn buyout(
        &mut self,
        auction_id: &str,
        buyer_id: String,
        buyer_name: String,
    ) -> Result<(AuctionItem, u64)> {
        let item = self.items.get_mut(auction_id)
            .ok_or_else(|| MudError::NotFound("拍品不存在".to_string()))?;

        // 验证状态
        if item.status != AuctionStatus::Active {
            return Err(MudError::RuntimeError("拍品不在拍卖中".to_string()));
        }

        // 验证一口价
        let buyout_price = item.buyout_price
            .ok_or_else(|| MudError::RuntimeError("此拍品不支持一口价".to_string()))?;

        // 不能自己购买
        if item.seller_id == buyer_id {
            return Err(MudError::RuntimeError("不能购买自己的物品".to_string()));
        }

        // 更新状态
        item.status = AuctionStatus::Sold;
        item.current_bid = buyout_price;
        item.current_bidder = Some(buyer_id.clone());

        // 记录出价
        self.bid_history
            .entry(auction_id.to_string())
            .or_insert_with(Vec::new)
            .push(BidRecord {
                bidder_id: buyer_id.clone(),
                bidder_name: buyer_name,
                amount: buyout_price,
                timestamp: chrono::Utc::now().timestamp(),
            });

        // 计算手续费
        let fee = (buyout_price as f32 * self.fee_rate) as u64;
        let seller_receive = buyout_price - fee;

        Ok((item.clone(), seller_receive))
    }

    /// 取消拍卖
    pub fn cancel_auction(&mut self, auction_id: &str, seller_id: &str) -> Result<()> {
        let item = self.items.get_mut(auction_id)
            .ok_or_else(|| MudError::NotFound("拍品不存在".to_string()))?;

        // 验证权限
        if item.seller_id != seller_id {
            return Err(MudError::PermissionDenied);
        }

        // 验证状态
        if item.status != AuctionStatus::Active {
            return Err(MudError::RuntimeError("拍品不在拍卖中".to_string()));
        }

        // 如果已有出价，不能取消
        if item.current_bidder.is_some() {
            return Err(MudError::RuntimeError("已有出价，无法取消".to_string()));
        }

        item.status = AuctionStatus::Cancelled;
        Ok(())
    }

    /// 获取拍品
    pub fn get_item(&self, auction_id: &str) -> Option<&AuctionItem> {
        self.items.get(auction_id)
    }

    /// 搜索拍品
    pub fn search_items(&self, search: &AuctionSearch) -> Vec<&AuctionItem> {
        self.items.values()
            .filter(|item| {
                // 只显示拍卖中的
                if item.status != AuctionStatus::Active {
                    return false;
                }

                // 类型过滤
                if let Some(ref t) = search.item_type {
                    if &item.item_type != t {
                        return false;
                    }
                }

                // 价格过滤
                if let Some(min) = search.min_price {
                    if item.current_bid < min {
                        return false;
                    }
                }
                if let Some(max) = search.max_price {
                    if item.current_bid > max {
                        return false;
                    }
                }

                // 品质过滤
                if let Some(min_q) = search.min_quality {
                    if item.quality < min_q {
                        return false;
                    }
                }

                // 卖家过滤
                if let Some(ref seller) = search.seller_id {
                    if &item.seller_id != seller {
                        return false;
                    }
                }

                // 关键词过滤
                if let Some(ref keyword) = search.keyword {
                    if !item.item_name.contains(keyword) && !item.description.contains(keyword) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// 获取卖家的拍品
    pub fn get_seller_items(&self, seller_id: &str) -> Vec<&AuctionItem> {
        if let Some(item_ids) = self.seller_items.get(seller_id) {
            item_ids.iter()
                .filter_map(|id| self.items.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 处理过期拍卖（定时调用）
    pub fn process_expired_auctions(&mut self) -> Vec<(AuctionItem, Option<String>, u64)> {
        let mut completed = Vec::new();
        let now = chrono::Utc::now().timestamp();

        for item in self.items.values_mut() {
            if item.status == AuctionStatus::Active && item.end_time < now {
                if let Some(ref bidder_id) = item.current_bidder {
                    // 已售出
                    item.status = AuctionStatus::Sold;
                    let fee = (item.current_bid as f32 * self.fee_rate) as u64;
                    let seller_receive = item.current_bid - fee;
                    completed.push((item.clone(), Some(bidder_id.clone()), seller_receive));
                } else {
                    // 流拍
                    item.status = AuctionStatus::Failed;
                    completed.push((item.clone(), None, 0));
                }
            }
        }

        completed
    }

    /// 清理已完成的拍品
    pub fn cleanup_completed(&mut self, days: i64) -> usize {
        let cutoff = chrono::Utc::now().timestamp() - (days * 86400);
        let mut to_remove = Vec::new();

        for (id, item) in &self.items {
            if item.end_time < cutoff {
                to_remove.push(id.clone());
            }
        }

        for id in &to_remove {
            if let Some(item) = self.items.remove(id) {
                // 从卖家索引中移除
                if let Some(items) = self.seller_items.get_mut(&item.seller_id) {
                    items.retain(|x| x != id);
                }
            }
        }

        to_remove.len()
    }

    /// 格式化拍卖列表
    pub fn format_auction_list(&self, items: &[&AuctionItem]) -> String {
        let mut output = format!("§H=== 拍卖列表 ({}件) ===§N\n", items.len());

        if items.is_empty() {
            output.push_str("没有符合条件的拍品。\n");
        } else {
            for item in items {
                output.push_str(&format!("  {}\n", item.format_list_item()));
            }
        }

        output
    }

    /// 获取出价历史
    pub fn get_bid_history(&self, auction_id: &str) -> Vec<&BidRecord> {
        if let Some(records) = self.bid_history.get(auction_id) {
            records.iter().collect()
        } else {
            Vec::new()
        }
    }

    /// 格式化出价历史
    pub fn format_bid_history(&self, auction_id: &str) -> String {
        let records = self.get_bid_history(auction_id);
        let mut output = String::from("§H=== 出价记录 ===§N\n");

        if records.is_empty() {
            output.push_str("暂无出价记录。\n");
        } else {
            for record in records.iter().rev().take(10) {
                let time = chrono::DateTime::from_timestamp(record.timestamp, 0)
                    .map(|dt| dt.format("%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "未知".to_string());

                output.push_str(&format!(
                    "{} - {} 出价 {}金币\n",
                    time, record.bidder_name, record.amount
                ));
            }
        }

        output
    }
}

impl Default for AuctionDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局拍卖守护进程
pub static AUCTIOND: std::sync::OnceLock<RwLock<AuctionDaemon>> = std::sync::OnceLock::new();

/// 获取拍卖守护进程
pub fn get_auctiond() -> &'static RwLock<AuctionDaemon> {
    AUCTIOND.get_or_init(|| RwLock::new(AuctionDaemon::default()))
}
