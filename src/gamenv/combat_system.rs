// gamenv/combat_system.rs - 战斗系统
// 对应 txpike9 的战斗系统

use crate::gamenv::world::{Npc, NpcBehavior};
use crate::gamenv::player_state::{PlayerState, QuestProgress};
use crate::core::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// 战斗状态
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CombatStatus {
    /// 准备中
    Preparing,
    /// 战斗中
    Fighting,
    /// 玩家胜利
    PlayerWin,
    /// 怪物胜利
    MonsterWin,
    /// 逃跑
    Fled,
    /// 结束
    Ended,
}

/// 战斗回合结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatRound {
    /// 回合数
    pub round: u32,
    /// 玩家造成的伤害
    pub player_damage: u32,
    /// 怪物造成的伤害
    pub monster_damage: u32,
    /// 玩家当前HP
    pub player_hp: u32,
    /// 怪物当前HP
    pub monster_hp: i32,
    /// 战斗日志
    pub log: Vec<String>,
    /// 战斗是否结束
    pub ended: bool,
    /// 谁赢了
    pub winner: Option<CombatStatus>,
}

/// 战斗会话
#[derive(Clone, Debug)]
pub struct CombatSession {
    /// 怪物
    pub monster: Npc,
    /// 怪物当前HP
    pub monster_hp: i32,
    /// 战斗回合数
    pub round: u32,
    /// 战斗状态
    pub status: CombatStatus,
}

impl CombatSession {
    /// 创建新的战斗会话
    pub fn new(monster: Npc) -> Self {
        let monster_hp = monster.hp;
        Self {
            monster,
            monster_hp,
            round: 0,
            status: CombatStatus::Preparing,
        }
    }

    /// 执行一个战斗回合
    pub fn execute_round(&mut self, player: &PlayerState) -> CombatRound {
        self.round += 1;
        self.status = CombatStatus::Fighting;

        let mut log = Vec::new();
        let mut player_damage = 0;
        let mut monster_damage = 0;

        // 玩家攻击
        player_damage = self.calculate_player_damage(player);
        self.monster_hp -= player_damage as i32;
        log.push(format!("你攻击{}，造成了{}点伤害！", self.monster.short, player_damage));

        // 检查怪物是否死亡
        if self.monster_hp <= 0 {
            self.monster_hp = 0;
            self.status = CombatStatus::PlayerWin;
            log.push(format!("§G{}倒下了！§N", self.monster.short));
            log.push(format!("§Y战斗胜利！§N"));
            return CombatRound {
                round: self.round,
                player_damage,
                monster_damage: 0,
                player_hp: player.hp,
                monster_hp: 0,
                log,
                ended: true,
                winner: Some(CombatStatus::PlayerWin),
            };
        }

        // 怪物反击
        monster_damage = self.calculate_monster_damage(player);
        log.push(format!("{}攻击你，造成了{}点伤害！", self.monster.short, monster_damage));

        CombatRound {
            round: self.round,
            player_damage,
            monster_damage,
            player_hp: player.hp.saturating_sub(monster_damage),
            monster_hp: self.monster_hp,
            log,
            ended: false,
            winner: None,
        }
    }

    /// 计算玩家伤害
    fn calculate_player_damage(&self, player: &PlayerState) -> u32 {
        let mut rng = rand::thread_rng();

        // 基础伤害 = 玩家攻击
        let mut damage = player.attack as i32;

        // 暴击判定 (10%几率)
        let is_crit = rng.gen_range(0..100) < 10;
        if is_crit {
            damage = (damage as f32 * 1.5) as i32;
        }

        // 减去怪物防御
        damage = (damage - (self.monster.defense / 2)).max(1);

        // 随机波动 +/- 20%
        let variance = (damage as f32 * 0.2) as i32;
        damage = damage + rng.gen_range(-variance..=variance);

        damage.max(1) as u32
    }

    /// 计算怪物伤害
    fn calculate_monster_damage(&self, player: &PlayerState) -> u32 {
        let mut rng = rand::thread_rng();

        // 基础伤害 = 怪物攻击
        let mut damage = self.monster.attack;

        // 减去玩家防御
        damage = (damage - (player.defense as i32 / 2)).max(1);

        // 随机波动 +/- 20%
        let variance = (damage as f32 * 0.2) as i32;
        damage = damage + rng.gen_range(-variance..=variance);

        damage.max(1) as u32
    }

