// gamenv/single/daemons/schoold.rs - 门派系统守护进程
// 对应 txpike9/pikenv/mudlib/single/schoold.pike

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 门派
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct School {
    /// 门派ID
    pub id: String,
    /// 门派名称（拼音）
    pub name: String,
    /// 门派名称（中文）
    pub name_cn: String,
    /// 门派描述
    pub description: String,
    /// 门派技能ID列表
    #[serde(default)]
    pub skills: Vec<String>,
}

impl School {
    /// 格式化门派信息
    pub fn format(&self) -> String {
        format!("§Y{}§N - {}\n", self.name_cn, self.description)
    }
}

/// 门派守护进程
pub struct SchoolDaemon {
    /// 所有门派
    schools: HashMap<String, School>,
}

impl SchoolDaemon {
    /// 创建新的门派守护进程
    pub fn new() -> Self {
        let mut daemon = Self {
            schools: HashMap::new(),
        };

        daemon.init_default_schools();
        daemon
    }

    /// 初始化默认门派
    fn init_default_schools(&mut self) {
        // 武堂
        let wutang = School {
            id: "wutang".to_string(),
            name: "wutang".to_string(),
            name_cn: "武堂".to_string(),
            description: "游戏中比较特殊的门派。武功特点为化繁为简，招式虽少，但威力惊人，武功易学易用。不管刀剑都采其别门派的绝学优点汇聚而成。".to_string(),
            skills: vec!["skill_basic_attack".to_string(), "skill_crit_training".to_string()],
        };

        // 武当
        let wudang = School {
            id: "wudang".to_string(),
            name: "wudang".to_string(),
            name_cn: "武当".to_string(),
            description: "素有\"北宗少林，南尊武当\"之称的武当门派由天涯剑客丰所创，武功讲求以柔克刚，借力打力。特点是形神合一，用意不用力，圆转贯串，滔滔不绝，为内家功之鼻祖。".to_string(),
            skills: vec!["skill_heal".to_string()],
        };

        // 少林
        let shaolin = School {
            id: "shaolin".to_string(),
            name: "shaolin".to_string(),
            name_cn: "少林".to_string(),
            description: "少林武功博大精深，名显于世。三十六路拳脚十八般兵器及各式各样的武学密籍，易筋无尽经书、洗髓经、72绝技更乃世间奇功。".to_string(),
            skills: vec!["skill_power_defense".to_string()],
        };

        // 天鹏
        let tianpeng = School {
            id: "tianpeng".to_string(),
            name: "tianpeng".to_string(),
            name_cn: "天鹏".to_string(),
            description: "天鹏门善用剑法，门派依靠绝学天鹏剑法、金鹏剑法震慑武林。此剑法诡奇狠辣，难躲难防，如松之劲，如风之迅。".to_string(),
            skills: vec![],
        };

        // 游龙
        let youlong = School {
            id: "youlong".to_string(),
            name: "youlong".to_string(),
            name_cn: "游龙".to_string(),
            description: "五行龙派以刀法闻名于世，出手诡异，变化多端，刚柔相济，浑厚苍劲。前期修炼突飞猛进。".to_string(),
            skills: vec![],
        };

        // 凝凤
        let ningfeng = School {
            id: "ningfeng".to_string(),
            name: "ningfeng".to_string(),
            name_cn: "凝凤".to_string(),
            description: "凝凤以剑法著称，门派绝学凝凤剑术与傲凰剑决，此派前期武功威力比较明显，但后期提高较慢，剑法妙招纷着，层出不穷。".to_string(),
            skills: vec![],
        };

        // 灵鹫
        let lingjiu = School {
            id: "lingjiu".to_string(),
            name: "lingjiu".to_string(),
            name_cn: "灵鹫".to_string(),
            description: "灵鹫庙位于天山飘渺峰上，是武林之中的一大神秘帮派。由于灵鹫庙弟子行事隐秘，武功高强，招式狠毒。最为厉害的是八荒六合唯我独尊功。".to_string(),
            skills: vec![],
        };

        // 星影宗（星宿）
        let xingxiu = School {
            id: "xingxiu".to_string(),
            name: "xingxiu".to_string(),
            name_cn: "星影宗".to_string(),
            description: "星影宗的开山祖师丁宏伟不同于道家的炼丹而是一生精于炼毒，任何功夫上都带有剧毒。主要武功有化功无尽大法、摘星功、吸髓掌、三阴蜈蚣无影爪等。".to_string(),
            skills: vec![],
        };

        // 华山（雄风宗）
        let huashan = School {
            id: "huashan".to_string(),
            name: "huashan".to_string(),
            name_cn: "雄风宗".to_string(),
            description: "原为五岳剑派中势力最强的雄风宗武功博大精深，历史源远流长。主要武功有华山飞瀑剑法、华山身法、三仙剑等，更有独影十八剑。".to_string(),
            skills: vec!["skill_fireball".to_string()],
        };

        self.schools.insert(wutang.id.clone(), wutang);
        self.schools.insert(wudang.id.clone(), wudang);
        self.schools.insert(shaolin.id.clone(), shaolin);
        self.schools.insert(tianpeng.id.clone(), tianpeng);
        self.schools.insert(youlong.id.clone(), youlong);
        self.schools.insert(ningfeng.id.clone(), ningfeng);
        self.schools.insert(lingjiu.id.clone(), lingjiu);
        self.schools.insert(xingxiu.id.clone(), xingxiu);
        self.schools.insert(huashan.id.clone(), huashan);
    }

    /// 获取门派
    pub fn get_school(&self, school_id: &str) -> Option<&School> {
        self.schools.get(school_id)
    }

    /// 获取所有门派
    pub fn get_all_schools(&self) -> Vec<&School> {
        let mut schools: Vec<_> = self.schools.values().collect();
        schools.sort_by(|a, b| a.name_cn.cmp(&b.name_cn));
        schools
    }

    /// 获取技能（委托给skilld）
    pub fn get_skill(&self, skill_id: &str) -> Option<crate::gamenv::single::skilld::Skill> {
        // 获取技能守护进程
        let skilld = crate::gamenv::single::skilld::get_skilld().blocking_read();
        skilld.get_skill(skill_id).cloned()
    }
}

impl Default for SchoolDaemon {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局门派守护进程
pub static SCHOOLD: std::sync::OnceLock<RwLock<SchoolDaemon>> = std::sync::OnceLock::new();

/// 获取门派守护进程
pub fn get_schoold() -> &'static RwLock<SchoolDaemon> {
    SCHOOLD.get_or_init(|| RwLock::new(SchoolDaemon::default()))
}
