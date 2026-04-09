// gamenv/single/daemons/bangd.rs - 帮派系统守护进程
// 对应 txpike9/gamenv/single/daemons/bangd.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 帮派职位
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuildRank {
    /// 成员
    Member = 0,
    /// 长老
    Elder = 1,
    /// 副帮主
    ViceLeader = 2,
    /// 帮主
    Leader = 3,
}

/// 帮派成员信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildMember {
    /// 玩家ID
    pub userid: String,
    /// 玩家名称
    pub name: String,
    /// 职位
    pub rank: GuildRank,
    /// 贡献度
    pub contribution: u64,
    /// 加入时间
    pub joined_at: i64,
}

/// 帮派
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guild {
    /// 帮派ID
    pub id: String,
    /// 帮派名称
    pub name: String,
    /// 帮派简称（最多4个字符）
    pub short_name: String,
    /// 帮主ID
    pub leader_id: String,
    /// 帮主名称
    pub leader_name: String,
    /// 等级
    pub level: u32,
    /// 经验值
    pub exp: u64,
    /// 资金
    pub funds: u64,
    /// 成员列表
    pub members: Vec<GuildMember>,
    /// 最大成员数
    pub max_members: usize,
    /// 创建时间
    pub created_at: i64,
    /// 帮派公告
    pub announcement: String,
    /// 帮派描述
    pub description: String,
}

impl Guild {
    /// 创建新帮派
    pub fn new(id: String, name: String, short_name: String, leader_id: String, leader_name: String) -> Self {
        let leader_id_clone = leader_id.clone();
        let leader_name_clone = leader_name.clone();

        Self {
            id,
            name,
            short_name,
            leader_id,
            leader_name,
            level: 1,
            exp: 0,
            funds: 0,
            members: vec![GuildMember {
                userid: leader_id_clone,
                name: leader_name_clone,
                rank: GuildRank::Leader,
                contribution: 0,
                joined_at: chrono::Utc::now().timestamp(),
            }],
            max_members: 20,
            created_at: chrono::Utc::now().timestamp(),
            announcement: String::new(),
            description: String::new(),
        }
    }

    /// 获取成员数
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// 是否满员
    pub fn is_full(&self) -> bool {
        self.members.len() >= self.max_members
    }

    /// 添加成员
    pub fn add_member(&mut self, userid: String, name: String) -> Result<()> {
        if self.is_full() {
            return Err(MudError::RuntimeError("帮派已满".to_string()));
        }

        self.members.push(GuildMember {
            userid,
            name,
            rank: GuildRank::Member,
            contribution: 0,
            joined_at: chrono::Utc::now().timestamp(),
        });

        Ok(())
    }

    /// 移除成员
    pub fn remove_member(&mut self, userid: &str) -> Result<()> {
        let pos = self.members.iter()
            .position(|m| m.userid == userid)
            .ok_or_else(|| MudError::NotFound("成员不存在".to_string()))?;

        let member = &self.members[pos];
        if member.rank == GuildRank::Leader {
            return Err(MudError::RuntimeError("无法移除帮主".to_string()));
        }

        self.members.remove(pos);
        Ok(())
    }

    /// 设置成员职位
    pub fn set_member_rank(&mut self, userid: &str, rank: GuildRank) -> Result<()> {
        let member = self.members.iter_mut()
            .find(|m| m.userid == userid)
            .ok_or_else(|| MudError::NotFound("成员不存在".to_string()))?;

        if member.rank == GuildRank::Leader {
            return Err(MudError::RuntimeError("无法修改帮主职位".to_string()));
        }

        member.rank = rank;
        Ok(())
    }

    /// 添加贡献
    pub fn add_contribution(&mut self, userid: &str, amount: u64) {
        if let Some(member) = self.members.iter_mut().find(|m| m.userid == userid) {
            member.contribution += amount;
        }
    }

    /// 增加经验
    pub fn add_exp(&mut self, exp: u64) -> bool {
        self.exp += exp;

        // 简化升级公式：每级需要 1000 * 等级 经验
        let needed = 1000 * (self.level as u64);
        if self.exp >= needed {
            self.exp -= needed;
            self.level += 1;
            self.max_members = (20 + (self.level * 5) as usize).min(100);
            return true;
        }

        false
    }

