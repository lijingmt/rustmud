// gamenv/single/daemons/toptend.rs - 排行榜守护进程
// 对应 txpike9/gamenv/single/daemons/toptend.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

/// 排行榜类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RankType {
    /// 等级排行榜
    Level,
    /// 战力排行榜
    Power,
    /// 财富排行榜
    Gold,
    /// 帮派排行榜
    Guild,
    /// 击杀排行榜
    Kills,
    /// 死亡排行榜
    Deaths,
}

/// 排行榜条目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RankEntry {
    /// 用户ID
    pub userid: String,
    /// 用户名
    pub name: String,
    /// 分数/数值
    pub score: u64,
    /// 额外数据
    pub data: serde_json::Value,
}

/// 排行榜
#[derive(Clone, Debug)]
pub struct RankBoard {
    /// 排行榜类型
    pub rank_type: RankType,
    /// 条目
    entries: Vec<RankEntry>,
    /// 最大条目数
    max_entries: usize,
    /// 上次更新时间
    last_update: i64,
    /// 更新间隔（秒）
    update_interval: i64,
}

impl RankBoard {
    /// 创建新的排行榜
    pub fn new(rank_type: RankType, max_entries: usize) -> Self {
        Self {
            rank_type,
            entries: Vec::new(),
            max_entries,
            last_update: 0,
            update_interval: 300, // 5分钟更新一次
        }
    }

    /// 设置更新间隔
    pub fn set_update_interval(&mut self, seconds: i64) {
        self.update_interval = seconds;
    }

    /// 检查是否需要更新
    pub fn needs_update(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.last_update >= self.update_interval
    }

    /// 更新排行榜
    pub fn update(&mut self, entries: Vec<RankEntry>) {
        self.entries = entries;
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        self.entries.truncate(self.max_entries);
        self.last_update = chrono::Utc::now().timestamp();
    }

    /// 获取前N名
    pub fn get_top(&self, n: usize) -> &[RankEntry] {
        &self.entries[..self.entries.len().min(n)]
    }

    /// 获取排名
    pub fn get_rank(&self, userid: &str) -> Option<usize> {
        self.entries.iter().position(|e| e.userid == userid).map(|r| r + 1)
    }

    /// 获取条目
    pub fn get_entry(&self, userid: &str) -> Option<&RankEntry> {
        self.entries.iter().find(|e| e.userid == userid)
    }

    /// 格式化输出
    pub fn format(&self, top_n: usize) -> String {
        let mut output = format!("§H=== {}排行榜 ===§N\n", self.rank_type_name());

        for (idx, entry) in self.get_top(top_n).iter().enumerate() {
            let rank = idx + 1;
            let medal = match rank {
                1 => "§Y[第1名]§N",
                2 => "§W[第2名]§N",
                3 => "§C[第3名]§N",
                _ => &format!("[第{}名]", rank),
            };
            output.push_str(&format!("{} {} - {}分\n", medal, entry.name, entry.score));
        }

        output
    }

    /// 排行榜名称
    fn rank_type_name(&self) -> &str {
        match self.rank_type {
            RankType::Level => "等级",
            RankType::Power => "战力",
            RankType::Gold => "财富",
            RankType::Guild => "帮派",
            RankType::Kills => "击杀",
            RankType::Deaths => "死亡",
        }
    }
}

/// 排行榜守护进程
#[derive(Clone)]
pub struct ToptendDaemon {
    /// 所有排行榜
    boards: HashMap<RankType, RankBoard>,
}

impl ToptendDaemon {
    /// 创建新的排行榜守护进程
    pub fn new() -> Self {
        let mut boards = HashMap::new();

        boards.insert(RankType::Level, RankBoard::new(RankType::Level, 100));
        boards.insert(RankType::Power, RankBoard::new(RankType::Power, 100));
        boards.insert(RankType::Gold, RankBoard::new(RankType::Gold, 100));
        boards.insert(RankType::Guild, RankBoard::new(RankType::Guild, 50));
        boards.insert(RankType::Kills, RankBoard::new(RankType::Kills, 100));
        boards.insert(RankType::Deaths, RankBoard::new(RankType::Deaths, 100));

        Self { boards }
    }

    /// 获取排行榜
    pub fn get_board(&self, rank_type: &RankType) -> Option<&RankBoard> {
        self.boards.get(rank_type)
    }

    /// 获取排行榜（可变）
    pub fn get_board_mut(&mut self, rank_type: &RankType) -> Option<&mut RankBoard> {
        self.boards.get_mut(rank_type)
    }

    /// 更新指定排行榜
    pub fn update_board(&mut self, rank_type: RankType, entries: Vec<RankEntry>) {
        if let Some(board) = self.get_board_mut(&rank_type) {
            board.update(entries);
        }
    }

    /// 获取排行榜前N名
    pub fn get_top(&self, rank_type: RankType, n: usize) -> Vec<RankEntry> {
        if let Some(board) = self.get_board(&rank_type) {
            board.get_top(n).to_vec()
        } else {
            Vec::new()
        }
    }

    /// 获取玩家排名
    pub fn get_player_rank(&self, rank_type: RankType, userid: &str) -> Option<usize> {
        if let Some(board) = self.get_board(&rank_type) {
            board.get_rank(userid)
        } else {
            None
        }
    }

    /// 格式化排行榜
    pub fn format_board(&self, rank_type: RankType, top_n: usize) -> String {
        if let Some(board) = self.get_board(&rank_type) {
            board.format(top_n)
        } else {
            "排行榜不存在".to_string()
        }
    }

    /// 检查是否需要更新并更新所有排行榜
    pub async fn check_and_update_all(&mut self) {
        for (rank_type, board) in &mut self.boards {
            if board.needs_update() {
                tracing::info!("Updating {} leaderboard", board.rank_type_name());
                // TODO: 从数据库获取最新数据并更新
            }
        }
    }

    /// 添加玩家分数
    pub fn add_player_score(&mut self, rank_type: RankType, userid: String, name: String, score: u64) {
        if let Some(board) = self.get_board_mut(&rank_type) {
            if let Some(entry) = board.get_entry(&userid) {
                // 更新现有分数
                // 这里需要重新构建整个entries来更新，因为get_entry只返回引用
                let new_entries = board.entries.iter().cloned().map(|mut e| {
                    if e.userid == userid {
                        e.score = score;
                        e.name = name.clone();
                    }
                    e
                }).collect();
                board.update(new_entries);
            } else {
                // 添加新条目
                let entry = RankEntry {
                    userid,
                    name,
                    score,
                    data: serde_json::json!({}),
                };
                let mut new_entries = board.entries.clone();
                new_entries.push(entry);
                board.update(new_entries);
            }
        }
    }

    /// 启动排行榜定时更新
    pub async fn start_update_loop(&self) {
        let daemon = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                // TODO: 触发排行榜更新
                tracing::info!("Ranking update tick");
            }
        });
    }
}

impl Default for ToptendDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局排行榜守护进程
pub static TOPTEND: std::sync::OnceLock<TokioRwLock<ToptendDaemon>> = std::sync::OnceLock::new();

/// 获取排行榜守护进程
pub fn get_toptend() -> &'static TokioRwLock<ToptendDaemon> {
    TOPTEND.get_or_init(|| TokioRwLock::new(ToptendDaemon::new()))
}