    /// 玩家逃跑
    pub fn flee(&mut self, player: &PlayerState) -> CombatRound {
        let mut rng = rand::thread_rng();

        // 逃跑成功率 50%
        let success = rng.gen_range(0..100) < 50;

        if success {
            self.status = CombatStatus::Fled;
            CombatRound {
                round: self.round,
                player_damage: 0,
                monster_damage: 0,
                player_hp: player.hp,
                monster_hp: self.monster_hp,
                log: vec!["你成功逃脱了！".to_string()],
                ended: true,
                winner: Some(CombatStatus::Fled),
            }
        } else {
            // 逃跑失败，怪物会攻击
            let damage = self.calculate_monster_damage(player);
            CombatRound {
                round: self.round,
                player_damage: 0,
                monster_damage: damage,
                player_hp: player.hp.saturating_sub(damage),
                monster_hp: self.monster_hp,
                log: vec![
                    "你试图逃跑，但失败了！".to_string(),
                    format!("{}攻击你，造成了{}点伤害！", self.monster.short, damage),
                ],
                ended: false,
                winner: None,
            }
        }
    }

    /// 检查战斗是否结束
    pub fn is_ended(&self) -> bool {
        matches!(
            self.status,
            CombatStatus::PlayerWin | CombatStatus::MonsterWin | CombatStatus::Fled | CombatStatus::Ended
        )
    }

    /// 获取战斗奖励
    pub fn get_rewards(&self) -> CombatRewards {
        CombatRewards {
            exp: self.monster.exp as u64,
            gold: self.monster.gold as u64,
            loot: self.monster.calculate_loot(),
        }
    }
}

/// 战斗奖励
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatRewards {
    /// 获得的经验
    pub exp: u64,
    /// 获得的金币
    pub gold: u64,
    /// 掉落物品 (item_id, count)
    pub loot: Vec<(String, i32)>,
}

/// 战斗系统
pub struct CombatSystem;

impl CombatSystem {
    /// 开始战斗
    pub fn start_combat(monster: Npc) -> CombatSession {
        CombatSession::new(monster)
    }

    /// 执行战斗回合
    pub fn execute_round(session: &mut CombatSession, player: &PlayerState) -> CombatRound {
        session.execute_round(player)
    }

    /// 玩家尝试逃跑
    pub fn flee(session: &mut CombatSession, player: &PlayerState) -> CombatRound {
        session.flee(player)
    }

    /// 应用战斗结果到玩家状态
    pub fn apply_rewards(player: &mut PlayerState, rewards: &CombatRewards) -> String {
        let mut messages = vec![];

        // 添加经验
        let leveled_up = player.add_exp(rewards.exp);
        messages.push(format!("§Y获得了 {} 点经验！§N", rewards.exp));

        if leveled_up {
            messages.push(format!("§G恭喜！你升到了 {} 级！§N", player.level));
            messages.push(format!("HP提升到 {}，MP提升到 {}，攻击提升到 {}，防御提升到 {}",
                player.hp_max, player.mp_max, player.attack, player.defense));
        }

        // 添加金币
        player.add_gold(rewards.gold);
        messages.push(format!("§Y获得了 {} 金币！§N", rewards.gold));

        // 添加物品
        for (item_id, count) in &rewards.loot {
            player.inventory.insert(item_id.clone(), (item_id.clone(), *count, true));
            messages.push(format!("§Y获得了 {} x{}！§N", item_id, count));
        }

        messages.join("\n")
    }

    /// 检查是否可以战斗
    pub fn can_fight(player: &PlayerState, monster: &Npc) -> bool {
        if player.is_dead() {
            return false;
        }
        if monster.is_dead() {
            return false;
        }
        true
    }

    /// 计算怪物强度描述
    pub fn describe_monster_difficulty(player_level: u32, monster_level: i32) -> &'static str {
        let diff = monster_level as i32 - player_level as i32;
        match diff {
            d if d >= 10 => "§R[极度危险]§N",
            d if d >= 5 => "§R[非常危险]§N",
            d if d >= 2 => "§Y[危险]§N",
            d if d <= -5 => "§C[简单]§N",
            d if d <= -2 => "§H[较简单]§N",
            _ => "§N[相当]§N",
        }
    }

    /// 格式化战斗回合输出
    pub fn format_round(round: &CombatRound) -> String {
        let mut output = format!("§H--- 第 {} 回合 ---§N\n", round.round);
        for log in &round.log {
            output.push_str(log);
            output.push('\n');
        }
        output.push_str(&format!(
            "§H你的HP: {}  {}HP: {}§N",
            round.player_hp,
            if round.monster_hp > 0 { "怪物" } else { "怪物(已死)" },
            round.monster_hp
        ));
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_session_creation() {
        let monster = Npc {
            id: "test_monster".to_string(),
            name: "测试怪物".to_string(),
            short: "一只测试怪物".to_string(),
            long: "这是一个测试怪物".to_string(),
            level: 5,
            hp: 100,
            hp_max: 100,
            mp: 0,
            mp_max: 0,
            attack: 20,
            defense: 10,
            exp: 50,
            gold: 10,
            behavior: NpcBehavior::Aggressive,
            dialogs: vec![],
            shop: None,
            loot: vec![],
        };

        let session = CombatSession::new(monster);
        assert_eq!(session.monster_hp, 100);
        assert_eq!(session.round, 0);
        assert_eq!(session.status, CombatStatus::Preparing);
    }
}