    /// 格式化帮派信息
    pub fn format_info(&self) -> String {
        format!(
            "§H=== {} ===§N\n\
             等级: {}  成员: {}/{}  资金: {}\
             帮主: {}\
             帮派简介: {}\
             公告: {}",
            self.name,
            self.level,
            self.member_count(),
            self.max_members,
            self.funds,
            self.leader_name,
            if self.description.is_empty() { "暂无" } else { &self.description },
            if self.announcement.is_empty() { "暂无" } else { &self.announcement }
        )
    }

    /// 格式化成员列表
    pub fn format_members(&self) -> String {
        let mut output = format!("§H=== {} 成员列表 ===§N\n", self.name);

        for member in &self.members {
            let rank_name = match member.rank {
                GuildRank::Leader => "§Y[帮主]§N",
                GuildRank::ViceLeader => "§C[副帮主]§N",
                GuildRank::Elder => "§G[长老]§N",
                GuildRank::Member => "[成员]",
            };

            output.push_str(&format!(
                "{} {} - 贡献: {}\n",
                rank_name,
                member.name,
                member.contribution
            ));
        }

        output
    }
}

/// 帮派申请
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuildApplication {
    /// 玩家ID
    pub userid: String,
    /// 玩家名称
    pub name: String,
    /// 申请时间
    pub applied_at: i64,
}

/// 帮派守护进程
pub struct BangDaemon {
    /// 所有帮派
    guilds: HashMap<String, Guild>,
    /// 玩家到帮派的映射
    player_guilds: HashMap<String, String>,
    /// 帮派申请 (guild_id -> Vec<Application>)
    applications: HashMap<String, Vec<GuildApplication>>,
}

impl BangDaemon {
    /// 创建新的帮派守护进程
    pub fn new() -> Self {
        Self {
            guilds: HashMap::new(),
            player_guilds: HashMap::new(),
            applications: HashMap::new(),
        }
    }

    /// 创建帮派
    pub fn create_guild(
        &mut self,
        creator_id: String,
        creator_name: String,
        name: String,
        short_name: String,
    ) -> Result<Guild> {
        // 检查玩家是否已在帮派中
        if self.player_guilds.contains_key(&creator_id) {
            return Err(MudError::RuntimeError("你已在其他帮派中".to_string()));
        }

        // 检查帮派名称是否存在
        if self.guilds.values().any(|g| g.name == name) {
            return Err(MudError::RuntimeError("帮派名称已存在".to_string()));
        }

        let guild_id = format!("guild_{}", chrono::Utc::now().timestamp());
        let guild = Guild::new(guild_id.clone(), name, short_name, creator_id.clone(), creator_name.clone());

        self.player_guilds.insert(creator_id, guild_id.clone());
        self.guilds.insert(guild_id.clone(), guild.clone());

        Ok(guild)
    }

    /// 解散帮派
    pub fn disband_guild(&mut self, guild_id: &str, leader_id: &str) -> Result<()> {
        let guild = self.guilds.get(guild_id)
            .ok_or_else(|| MudError::NotFound("帮派不存在".to_string()))?;

        if guild.leader_id != leader_id {
            return Err(MudError::PermissionDenied);
        }

        // 移除所有成员的映射
        for member in &guild.members {
            self.player_guilds.remove(&member.userid);
        }

        self.guilds.remove(guild_id);
        self.applications.remove(guild_id);

        Ok(())
    }

    /// 加入帮派
    pub fn join_guild(&mut self, guild_id: &str, userid: String, name: String) -> Result<()> {
        let guild = self.guilds.get_mut(guild_id)
            .ok_or_else(|| MudError::NotFound("帮派不存在".to_string()))?;

        guild.add_member(userid.clone(), name)?;
        self.player_guilds.insert(userid, guild_id.to_string());

        Ok(())
    }

    /// 离开帮派
    pub fn leave_guild(&mut self, userid: &str) -> Result<()> {
        let guild_id = self.player_guilds.get(userid)
            .ok_or_else(|| MudError::NotFound("你不在任何帮派中".to_string()))?
            .clone();

        let guild = self.guilds.get_mut(&guild_id)
            .ok_or_else(|| MudError::NotFound("帮派不存在".to_string()))?;

        if guild.leader_id == userid {
            return Err(MudError::RuntimeError("帮主无法离开帮派".to_string()));
        }

        guild.remove_member(userid)?;
        self.player_guilds.remove(userid);

        Ok(())
    }

