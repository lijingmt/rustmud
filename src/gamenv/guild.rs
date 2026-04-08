// gamenv/guild.rs - 帮派系统
// 对应 txpike9/gamenv/single/bangd.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 帮派职位
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GuildRank {
    /// 帮主
    Leader = 0,
    /// 副帮主
    ViceLeader = 1,
    /// 长老
    Elder = 2,
    /// 堂主
    HallMaster = 3,
    /// 精英
    Elite = 4,
    /// 普通成员
    Member = 5,
    /// 新人
    Novice = 6,
}

impl GuildRank {
    /// 获取职位名称
    pub fn name(&self) -> &str {
        match self {
            GuildRank::Leader => "帮主",
            GuildRank::ViceLeader => "副帮主",
            GuildRank::Elder => "长老",
            GuildRank::HallMaster => "堂主",
            GuildRank::Elite => "精英",
            GuildRank::Member => "成员",
            GuildRank::Novice => "新人",
        }
    }

    /// 检查权限
    pub fn can_invite(&self) -> bool {
        *self <= GuildRank::HallMaster
    }

    pub fn can_kick(&self) -> bool {
        *self <= GuildRank::ViceLeader
    }

    pub fn can_promote(&self) -> bool {
        *self <= GuildRank::ViceLeader
    }

    pub fn can_disband(&self) -> bool {
        *self == GuildRank::Leader
    }
}

/// 帮派成员信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMember {
    /// 成员ID (用户名)
    pub id: String,
    /// 成员名称
    pub name: String,
    /// 职位
    pub rank: GuildRank,
    /// 贡献度
    pub contribution: u32,
    /// 加入时间
    pub join_time: i64,
    /// 最后在线时间
    pub last_online: i64,
}

impl GuildMember {
    pub fn new(id: String, name: String, rank: GuildRank) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            name,
            rank,
            contribution: 0,
            join_time: now,
            last_online: now,
        }
    }
}

/// 帮派
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    /// 帮派ID
    pub id: String,
    /// 帮派名称
    pub name: String,
    /// 帮派描述
    pub description: String,
    /// 帮主ID
    pub leader_id: String,
    /// 成员列表 (成员ID -> 成员信息)
    pub members: HashMap<String, GuildMember>,
    /// 帮派等级
    pub level: u32,
    /// 帮派经验
    pub exp: u64,
    /// 帮派资金
    pub funds: u64,
    /// 最大成员数
    pub max_members: u32,
    /// 创建时间
    pub created_time: i64,
    /// 帮派公告
    pub announcement: String,
    /// 入帮申请 (用户ID列表)
    pub applications: Vec<String>,
    /// 邀请列表 (用户ID -> 邀请时间)
    pub invitations: HashMap<String, i64>,
}

