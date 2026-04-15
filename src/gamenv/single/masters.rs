// gamenv/single/masters.rs - 师父/门派系统
// 对应 txpike9 的师父和拜师系统

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 门派等级 - 对应 txpike9 的门派职位
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchoolRank {
    /// 普通弟子
    Disciple = 1,
    /// 入室弟子
    InnerDisciple = 2,
    /// 传功弟子
    SeniorDisciple = 3,
    /// 长老
    Elder = 4,
    /// 掌门
    Master = 5,
    /// 祖师
    GrandMaster = 6,
}

impl SchoolRank {
    /// 获取中文名称
    pub fn cn_name(&self) -> &str {
        match self {
            SchoolRank::Disciple => "普通弟子",
            SchoolRank::InnerDisciple => "入室弟子",
            SchoolRank::SeniorDisciple => "传功弟子",
            SchoolRank::Elder => "长老",
            SchoolRank::Master => "掌门",
            SchoolRank::GrandMaster => "祖师",
        }
    }

    /// 下一等级
    pub fn next(&self) -> Option<SchoolRank> {
        match self {
            SchoolRank::Disciple => Some(SchoolRank::InnerDisciple),
            SchoolRank::InnerDisciple => Some(SchoolRank::SeniorDisciple),
            SchoolRank::SeniorDisciple => Some(SchoolRank::Elder),
            SchoolRank::Elder => Some(SchoolRank::Master),
            SchoolRank::Master => Some(SchoolRank::GrandMaster),
            SchoolRank::GrandMaster => None,
        }
    }
}

/// 师徒关系
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterApprentice {
    /// 师父ID
    pub master_id: String,
    /// 徒弟ID
    pub apprentice_id: String,
    /// 所属门派
    pub school_id: String,
    /// 师父给徒弟起的名字 (法号)
    pub dharma_name: Option<String>,
    /// 拜师时间
    pub since: i64,
    /// 徒弟在门派的等级
    pub rank: SchoolRank,
    /// 贡献度
    pub contribution: i32,
}

impl MasterApprentice {
    /// 创建新的师徒关系
    pub fn new(master_id: String, apprentice_id: String, school_id: String) -> Self {
        Self {
            master_id,
            apprentice_id,
            school_id,
            dharma_name: None,
            since: chrono::Utc::now().timestamp(),
            rank: SchoolRank::Disciple,
            contribution: 0,
        }
    }
}

/// 师父NPC数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterData {
    /// NPC ID
    pub id: String,
    /// NPC名称
    pub name: String,
    /// 所属门派
    pub school_id: String,
    /// 师父等级
    pub rank: SchoolRank,
    /// 可教授的技能列表
    pub teachable_skills: Vec<TeachableSkill>,
    /// 收徒要求
    pub requirements: MasterRequirements,
    /// 位置
    pub location: String,
}

/// 可教授的技能
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeachableSkill {
    /// 技能ID
    pub skill_id: String,
    /// 需要的门派等级
    pub required_rank: SchoolRank,
    /// 需要的属性要求
    pub stat_requirements: Option<StatRequirements>,
    /// 需要的已学技能
    pub prerequisites: Vec<String>,
    /// 学习费用
    pub cost: u32,
}

/// 收徒要求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MasterRequirements {
    /// 最低等级要求
    pub min_level: Option<u32>,
    /// 属性要求
    pub stat_requirements: Option<StatRequirements>,
    /// 必须没有拜过其他师父
    pub exclusive: bool,
    /// 需要完成的任务
    pub quest_required: Option<String>,
}

/// 属性要求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatRequirements {
    pub gen: Option<i32>, // 根骨
    pub str: Option<i32>, // 臂力
    pub con: Option<i32>, // 体质
    pub dex: Option<i32>, // 灵巧
    pub int: Option<i32>, // 悟性
}

impl Default for StatRequirements {
    fn default() -> Self {
        Self {
            gen: None,
            str: None,
            con: None,
            dex: None,
            int: None,
        }
    }
}

/// 师父管理器
pub struct MasterDaemon {
    /// 所有师父NPC
    masters: HashMap<String, MasterData>,
    /// 所有师徒关系 (apprentice_id -> relationship)
    apprenticeships: HashMap<String, MasterApprentice>,
    /// 门派成员 (school_id -> Vec<apprentice_id>)
    school_members: HashMap<String, Vec<String>>,
}

impl MasterDaemon {
    /// 创建新的师父管理器
    pub fn new() -> Self {
        let mut daemon = Self {
            masters: HashMap::new(),
            apprenticeships: HashMap::new(),
            school_members: HashMap::new(),
        };

        daemon.init_default_masters();
        daemon
    }

