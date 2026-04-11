// gamenv/entities/character.rs - 角色基类
// 对应 txpike9/wapmud2/inherit/feature/char.pike

use serde::{Deserialize, Serialize};

/// 角色基类 - 包含所有角色共有的属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// 角色ID
    pub id: String,
    /// 角色名
    pub name: String,
    /// 中文名
    pub name_cn: String,
    /// 等级
    pub level: u32,
    /// 经验值
    pub exp: u64,
    /// 生命值
    pub hp: i32,
    /// 最大生命值
    pub hp_max: i32,
    /// 内力
    pub qi: i32,
    /// 最大内力
    pub qi_max: i32,
    /// 攻击力
    pub attack: i32,
    /// 防御力
    pub defense: i32,
    /// 速度
    pub speed: i32,
    /// 创建时间
    pub created_at: i64,
}

impl Character {
    /// 创建新角色
    pub fn new(id: String, name: String, name_cn: String) -> Self {
        Self {
            id,
            name,
            name_cn,
            level: 1,
            exp: 0,
            hp: 100,
            hp_max: 100,
            qi: 50,
            qi_max: 50,
            attack: 10,
            defense: 5,
            speed: 10,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// 获取生命值百分比
    pub fn hp_percent(&self) -> i32 {
        if self.hp_max == 0 { return 0; }
        (self.hp * 100 / self.hp_max).max(0).min(100)
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// 受到伤害
    pub fn take_damage(&mut self, damage: i32) -> bool {
        self.hp = (self.hp - damage).max(0);
        self.hp == 0
    }

    /// 恢复生命
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    /// 获取战斗属性（用于PK战斗）
    pub fn to_combat_stats(&self) -> crate::gamenv::single::daemons::pkd::CombatStats {
        crate::gamenv::single::daemons::pkd::CombatStats {
            id: self.id.clone(),
            name: self.name.clone(),
            name_cn: self.name_cn.clone(),
            level: self.level,
            hp: self.hp,
            hp_max: self.hp_max,
            qi: self.qi,
            qi_max: self.qi_max,
            attack: self.attack,
            defense: self.defense,
            speed: self.speed,
        }
    }

    /// 从战斗属性更新角色状态
    pub fn update_from_combat_stats(&mut self, stats: &crate::gamenv::single::daemons::pkd::CombatStats) {
        self.hp = stats.hp;
        self.hp_max = stats.hp_max;
        self.qi = stats.qi;
        self.qi_max = stats.qi_max;
    }
}
