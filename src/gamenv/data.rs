// gamenv/data.rs - 数据定义
// 对应 txpike9/gamenv/data/ 目录

/// 技能数据
pub mod skill {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Skill {
        pub id: String,
        pub name: String,
        pub name_cn: String,
        pub max_level: u32,
    }
}

/// 门派数据
pub mod school {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct School {
        pub id: String,
        pub name: String,
        pub name_cn: String,
    }
}