impl Guild {
    /// 创建新帮派
    pub fn new(id: String, name: String, leader_id: String, leader_name: String) -> Self {
        let mut members = HashMap::new();
        members.insert(leader_id.clone(), GuildMember::new(
            leader_id.clone(),
            leader_name,
            GuildRank::Leader,
        ));

        let now = chrono::Utc::now().timestamp();

        Self {
            id,
            name,
            description: String::new(),
            leader_id,
            members,
            level: 1,
            exp: 0,
            funds: 0,
            max_members: 20,
            created_time: now,
            announcement: String::new(),
            applications: Vec::new(),
            invitations: HashMap::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    /// 添加成员
    pub fn add_member(&mut self, user_id: String, user_name: String, rank: GuildRank) -> Result<()> {
        if self.members.len() >= self.max_members as usize {
            return Err(MudError::InvalidOperation("帮派成员已满".to_string()));
        }

        if self.members.contains_key(&user_id) {
            return Err(MudError::InvalidOperation("玩家已在帮派中".to_string()));
        }

        let member = GuildMember::new(user_id.clone(), user_name, rank);
        self.members.insert(user_id, member);
        Ok(())
    }

    /// 移除成员
    pub fn remove_member(&mut self, user_id: &str) -> Result<GuildMember> {
        if user_id == self.leader_id {
            return Err(MudError::InvalidOperation("无法移除帮主".to_string()));
        }

        self.members.remove(user_id)
            .ok_or_else(|| MudError::NotFound("玩家不在帮派中".to_string()))
    }

    /// 获取成员
    pub fn get_member(&self, user_id: &str) -> Option<&GuildMember> {
        self.members.get(user_id)
    }

    /// 获取可变成员
    pub fn get_member_mut(&mut self, user_id: &str) -> Option<&mut GuildMember> {
        self.members.get_mut(user_id)
    }

    /// 更新成员职位
    pub fn promote_member(&mut self, user_id: &str, new_rank: GuildRank) -> Result<()> {
        if user_id == self.leader_id {
            return Err(MudError::InvalidOperation("无法更改帮主职位".to_string()));
        }

        let member = self.members.get_mut(user_id)
            .ok_or_else(|| MudError::NotFound("玩家不在帮派中".to_string()))?;

        member.rank = new_rank;
        Ok(())
    }

    /// 添加贡献
    pub fn add_contribution(&mut self, user_id: &str, amount: u32) -> Result<()> {
        let member = self.members.get_mut(user_id)
            .ok_or_else(|| MudError::NotFound("玩家不在帮派中".to_string()))?;

        member.contribution += amount;
        Ok(())
    }

    /// 更新最后在线时间
    pub fn update_last_online(&mut self, user_id: &str) {
        if let Some(member) = self.members.get_mut(user_id) {
            member.last_online = chrono::Utc::now().timestamp();
        }
    }

    /// 获取成员数量
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// 增加经验
    pub fn add_exp(&mut self, exp: u64) {
        self.exp += exp;

        // 检查升级
        let exp_needed = self.exp_for_next_level();
        if self.exp >= exp_needed {
            self.exp -= exp_needed;
            self.level_up();
        }
    }

    /// 升级所需经验
    pub fn exp_for_next_level(&self) -> u64 {
        (self.level as u64) * 1000
    }

    /// 帮派升级
    pub fn level_up(&mut self) {
        self.level += 1;
        self.max_members = 20 + (self.level * 5);
    }

    /// 添加入帮申请
    pub fn add_application(&mut self, user_id: String) {
        if !self.applications.contains(&user_id) {
            self.applications.push(user_id);
        }
    }

    /// 移除入帮申请
    pub fn remove_application(&mut self, user_id: &str) -> bool {
        if let Some(pos) = self.applications.iter().position(|x| x == user_id) {
            self.applications.remove(pos);
            true
        } else {
            false
        }
    }

    /// 邀请玩家
    pub fn invite_player(&mut self, user_id: String) {
        let now = chrono::Utc::now().timestamp();
        self.invitations.insert(user_id, now);
    }

    /// 检查邀请是否有效 (24小时有效)
    pub fn is_invitation_valid(&self, user_id: &str) -> bool {
        if let Some(&time) = self.invitations.get(user_id) {
            let now = chrono::Utc::now().timestamp();
            now - time < 86400 // 24小时
        } else {
            false
        }
    }

    /// 渲染帮派信息
    pub fn render_info(&self) -> String {
        let mut result = format!("§c【{}】§r (Lv.{})\n", self.name, self.level);
        result.push_str(&format!("成员: {}/{}\n", self.member_count(), self.max_members));
        result.push_str(&format!("资金: {}\n", self.funds));
        result.push_str(&format!("经验: {}/{}\n", self.exp, self.exp_for_next_level()));

        if !self.description.is_empty() {
            result.push_str(&format!("\n{}\n", self.description));
        }

        if !self.announcement.is_empty() {
            result.push_str(&format!("\n§c公告:§r {}\n", self.announcement));
        }

        result
    }

    /// 渲染成员列表
    pub fn render_members(&self) -> String {
        let mut result = String::from("=== 帮派成员 ===\n");

        let mut members: Vec<_> = self.members.values().collect();
        members.sort_by_key(|m| m.rank);

        for member in members {
            result.push_str(&format!("{} [{}] 贡献: {}\n",
                member.name,
                member.rank.name(),
                member.contribution
            ));
        }

        result
    }
}

/// 玩家帮派数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGuildData {
    /// 帮派ID (如果没有帮派则为None)
    pub guild_id: Option<String>,
    /// 入帮时间
    pub join_time: Option<i64>,
    /// 本周贡献
    pub weekly_contribution: u32,
    /// 上次重置时间
    pub last_reset: i64,
}

impl Default for PlayerGuildData {
    fn default() -> Self {
        Self {
            guild_id: None,
            join_time: None,
            weekly_contribution: 0,
            last_reset: chrono::Utc::now().timestamp(),
        }
    }
}

impl PlayerGuildData {
    /// 是否有帮派
    pub fn has_guild(&self) -> bool {
        self.guild_id.is_some()
    }