    /// 获取玩家的帮派
    pub fn get_player_guild(&self, userid: &str) -> Option<&Guild> {
        if let Some(guild_id) = self.player_guilds.get(userid) {
            self.guilds.get(guild_id)
        } else {
            None
        }
    }

    /// 获取帮派
    pub fn get_guild(&self, guild_id: &str) -> Option<&Guild> {
        self.guilds.get(guild_id)
    }

    /// 获取所有帮派
    pub fn get_all_guilds(&self) -> Vec<&Guild> {
        self.guilds.values().collect()
    }

    /// 提交入帮申请
    pub fn apply_to_guild(&mut self, guild_id: &str, userid: String, name: String) -> Result<()> {
        // 检查是否已在帮派中
        if self.player_guilds.contains_key(&userid) {
            return Err(MudError::RuntimeError("你已在其他帮派中".to_string()));
        }

        let applications = self.applications.entry(guild_id.to_string()).or_insert_with(Vec::new);

        // 检查是否已申请
        if applications.iter().any(|a| a.userid == userid) {
            return Err(MudError::RuntimeError("你已申请过此帮派".to_string()));
        }

        applications.push(GuildApplication {
            userid,
            name,
            applied_at: chrono::Utc::now().timestamp(),
        });

        Ok(())
    }

    /// 处理入帮申请
    pub fn handle_application(
        &mut self,
        guild_id: &str,
        applicant_id: &str,
        accept: bool,
    ) -> Result<()> {
        let applications = self.applications.get_mut(guild_id)
            .ok_or_else(|| MudError::NotFound("没有申请记录".to_string()))?;

        let pos = applications.iter()
            .position(|a| a.userid == applicant_id)
            .ok_or_else(|| MudError::NotFound("申请不存在".to_string()))?;

        let application = applications.remove(pos);

        if accept {
            self.join_guild(guild_id, application.userid.clone(), application.name)?;
        }

        Ok(())
    }

    /// 格式化帮派列表
    pub fn format_guild_list(&self) -> String {
        let mut output = String::from("§H=== 帮派列表 ===§N\n");

        let mut guilds: Vec<_> = self.guilds.values().collect();
        guilds.sort_by(|a, b| b.level.cmp(&a.level));

        for guild in guilds {
            output.push_str(&format!(
                "§Y[{}]§N Lv.{}  成员:{}/{}  帮主:{}\n",
                guild.name,
                guild.level,
                guild.member_count(),
                guild.max_members,
                guild.leader_name
            ));
        }

        output
    }

    /// 获取玩家的职位
    pub fn get_player_rank(&self, userid: &str) -> Option<GuildRank> {
        if let Some(guild_id) = self.player_guilds.get(userid) {
            if let Some(guild) = self.guilds.get(guild_id) {
                return guild.members.iter()
                    .find(|m| m.userid == userid)
                    .map(|m| m.rank.clone());
            }
        }
        None
    }

    /// 是否是帮主
    pub fn is_leader(&self, userid: &str, guild_id: &str) -> bool {
        if let Some(guild) = self.guilds.get(guild_id) {
            return guild.leader_id == userid;
        }
        false
    }

    /// 捐赠资金
    pub fn donate(&mut self, userid: &str, amount: u64) -> Result<()> {
        let guild_id = self.player_guilds.get(userid)
            .ok_or_else(|| MudError::NotFound("你不在任何帮派中".to_string()))?
            .clone();

        let guild = self.guilds.get_mut(&guild_id)
            .ok_or_else(|| MudError::NotFound("帮派不存在".to_string()))?;

        guild.funds += amount;
        guild.add_contribution(userid, amount);

        Ok(())
    }
}

impl Default for BangDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局帮派守护进程
pub static BANGD: std::sync::OnceLock<RwLock<BangDaemon>> = std::sync::OnceLock::new();

/// 获取帮派守护进程
pub fn get_bangd() -> &'static RwLock<BangDaemon> {
    BANGD.get_or_init(|| RwLock::new(BangDaemon::default()))
}
