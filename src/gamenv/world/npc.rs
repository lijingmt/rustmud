// gamenv/world/npc.rs - NPC系统

use serde::{Deserialize, Serialize};

/// NPC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Npc {
    /// NPC ID
    pub id: String,
    /// 名称
    pub name: String,
    /// 短描述
    pub short: String,
    /// 长描述
    pub long: String,
    /// 等级
    pub level: i32,
    /// 当前HP
    pub hp: i32,
    /// 最大HP
    pub hp_max: i32,
    /// 当前MP
    pub mp: i32,
    /// 最大MP
    pub mp_max: i32,
    /// 攻击力
    pub attack: i32,
    /// 防御力
    pub defense: i32,
    /// 给予经验
    pub exp: i32,
    /// 携带金币
    pub gold: i32,
    /// 行为模式
    #[serde(default)]
    pub behavior: NpcBehavior,
    /// 对话树
    #[serde(default)]
    pub dialogs: Vec<DialogNode>,
    /// 关联商店ID
    pub shop: Option<String>,
    /// 掉落物品
    #[serde(default)]
    pub loot: Vec<LootDrop>,
}

/// NPC行为模式
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NpcBehavior {
    /// 被动 - 不主动攻击
    Passive,
    /// 主动攻击 - 见到玩家就攻击
    Aggressive,
    /// 守护 - 保护特定区域
    Guard,
    /// 商人
    Merchant,
    /// 训练师
    Trainer,
}

impl Default for NpcBehavior {
    fn default() -> Self {
        NpcBehavior::Passive
    }
}

/// 对话节点
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogNode {
    /// 节点ID
    pub id: String,
    /// 对话文本
    pub text: String,
    /// 选项列表
    #[serde(default)]
    pub options: Vec<DialogOption>,
    /// 触发动作
    pub action: Option<DialogAction>,
}

/// 对话选项
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DialogOption {
    /// 选项文本
    pub text: String,
    /// 下一个节点ID
    pub next: String,
}

/// 对话动作
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DialogAction {
    /// 给予任务
    GiveQuest {
        quest_id: String,
        target: String,
        count: i32,
        reward_exp: i32,
        reward_gold: i32,
    },
    /// 完成任务
    CompleteQuest {
        quest_id: String,
    },
    /// 打开商店
    OpenShop {
        shop_id: String,
    },
    /// 传送到地图
    Teleport {
        room_id: String,
    },
    /// 给予物品
    GiveItem {
        item_id: String,
        count: i32,
    },
    /// 学习技能
    TeachSkill {
        skill_id: String,
    },
    /// 治疗
    Heal,
}

/// 掉落物品
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LootDrop {
    /// 物品ID
    pub item_id: String,
    /// 掉落概率 (0-100)
    pub chance: i32,
    /// 数量范围 (最小, 最大)
    pub count: (i32, i32),
}

impl Npc {
    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// 是否死亡
    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    /// 受到伤害
    pub fn take_damage(&mut self, damage: i32) -> i32 {
        let actual_damage = damage.max(0);
        self.hp = (self.hp - actual_damage).max(0);
        actual_damage
    }

    /// 治疗
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    /// 获取对话节点
    pub fn get_dialog(&self, node_id: &str) -> Option<&DialogNode> {
        self.dialogs.iter().find(|d| d.id == node_id)
    }

    /// 是否有商店
    pub fn has_shop(&self) -> bool {
        self.shop.is_some()
    }

    /// 是否是怪物
    pub fn is_monster(&self) -> bool {
        matches!(self.behavior, NpcBehavior::Aggressive)
    }

    /// 计算掉落
    pub fn calculate_loot(&self) -> Vec<(String, i32)> {
        let mut result = vec![];
        let mut rng = rand::thread_rng();

        for loot in &self.loot {
            if rand::Rng::gen_range(&mut rng, 0..100) < loot.chance {
                let count = rand::Rng::gen_range(&mut rng, loot.count.0..=loot.count.1);
                result.push((loot.item_id.clone(), count));
            }
        }

        result
    }

    /// 格式化短描述
    pub fn format_short(&self) -> String {
        format!("[{}{}级] {}", if self.is_monster() { "★" } else { "" }, self.level, self.short)
    }

    /// 格式化状态
    pub fn format_status(&self) -> String {
        format!(
            "{} - HP:{}/{} MP:{}/{} 攻击:{} 防御:{}",
            self.name, self.hp, self.hp_max, self.mp, self.mp_max, self.attack, self.defense
        )
    }
}

use rand;
