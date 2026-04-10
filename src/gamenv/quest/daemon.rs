// gamenv/quest/daemon.rs - 任务守护进程
// 对应 txpike9/gamenv/single/daemons/questd.pike

use super::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 任务守护进程
pub struct QuestDaemon {
    /// 任务数据
    data: Arc<RwLock<QuestDaemonData>>,
}

impl QuestDaemon {
    /// 创建新的任务守护进程
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(QuestDaemonData::default())),
        }
    }

    /// 添加任务模板
    pub async fn add_quest_template(&self, template: QuestTemplate) {
        let mut data = self.data.write().await;
        let npc_templates = data.quest_templates
            .entry(template.npc_name.clone())
            .or_insert_with(Vec::new);

        // 按道行排序插入
        let pos = npc_templates
            .binary_search_by_key(&template.required_level, |t| t.required_level)
            .unwrap_or_else(|e| e);

        npc_templates.insert(pos, template);
    }

    /// 根据玩家等级获取合适的任务索引
    pub async fn quest_accurate_index(&self, npc_name: &str, player_level: i32) -> Option<usize> {
        let data = self.data.read().await;
        let templates = data.quest_templates.get(npc_name)?;

        // 二分查找最接近的任务
        let mut i = 0;
        let mut k = templates.len();

        while i < k {
            let j = (k - i) / 2 + i;
            let quest_level = templates[j].required_level;

            if quest_level == player_level {
                return Some(j);
            } else if quest_level > player_level {
                k = j;
            } else {
                i = j + 1;
            }
        }

        Some(i.min(templates.len().saturating_sub(1)))
    }

    /// 获取随机任务索引（带浮动）
    pub async fn quest_random_index(
        &self,
        npc_name: &str,
        player_level: i32,
        player: &PlayerQuestData,
    ) -> Option<usize> {
        const INDEX_DELTA: i32 = 200;
        const CACHE_SIZE: usize = 30;

        let data = self.data.read().await;
        let templates = data.quest_templates.get(npc_name)?;

        let base_index = self.quest_accurate_index(npc_name, player_level).await?;

        let mut lower = base_index as i32 - INDEX_DELTA;
        let mut upper = base_index as i32 + INDEX_DELTA;

        if lower < 0 {
            lower = 0;
            upper = INDEX_DELTA * 2;
        }

        if upper >= templates.len() as i32 {
            upper = templates.len() as i32 - 1;
        }

        // 调整下限，允许访问低级任务
        lower = upper / 4;
        if upper - lower < INDEX_DELTA {
            lower = 0;
        }

        // 使用缓存避免重复任务
        let cache_name = npc_name;
        if let Some(cache) = player.quest_cache.get(cache_name) {
            if cache.len() >= CACHE_SIZE {
                // 清理缓存（移除最旧的条目）
                return None;
            }

            // 尝试找到未完成的任务
            for _ in 0..40 {
                let idx = (lower as usize + rand::random::<usize>() % ((upper - lower) as usize + 1))
                    .min(templates.len() - 1);

                if let Some(template) = templates.get(idx) {
                    let cache_key = format!("{}:{}", cache_name, template.quest_name);
                    if !cache.contains_key(&cache_key) {
                        return Some(idx);
                    }
                }
            }
        }

        // 无缓存，直接返回随机任务
        let idx = (lower as usize + rand::random::<usize>() % ((upper - lower) as usize + 1))
            .min(templates.len() - 1);

        Some(idx)
    }

    /// 获取主线任务
    pub async fn get_main_quest(
        &self,
        npc_name: &str,
        player: &PlayerQuestData,
    ) -> Option<(usize, String)> {
        let data = self.data.read().await;
        let templates = data.quest_templates.get(npc_name)?;

        for (idx, template) in templates.iter().enumerate() {
            if template.required_level >= 0 {
                continue; // 只处理主线任务（负道行）
            }

            // 检查前置条件
            if self.can_accept_main_quest(template, player) {
                return Some((idx, template.base_quest.clone()));
            }
        }

        None
    }

    /// 检查是否可以接受主线任务
    fn can_accept_main_quest(&self, template: &QuestTemplate, player: &PlayerQuestData) -> bool {
        // 解析base_quest要求
        // 格式: "/initquest)|[oldquest)|{forbidquest)|finalquest"
        if template.base_quest.is_empty() {
            return true;
        }

        let requirements: Vec<&str> = template.base_quest.split(')').collect();

        for req in requirements {
            if req.is_empty() {
                continue;
            }

            let req = req.trim_start_matches('(').trim_start_matches('/');

            if req.starts_with('{') {
                // 不允许完成的任务
                let quest_name = req.trim_start_matches('{');
                if player.is_main_quest_completed(&template.base_quest, quest_name) {
                    return false;
                }
            } else if req.starts_with('[') {
                // 曾经需要完成的任务
                // 这里简化处理
            } else if !req.starts_with(|c: char| c.is_alphanumeric()) {
                // 最终任务
                for (_main_line, path) in &player.main_quests {
                    if path.ends_with(&format!("/{}", req)) {
                        return false; // 已完成
                    }
                }
            }
        }

        true
    }

    /// 为玩家分配任务
    pub async fn assign_quest(
        &self,
        npc_name: &str,
        player_level: i32,
        player_data: &PlayerQuestData,
    ) -> Option<Quest> {
        let data = self.data.read().await;
        let templates = data.quest_templates.get(npc_name)?;

        // 先尝试获取主线任务
        if let Some((idx, main_line)) = self.get_main_quest(npc_name, player_data).await {
            let template = &templates[idx];
            return Some(self.template_to_quest(template, main_line));
        }

        // 普通任务：根据等级精确匹配
        let base_index = self.quest_accurate_index(npc_name, player_level).await?;

        // 尝试随机任务
        if let Some(idx) = self
            .quest_random_index(npc_name, player_level, player_data)
            .await
        {
            let template = &templates[idx];
            return Some(self.template_to_quest(template, String::new()));
        }

        // 默认返回最低级任务
        let template = &templates.first()?;
        Some(self.template_to_quest(template, String::new()))
    }

    /// 将模板转换为任务
    fn template_to_quest(&self, template: &QuestTemplate, main_line: String) -> Quest {
        let is_main = template.required_level < 0;

        let mut quest = Quest::new(
            format!("{}_{}", template.npc_name, template.quest_name),
            template.quest_type.clone(),
            template.target_npc.clone(),
            template.target_object.clone(),
            template.target_amount,
            template.required_level.abs(),
        )
        .with_talks(
            &template.quest_talk,
            &template.finish_talk,
            &template.npc_talk,
        )
        .with_reward(self.parse_reward(&template.reward_code));

        if is_main {
            quest = quest.with_main_quest(
                &template.quest_name,
                &main_line,
                if template.delete_name.is_empty() {
                    None
                } else {
                    Some(template.delete_name.clone())
                },
            );
        }

        quest
    }

    /// 解析奖励代码
    fn parse_reward(&self, reward_code: &str) -> RewardType {
        // 格式: "type:value:amount"
        // 例如: "add:exp:1000", "money:silver:10", "give:item_id:1"
        let parts: Vec<&str> = reward_code.split(':').collect();

        if parts.len() < 2 {
            return RewardType::Add {
                attr: "exp".to_string(),
                amount: 100,
            };
        }

        let reward_type = parts[0];
        let reward_value = parts[1];
        let amount = if parts.len() > 2 {
            parts[2].parse().unwrap_or(1)
        } else {
            1
        };

        match reward_type {
            "add" => RewardType::Add {
                attr: reward_value.to_string(),
                amount,
            },
            "money" => {
                if let Some(currency) = Currency::from_str(reward_value) {
                    RewardType::Money { currency, amount }
                } else {
                    RewardType::Money {
                        currency: Currency::Silver,
                        amount,
                    }
                }
            }
            "give" => RewardType::GiveItem {
                item_id: reward_value.to_string(),
                amount,
            },
            _ => RewardType::Add {
                attr: "exp".to_string(),
                amount: 100,
            },
        }
    }

    /// 检查并执行给予任务
    pub async fn check_give_quest(
        &self,
        player_data: &mut PlayerQuestData,
        npc_name: &str,
        has_item: bool,
    ) -> Option<String> {
        let quest = player_data.get_quest(npc_name)?;

        match quest.quest_type {
            QuestType::Give | QuestType::Find => {
                if quest.is_completed() {
                    return None; // 已完成，返回空
                }

                if has_item {
                    // 完成任务
                    Some(format!("你把{}交给了{}，可以向{}交差了。",
                        quest.target_object, quest.target_npc, npc_name))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// 检查并执行询问任务
    pub async fn check_ask_quest(
        &self,
        player_data: &mut PlayerQuestData,
        npc_name: &str,
    ) -> Option<String> {
        let quest = player_data.get_quest(npc_name)?;

        if quest.quest_type == QuestType::Ask && !quest.is_completed() {
            Some(format!("关于{}的事情我已经知道了，你可以向{}交差了。",
                quest.target_object, npc_name))
        } else {
            None
        }
    }

    /// 检查并执行击杀任务
    pub async fn check_kill_quest(
        &self,
        player_data: &mut PlayerQuestData,
        npc_name: &str,
    ) -> Option<String> {
        let quest = player_data.get_quest_mut(npc_name)?;

        if quest.quest_type == QuestType::Kill && !quest.is_completed() {
            quest.add_progress(1);
            Some(format!("你击杀了{}！任务进度: {}/{}",
                npc_name, quest.current_amount, quest.target_amount))
        } else {
            None
        }
    }

    /// 完成任务并发放奖励
    pub async fn complete_quest(
        &self,
        player_data: &mut PlayerQuestData,
        npc_name: &str,
    ) -> Option<(Quest, String)> {
        let quest = player_data.remove_quest(npc_name)?;

        // 处理主线任务记录
        if quest.is_main_quest {
            if let Some(main_line) = &quest.main_line_name {
                player_data.add_main_quest_completion(main_line, &quest.quest_name);

                // 删除指定记录
                if let Some(delete_name) = &quest.delete_name {
                    player_data.remove_main_quest_entry(main_line, delete_name);
                }
            }
        }

        // 生成奖励消息
        let reward_msg = self.format_reward(&quest.reward);

        let complete_msg = if !quest.finish_talk.is_empty() {
            quest.finish_talk.clone()
        } else {
            match quest.quest_type {
                QuestType::Find => format!("多谢这位客官，为{}找到{}，这是我的一点心意！",
                    quest.target_npc, quest.target_object),
                QuestType::Give => format!("帮{}把{}交给了{}，这是我的一点心意！",
                    npc_name, quest.target_object, quest.target_npc),
                QuestType::Kill => format!("帮{}杀死了{}，这是我的一点心意！",
                    npc_name, quest.target_npc),
                QuestType::Ask => format!("帮{}向{}打听关于{}的事情，这是我的一点心意！",
                    npc_name, quest.target_npc, quest.target_object),
            }
        };

        Some((quest, format!("{}\n{}", complete_msg, reward_msg)))
    }

    /// 格式化奖励消息
    fn format_reward(&self, reward: &RewardType) -> String {
        match reward {
            RewardType::Add { attr, amount } => {
                match attr.as_str() {
                    "exp" | "daoheng" => format!("§Y你得到了 {} 修为§N", amount),
                    "pot" | "potential" => format!("§Y你得到了 {} 点潜能§N", amount),
                    _ => format!("§Y你得到了 {} {}§N", amount, attr),
                }
            }
            RewardType::Money { currency, amount } => {
                format!("§Y你得到了 {} {}§N", amount, currency.cn_name())
            }
            RewardType::GiveItem { item_id, amount } => {
                format!("§Y你得到了 {} x{}§N", item_id, amount)
            }
        }
    }

    /// 获取随机奖励
    pub async fn get_random_reward(&self, reward_key: &str) -> Option<RewardType> {
        let data = self.data.read().await;
        let rewards = data.rewards.get(reward_key)?;

        if rewards.is_empty() {
            return None;
        }

        let idx = rand::random::<usize>() % rewards.len();
        Some(rewards[idx].clone())
    }

    /// 从CSV文件加载任务模板
    pub async fn load_quests_from_csv(&self, csv_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let csv_content = tokio::fs::read_to_string(csv_path).await?;
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)  // CSV使用数字索引，不需要标题行
            .from_reader(csv_content.as_bytes());

        let mut count = 0;
        for result in rdr.deserialize() {
            let row: QuestCsvRow = result?;

            // 跳过标题行（第一列如果是"发放任务的人"）
            if row.npc_name == "发放任务的人" || row.npc_name.is_empty() {
                continue;
            }

            let quest_type = match QuestType::from_str(row.quest_type.trim()) {
                Some(t) => t,
                None => {
                    eprintln!("Unknown quest type: {}", row.quest_type);
                    continue;
                }
            };

            let required_level = row.required_level.parse().unwrap_or(1000);

            // 合并quest_talk的两部分
            let quest_talk = if !row.quest_talk_part1.is_empty() && !row.quest_talk_part2.is_empty() {
                format!("{}\n{}", row.quest_talk_part1, row.quest_talk_part2)
            } else {
                row.quest_talk_part1.clone()
            };

            let template = QuestTemplate {
                npc_name: row.npc_name,
                required_level,
                quest_type,
                target_npc: row.target_npc,
                target_object: row.target_object,
                target_amount: row.target_amount.parse().unwrap_or(1),
                quest_talk,
                finish_talk: row.finish_talk,
                npc_talk: row.npc_talk,
                reward_code: row.reward_code,
                quest_name: row.quest_name,
                base_quest: row.base_quest,
                delete_name: row.delete_name,
            };

            self.add_quest_template(template).await;
            count += 1;
        }

        Ok(count)
    }

    /// 从CSV文件加载奖励配置
    pub async fn load_rewards_from_csv(&self, csv_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let csv_content = tokio::fs::read_to_string(csv_path).await?;
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(false)  // CSV使用数字索引，不需要标题行
            .from_reader(csv_content.as_bytes());

        let mut count = 0;
        for result in rdr.records() {
            let record = result?;

            if record.len() < 2 {
                continue;
            }

            let reward_key = record.get(0).unwrap_or("").trim();
            let reward_str = record.get(1).unwrap_or("");

            if reward_key.is_empty() || reward_str.is_empty() {
                continue;
            }

            // 跳过标题行
            if reward_key == "模块名称" {
                continue;
            }

            // 解析多行奖励
            let rewards: Vec<RewardType> = reward_str
                .lines()
                .filter(|line| !line.trim().is_empty())
                .filter_map(|line| self.parse_reward_line(line.trim()))
                .collect();

            if !rewards.is_empty() {
                let mut data = self.data.write().await;
                data.rewards.insert(reward_key.to_string(), rewards);
                count += 1;
            }
        }

        Ok(count)
    }

    /// 解析单行奖励
    fn parse_reward_line(&self, line: &str) -> Option<RewardType> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 2 {
            return None;
        }

        let reward_type = parts[0];
        let reward_value = parts[1];
        let amount = if parts.len() > 2 {
            parts[2].parse().unwrap_or(1)
        } else {
            1
        };

        match reward_type {
            "add" => Some(RewardType::Add {
                attr: reward_value.to_string(),
                amount,
            }),
            "money" => {
                if let Some(currency) = Currency::from_str(reward_value) {
                    Some(RewardType::Money { currency, amount })
                } else {
                    Some(RewardType::Money {
                        currency: Currency::Silver,
                        amount,
                    })
                }
            }
            "give" => Some(RewardType::GiveItem {
                item_id: reward_value.to_string(),
                amount,
            }),
            _ => None,
        }
    }

    /// 初始化任务系统（加载所有CSV数据）
    pub async fn initialize(&self, base_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 加载奖励配置
        let reward_path = format!("{}/quest/reward.csv", base_path);
        if let Ok(count) = self.load_rewards_from_csv(&reward_path).await {
            println!("Loaded {} reward configurations from {}", count, reward_path);
        }

        // 加载主线任务
        let main_quest_path = format!("{}/quest_main.csv", base_path);
        if let Ok(count) = self.load_quests_from_csv(&main_quest_path).await {
            println!("Loaded {} main quests from {}", count, main_quest_path);
        }

        // 加载新手村任务
        let xiushoucun_path = format!("{}/quest_xiushoucun.csv", base_path);
        if let Ok(count) = self.load_quests_from_csv(&xiushoucun_path).await {
            println!("Loaded {} xiushoucun quests from {}", count, xiushoucun_path);
        }

        Ok(())
    }
}

impl Default for QuestDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局任务守护进程
lazy_static::lazy_static! {
    pub static ref QUESTD: Arc<QuestDaemon> = Arc::new(QuestDaemon::new());
}
