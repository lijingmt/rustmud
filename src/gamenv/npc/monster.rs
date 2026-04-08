// gamenv/npc/monster.rs - 怪物系统
// 对应 txpike9/gamenv/clone/npc/ 中的怪物

use crate::core::*;
use crate::gamenv::npc::npc::{Npc, NpcType};
use crate::gamenv::combat::CombatStats;
use serde::{Deserialize, Serialize};
use rand::Rng;

/// 怪物稀有度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MonsterRarity {
    /// 普通
    Common = 0,
    /// 精英
    Elite = 1,
    /// 稀有
    Rare = 2,
    /// Boss
    Boss = 3,
}

impl MonsterRarity {
    /// 获取稀有度名称
    pub fn name(&self) -> &str {
        match self {
            MonsterRarity::Common => "",
            MonsterRarity::Elite => "精英",
            MonsterRarity::Rare => "稀有",
            MonsterRarity::Boss => "首领",
        }
    }

    /// 获取稀有度颜色代码
    pub fn color_code(&self) -> &str {
        match self {
            MonsterRarity::Common => "§w",
            MonsterRarity::Elite => "§g",
            MonsterRarity::Rare => "§b",
            MonsterRarity::Boss => "§o",
        }
    }

    /// 属性倍率
    pub fn stat_multiplier(&self) -> f32 {
        match self {
            MonsterRarity::Common => 1.0,
            MonsterRarity::Elite => 2.0,
            MonsterRarity::Rare => 5.0,
            MonsterRarity::Boss => 10.0,
        }
    }
}

/// 怪物 (继承自NPC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    /// 基础NPC数据
    #[serde(flatten)]
    pub npc: Npc,
    /// 怪物稀有度
    pub rarity: MonsterRarity,
    /// 击杀奖励 (修为)
    pub exp_reward: u32,
    /// 击杀奖励 (潜能)
    pub potential_reward: u32,
    /// 掉落列表
    pub drops: Vec<String>,
    /// 掉落几率 (0-10000)
    pub drop_rate: u32,
}

impl Monster {
    /// 创建新怪物
    pub fn new(name: String, name_cn: String, level: u32) -> Self {
        let mut npc = Npc::new(name.clone(), name_cn.clone(), NpcType::Monster, level);
        npc.base.attackable = true;

        Self {
            npc,
            rarity: MonsterRarity::Common,
            exp_reward: level * level,
            potential_reward: level,
            drops: Vec::new(),
            drop_rate: 1000, // 默认10%掉落率
        }
    }

    /// 设置稀有度
    pub fn with_rarity(mut self, rarity: MonsterRarity) -> Self {
        self.rarity = rarity;
        let multiplier = rarity.stat_multiplier();

        // 根据稀有度调整属性
        self.npc.combat.max_hp = (self.npc.combat.max_hp as f32 * multiplier) as u32;
        self.npc.combat.hp = self.npc.combat.max_hp;
        self.npc.combat.attack = (self.npc.combat.attack as f32 * multiplier) as u32;
        self.npc.combat.defense = (self.npc.combat.defense as f32 * multiplier) as u32;

        // 调整奖励
        self.exp_reward = (self.exp_reward as f32 * multiplier) as u32;
        self.potential_reward = (self.potential_reward as f32 * multiplier) as u32;

        self
    }

    /// 设置掉落
    pub fn with_drops(mut self, drops: Vec<String>, drop_rate: u32) -> Self {
        self.drops = drops;
        self.drop_rate = drop_rate;
        self
    }

    /// 设置主动攻击
    pub fn with_aggressive(mut self, aggressive: bool) -> Self {
        self.npc.base.aggressive = aggressive;
        self
    }

    /// 渲染怪物信息
    pub fn render_info(&self) -> String {
        let mut info = format!("{}{}{}§r ",
            self.rarity.color_code(),
            self.rarity.name(),
            self.npc.base.name_cn
        );
        info.push_str(&format!("(Lv{})\n", self.npc.base.level));

        if self.npc.combat.hp < self.npc.combat.max_hp {
            info.push_str(&format!("生命: {}/{}\n", self.npc.combat.hp, self.npc.combat.max_hp));
        }

        info
    }

    /// 计算掉落
    pub fn calculate_drops(&self) -> Option<String> {
        if self.drops.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..10000);

        if roll < self.drop_rate {
            let idx = rng.gen_range(0..self.drops.len());
            Some(self.drops[idx].clone())
        } else {
            None
        }
    }
}

/// 预设怪物列表
pub fn create_preset_monsters() -> Vec<Monster> {
    vec![
        // 新手村怪物
        Monster::new(
            "monster_rat".to_string(),
            "老鼠".to_string(),
            1,
        )
        .with_rarity(MonsterRarity::Common),

        Monster::new(
            "monster_snake".to_string(),
            "毒蛇".to_string(),
            3,
        )
        .with_rarity(MonsterRarity::Common),

        Monster::new(
            "monster_wolf".to_string(),
            "野狼".to_string(),
            5,
        )
        .with_rarity(MonsterRarity::Common)
        .with_aggressive(true),

        // 北京城周边怪物
        Monster::new(
            "monster_bandit".to_string(),
            "强盗".to_string(),
            10,
        )
        .with_rarity(MonsterRarity::Common)
        .with_aggressive(true),

        Monster::new(
            "monster_bandit_leader".to_string(),
            "强盗头目".to_string(),
            20,
        )
        .with_rarity(MonsterRarity::Elite)
        .with_aggressive(true)
        .with_drops(vec![
            "item/iron_sword".to_string(),
            "item/leather_armor".to_string(),
        ], 3000),

        // 副本怪物
        Monster::new(
            "monster_dungeon_boss".to_string(),
            "守护兽".to_string(),
            50,
        )
        .with_rarity(MonsterRarity::Boss)
        .with_drops(vec![
            "item/epic_weapon".to_string(),
            "item/boss_armor".to_string(),
        ], 10000),
    ]
}

/// 根据等级生成合适的怪物
pub fn spawn_monster_for_level(level: u32, area: &str) -> Option<Monster> {
    let presets = create_preset_monsters();

    // 找到等级相近的怪物
    let suitable: Vec<_> = presets.iter()
        .filter(|m| {
            let level_diff = if m.npc.base.level > level {
                m.npc.base.level - level
            } else {
                level - m.npc.base.level
            };
            level_diff <= 10 // 等级差不超过10
        })
        .collect();

    if suitable.is_empty() {
        return None;
    }

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..suitable.len());
    Some(suitable[idx].clone())
}