    /// 初始化默认师父
    fn init_default_masters(&mut self) {
        // 武堂 - 堂主
        self.masters.insert(
            "master_wutang_leader".to_string(),
            MasterData {
                id: "master_wutang_leader".to_string(),
                name: "武堂堂主".to_string(),
                school_id: "wutang".to_string(),
                rank: SchoolRank::Master,
                teachable_skills: vec![
                    TeachableSkill {
                        skill_id: "skill_xionghuquan".to_string(),
                        required_rank: SchoolRank::Disciple,
                        stat_requirements: Some(StatRequirements {
                            gen: Some(20),
                            str: Some(25),
                            ..Default::default()
                        }),
                        prerequisites: vec![],
                        cost: 100,
                    },
                ],
                requirements: MasterRequirements {
                    min_level: Some(10),
                    stat_requirements: Some(StatRequirements {
                        str: Some(20),
                        ..Default::default()
                    }),
                    exclusive: true,
                    quest_required: None,
                },
                location: "wutang_hall".to_string(),
            },
        );

        // 武当 - 掌门
        self.masters.insert(
            "master_wudang_leader".to_string(),
            MasterData {
                id: "master_wudang_leader".to_string(),
                name: "武当掌门".to_string(),
                school_id: "wudang".to_string(),
                rank: SchoolRank::Master,
                teachable_skills: vec![
                    TeachableSkill {
                        skill_id: "skill_taiji".to_string(),
                        required_rank: SchoolRank::Disciple,
                        stat_requirements: Some(StatRequirements {
                            gen: Some(30),
                            str: Some(15),
                            dex: Some(20),
                            ..Default::default()
                        }),
                        prerequisites: vec![],
                        cost: 100,
                    },
                ],
                requirements: MasterRequirements {
                    min_level: Some(10),
                    stat_requirements: Some(StatRequirements {
                        gen: Some(25),
                        ..Default::default()
                    }),
                    exclusive: true,
                    quest_required: None,
                },
                location: "wudang_hall".to_string(),
            },
        );

        // 少林 - 方丈
        self.masters.insert(
            "master_shaolin_leader".to_string(),
            MasterData {
                id: "master_shaolin_leader".to_string(),
                name: "少林方丈".to_string(),
                school_id: "shaolin".to_string(),
                rank: SchoolRank::GrandMaster,
                teachable_skills: vec![
                    TeachableSkill {
                        skill_id: "skill_luohanquan".to_string(),
                        required_rank: SchoolRank::Disciple,
                        stat_requirements: Some(StatRequirements {
                            str: Some(30),
                            con: Some(20),
                            ..Default::default()
                        }),
                        prerequisites: vec![],
                        cost: 100,
                    },
                ],
                requirements: MasterRequirements {
                    min_level: Some(10),
                    stat_requirements: Some(StatRequirements {
                        str: Some(25),
                        con: Some(15),
                        ..Default::default()
                    }),
                    exclusive: true,
                    quest_required: None,
                },
                location: "shaolin_hall".to_string(),
            },
        );

        // 华山 - 掌门
        self.masters.insert(
            "master_huashan_leader".to_string(),
            MasterData {
                id: "master_huashan_leader".to_string(),
                name: "华山掌门".to_string(),
                school_id: "huashan".to_string(),
                rank: SchoolRank::Master,
                teachable_skills: vec![
                    TeachableSkill {
                        skill_id: "skill_dugujiujian".to_string(),
                        required_rank: SchoolRank::InnerDisciple,
                        stat_requirements: Some(StatRequirements {
                            gen: Some(35),
                            str: Some(20),
                            dex: Some(30),
                            int: Some(25),
                            ..Default::default()
                        }),
                        prerequisites: vec![],
                        cost: 500,
                    },
                ],
                requirements: MasterRequirements {
                    min_level: Some(20),
                    stat_requirements: Some(StatRequirements {
                        int: Some(30),
                        dex: Some(25),
                        ..Default::default()
                    }),
                    exclusive: true,
                    quest_required: None,
                },
                location: "huashan_hall".to_string(),
            },
        );
    }

    /// 获取师父
    pub fn get_master(&self, master_id: &str) -> Option<&MasterData> {
        self.masters.get(master_id)
    }

    /// 获取门派的所有师父
    pub fn get_school_masters(&self, school_id: &str) -> Vec<&MasterData> {
        self.masters
            .values()
            .filter(|m| m.school_id == school_id)
            .collect()
    }

    /// 检查玩家是否已经拜师
    pub fn has_master(&self, player_id: &str) -> bool {
        self.apprenticeships.contains_key(player_id)
    }

    /// 获取玩家的师徒关系
    pub fn get_apprenticeship(&self, player_id: &str) -> Option<&MasterApprentice> {
        self.apprenticeships.get(player_id)
    }