    /// 加入帮派
    pub fn join_guild(&mut self, guild_id: String) {
        self.guild_id = Some(guild_id);
        self.join_time = Some(chrono::Utc::now().timestamp());
    }

    /// 离开帮派
    pub fn leave_guild(&mut self) {
        self.guild_id = None;
        self.join_time = None;
        self.weekly_contribution = 0;
    }

    /// 检查并重置每周贡献
    pub fn check_reset_weekly(&mut self) {
        use chrono::{TimeZone, Datelike};

        let now = chrono::Utc::now();
        // 获取ISO周数
        let current_week = (now.year(), now.iso_week().week());

        let last_dt = chrono::Utc.timestamp_opt(self.last_reset, 0);
        if let Some(last_dt) = last_dt.single() {
            let last_week = (last_dt.year(), last_dt.iso_week().week());
            if current_week != last_week {
                self.weekly_contribution = 0;
                self.last_reset = now.timestamp();
            }
        }
    }

    /// 添加贡献
    pub fn add_contribution(&mut self, amount: u32) {
        self.weekly_contribution += amount;
    }
}

/// 帮派管理器
pub struct GuildManager {
    /// 帮派列表 (帮派ID -> 帮派)
    guilds: HashMap<String, Guild>,
    /// 名称索引 (帮派名称 -> 帮派ID)
    name_index: HashMap<String, String>,
}

impl GuildManager {
    pub fn new() -> Self {
        Self {
            guilds: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    /// 创建帮派
    pub fn create_guild(&mut self, id: String, name: String, leader_id: String, leader_name: String) -> Result<Guild> {
        if self.name_index.contains_key(&name) {
            return Err(MudError::InvalidOperation("帮派名称已存在".to_string()));
        }

        let guild = Guild::new(id.clone(), name.clone(), leader_id, leader_name);
        self.name_index.insert(name, id.clone());
        self.guilds.insert(id, guild.clone());

        Ok(guild)
    }

    /// 获取帮派
    pub fn get_guild(&self, guild_id: &str) -> Option<&Guild> {
        self.guilds.get(guild_id)
    }

    /// 通过名称获取帮派
    pub fn get_guild_by_name(&self, name: &str) -> Option<&Guild> {
        if let Some(guild_id) = self.name_index.get(name) {
            self.guilds.get(guild_id)
        } else {
            None
        }
    }

    /// 获取可变帮派
    pub fn get_guild_mut(&mut self, guild_id: &str) -> Option<&mut Guild> {
        self.guilds.get_mut(guild_id)
    }

    /// 删除帮派
    pub fn remove_guild(&mut self, guild_id: &str) -> Result<Guild> {
        let guild = self.guilds.remove(guild_id)
            .ok_or_else(|| MudError::NotFound("帮派不存在".to_string()))?;

        self.name_index.remove(&guild.name);
        Ok(guild)
    }

    /// 解散帮派
    pub fn disband_guild(&mut self, guild_id: &str, leader_id: &str) -> Result<()> {
        let guild = self.guilds.get(guild_id)
            .ok_or_else(|| MudError::NotFound("帮派不存在".to_string()))?;

        if guild.leader_id != leader_id {
            return Err(MudError::PermissionDenied);
        }

        self.remove_guild(guild_id)?;
        Ok(())
    }

    /// 获取所有帮派
    pub fn all_guilds(&self) -> Vec<&Guild> {
        self.guilds.values().collect()
    }

    /// 渲染帮派列表
    pub fn render_guild_list(&self) -> String {
        let mut result = String::from("=== 帮派列表 ===\n");

        for guild in self.guilds.values() {
            result.push_str(&format!("§c{}§r (Lv.{}) 成员: {}/{}\n",
                guild.name,
                guild.level,
                guild.member_count(),
                guild.max_members
            ));
        }

        result
    }
}

impl Default for GuildManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局帮派管理器
pub static BANGD: once_cell::sync::Lazy<std::sync::Mutex<GuildManager>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(GuildManager::default()));