    /// 拜师 - 对应 txpike9 的 baishi 命令
    pub fn apprentice_to(
        &mut self,
        player_id: &str,
        master_id: &str,
        player_stats: &StatRequirements,
        player_level: u32,
    ) -> Result<String, String> {
        // 先提取师父信息，避免借用冲突
        let (school_id, master_name) = {
            let master = self.get_master(master_id)
                .ok_or_else(|| "找不到这位师父".to_string())?;
            (master.school_id.clone(), master.name.clone())
        };

        // 检查是否已经拜师
        if self.has_master(player_id) {
            return Err("你已经拜过师父了，不能重复拜师".to_string());
        }

        // 重新获取师父进行检查
        let master = self.get_master(master_id)
            .ok_or_else(|| "找不到这位师父".to_string())?;

        // 检查等级要求
        if let Some(min_level) = master.requirements.min_level {
            if player_level < min_level {
                return Err(format!("你的等级不足，需要达到{}级才能拜师", min_level));
            }
        }

        // 检查属性要求
        if let Some(req) = &master.requirements.stat_requirements {
            if let Some(req_gen) = req.gen {
                if player_stats.gen.unwrap_or(0) < req_gen {
                    return Err(format!("你的根骨不足，需要达到{}", req_gen));
                }
            }
            if let Some(req_str) = req.str {
                if player_stats.str.unwrap_or(0) < req_str {
                    return Err(format!("你的臂力不足，需要达到{}", req_str));
                }
            }
            if let Some(req_con) = req.con {
                if player_stats.con.unwrap_or(0) < req_con {
                    return Err(format!("你的体质不足，需要达到{}", req_con));
                }
            }
            if let Some(req_dex) = req.dex {
                if player_stats.dex.unwrap_or(0) < req_dex {
                    return Err(format!("你的灵巧不足，需要达到{}", req_dex));
                }
            }
            if let Some(req_int) = req.int {
                if player_stats.int.unwrap_or(0) < req_int {
                    return Err(format!("你的悟性不足，需要达到{}", req_int));
                }
            }
        }

        // 创建师徒关系
        let relationship = MasterApprentice::new(
            master_id.to_string(),
            player_id.to_string(),
            school_id.clone(),
        );

        self.apprenticeships.insert(player_id.to_string(), relationship);

        // 添加到门派成员
        self.school_members
            .entry(school_id.clone())
            .or_insert_with(Vec::new)
            .push(player_id.to_string());

        Ok(format!("你成功拜{}为师，成为{}{}", master_name, school_id, SchoolRank::Disciple.cn_name()))
    }

    /// 出师 - 离开门派
    pub fn leave_school(&mut self, player_id: &str) -> Result<String, String> {
        let relationship = self.apprenticeships.remove(player_id)
            .ok_or_else(|| "你还没有拜师".to_string())?;

        // 从门派成员中移除
        if let Some(members) = self.school_members.get_mut(&relationship.school_id) {
            members.retain(|id| id != player_id);
        }

        Ok("你已经离开门派，恢复自由身".to_string())
    }

    /// 获取门派的所有成员
    pub fn get_school_members(&self, school_id: &str) -> Vec<String> {
        self.school_members.get(school_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// 升级门派等级
    pub fn promote(&mut self, player_id: &str) -> Result<String, String> {
        let relationship = self.apprenticeships.get_mut(player_id)
            .ok_or_else(|| "你还没有拜师".to_string())?;

        if let Some(next_rank) = relationship.rank.next() {
            relationship.rank = next_rank;
            Ok(format!("恭喜你晋升为{}", next_rank.cn_name()))
        } else {
            Err("你已经是最高等级了".to_string())
        }
    }

    /// 检查是否可以学习某个技能
    pub fn can_learn_skill(
        &self,
        player_id: &str,
        skill_id: &str,
    ) -> Result<(), String> {
        let relationship = self.apprenticeships.get(player_id)
            .ok_or_else(|| "你还没有拜师，无法学习门派武功".to_string())?;

        // 获取师父
        let master = self.get_master(&relationship.master_id)
            .ok_or_else(|| "找不到你的师父".to_string())?;

        // 查找技能
        let teachable = master.teachable_skills
            .iter()
            .find(|s| s.skill_id == skill_id)
            .ok_or_else(|| "你的师父不会这门武功".to_string())?;

        // 检查等级要求
        if relationship.rank < teachable.required_rank {
            return Err(format!(
                "你需要达到{}才能学习这门武功",
                teachable.required_rank.cn_name()
            ));
        }

        Ok(())
    }
}

impl Default for MasterDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局师父管理器
pub static MASTERD: once_cell::sync::Lazy<Arc<RwLock<MasterDaemon>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(MasterDaemon::new())));

/// 获取师父管理器
pub fn get_masterd() -> Arc<RwLock<MasterDaemon>> {
    MASTERD.clone()
}
