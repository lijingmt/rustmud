// gamenv/single/daemons/pkd.rs - PK战斗守护进程
// 1:1 复刻自 txpike9/pikenv/wapmud2/inherit/feature/fight.pike

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// PK模式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PkMode {
    Peace,      // 和平模式 - 不能攻击其他玩家
    Free,       // 自由模式 - 可以攻击任何人（除了和平模式）
    Team,       // 组队模式 - 只能攻击敌方队伍成员
    Guild,      // 帮派模式 - 只能攻击敌方帮派成员
}

impl Default for PkMode {
    fn default() -> Self {
        PkMode::Peace
    }
}

impl PkMode {
    pub fn as_str(&self) -> &str {
        match self {
            PkMode::Peace => "和平模式",
            PkMode::Free => "自由模式",
            PkMode::Team => "组队模式",
            PkMode::Guild => "帮派模式",
        }
    }

    pub fn can_attack(self, other_mode: PkMode) -> bool {
        match (self, other_mode) {
            // 和平模式不能主动攻击
            (PkMode::Peace, _) => false,
            // 和平模式的人可以被自由/组队/帮派模式攻击
            (_, PkMode::Peace) => false,
            // 自由模式可以攻击任何非和平模式
            (PkMode::Free, _) => other_mode != PkMode::Peace,
            // 其他模式需要进一步判断
            _ => true,
        }
    }
}

/// PK值等级
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PkLevel {
    Citizen = 0,     // 良民 §G
    Gray = 1,        // 灰名 §C (1-19)
    Red = 20,        // 红名 §Y (20-49)
    Evil = 50,       // 恶人 §R (50-99)
    Demon = 100,     // 恶魔 §X (100+)
}

impl PkLevel {
    pub fn from_value(value: i32) -> Self {
        if value < 1 {
            PkLevel::Citizen
        } else if value < 20 {
            PkLevel::Gray
        } else if value < 50 {
            PkLevel::Red
        } else if value < 100 {
            PkLevel::Evil
        } else {
            PkLevel::Demon
        }
    }

    pub fn color_code(&self) -> &str {
        match self {
            PkLevel::Citizen => "#00ff00",  // §G
            PkLevel::Gray => "#cccccc",      // §C
            PkLevel::Red => "#ffff00",       // §Y
            PkLevel::Evil => "#ff0000",      // §R
            PkLevel::Demon => "#ff00ff",     // §X
        }
    }

    pub fn name(&self) -> &str {
        match self {
            PkLevel::Citizen => "良民",
            PkLevel::Gray => "灰名",
            PkLevel::Red => "红名",
            PkLevel::Evil => "恶人",
            PkLevel::Demon => "恶魔",
        }
    }
}

/// 战斗状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CombatStatus {
    Normal,      // 正常
    Fighting,    // 战斗中
    Escaped,     // 逃跑
    Dead,        // 死亡
    Unconscious, // 昏迷
}

/// 战斗动作
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CombatAction {
    Attack,
    Escape,
    Perform(String),
    Cast(String),
    Surrender,
}

/// 战斗者数据
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatStats {
    pub id: String,
    pub name: String,
    pub name_cn: String,
    pub level: i32,

    // 战斗属性
    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
    pub jing: i32,       // 精力
    pub jing_max: i32,
    pub qi: i32,         // 内力
    pub qi_max: i32,

    // 战斗技能
    pub attack: i32,     // 攻击力
    pub defense: i32,    // 防御力
    pub dodge: i32,      // 轻功
    pub parry: i32,      // 招架

    // 状态
    pub pk_mode: PkMode,
    pub pk_value: i32,
    pub kill_streak: i32,
    pub is_killing: bool,  // 是否想杀死对方
}

impl CombatStats {
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn hp_percent(&self) -> i32 {
        if self.hp_max == 0 {
            0
        } else {
            (self.hp * 100 / self.hp_max).max(0).min(100)
        }
    }

    /// 检查是否可以攻击对方
    pub fn can_attack(&self, target: &CombatStats) -> Result<(), String> {
        // 检查是否在和平区域
        // TODO: 检查房间是否和平

        // 检查PK模式
        if !self.pk_mode.can_attack(target.pk_mode) {
            return Err(format!("对方的PK模式不允许被攻击"));
        }

        // 检查是否已经在战斗中
        // TODO: 检查战斗状态

        Ok(())
    }
}

/// 战斗回合结果
#[derive(Clone, Debug)]
pub struct CombatRound {
    pub round_number: u32,
    pub attacker_damage: i32,
    pub defender_damage: i32,
    pub attacker_hp: i32,
    pub defender_hp: i32,
    pub log: Vec<String>,
    pub ended: bool,
    pub winner: Option<String>,
}

/// PK战斗会话
#[derive(Clone, Debug)]
pub struct PkBattle {
    pub battle_id: String,
    pub challenger: CombatStats,
    pub defender: CombatStats,
    pub round: u32,
    pub total_damage_dealt: i32,
    pub total_damage_taken: i32,
    pub start_time: i64,
    pub status: CombatStatus,
    pub combat_log: Vec<String>,
}

impl PkBattle {
    pub fn new(challenger: CombatStats, defender: CombatStats) -> Self {
        let battle_id = format!("pk_{}_{}", challenger.id, defender.id);
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            battle_id,
            challenger,
            defender,
            round: 0,
            total_damage_dealt: 0,
            total_damage_taken: 0,
            start_time,
            status: CombatStatus::Fighting,
            combat_log: vec![],
        }
    }

    /// 计算伤害
    pub fn calculate_damage(&self, attacker: &CombatStats, defender: &CombatStats) -> i32 {
        // 基础伤害 = 攻击 - 防御 (最低1)
        let base_damage = (attacker.attack - defender.defense / 2).max(1);

        // 命中检查 (轻功闪避)
        let dodge_chance = defender.dodge as f64 / (attacker.dodge + defender.dodge) as f64;
        if rand::random::<f64>() < dodge_chance {
            return 0; // 被闪避
        }

        // 招架检查
        let parry_chance = defender.parry as f64 / (attacker.attack + defender.parry) as f64;
        let mut damage = base_damage;
        if rand::random::<f64>() < parry_chance {
            damage = base_damage / 2; // 招架减半
        }

        // 暴击检查
        let crit_chance = 0.1; // 10%暴击
        if rand::random::<f64>() < crit_chance {
            damage = (damage as f64 * 1.5) as i32;
        }

        // +/- 10% 随机浮动
        let variance = (damage as f32 * 0.1) as i32;
        damage = damage + rand::random::<i32>().rem_euclid(variance * 2 + 1) - variance;

        damage.max(1)
    }

    /// 执行一个战斗回合
    pub fn execute_round(&mut self) -> CombatRound {
        self.round += 1;

        let mut attacker_damage = 0;
        let mut defender_damage = 0;
        let mut log = vec![];

        // 挑战者攻击
        if self.challenger.is_alive() {
            attacker_damage = self.calculate_damage(&self.challenger, &self.defender);

            if attacker_damage > 0 {
                log.push(format!(
                    "§Y{}§N对§R{}§N造成§R{}§N点伤害！",
                    self.challenger.name_cn,
                    self.defender.name_cn,
                    attacker_damage
                ));
            } else {
                log.push(format!(
                    "§Y{}§N的攻击被§R{}§N闪避了！",
                    self.challenger.name_cn,
                    self.defender.name_cn
                ));
            }

            self.defender.hp = (self.defender.hp - attacker_damage).max(0);
            self.total_damage_dealt += attacker_damage;
        }

        // 防守者反击（如果还活着）
        if self.defender.is_alive() && self.defender.hp > 0 {
            defender_damage = self.calculate_damage(&self.defender, &self.challenger);

            if defender_damage > 0 {
                log.push(format!(
                    "§R{}§N对§Y{}§N造成§R{}§N点伤害！",
                    self.defender.name_cn,
                    self.challenger.name_cn,
                    defender_damage
                ));
            } else {
                log.push(format!(
                    "§R{}§N的攻击被§Y{}§N闪避了！",
                    self.defender.name_cn,
                    self.challenger.name_cn
                ));
            }

            self.challenger.hp = (self.challenger.hp - defender_damage).max(0);
            self.total_damage_taken += defender_damage;
        }

        // 检查战斗是否结束
        let ended = !self.challenger.is_alive() || !self.defender.is_alive();
        let winner = if ended {
            if !self.challenger.is_alive() && !self.defender.is_alive() {
                None // 平局
            } else if !self.defender.is_alive() {
                Some(self.challenger.id.clone())
            } else {
                Some(self.defender.id.clone())
            }
        } else {
            None
        };

        if winner.is_some() {
            self.status = CombatStatus::Dead;
        }

        self.combat_log.extend(log.clone());

        CombatRound {
            round_number: self.round,
            attacker_damage,
            defender_damage,
            attacker_hp: self.challenger.hp,
            defender_hp: self.defender.hp,
            log,
            ended,
            winner,
        }
    }

    /// 生成战斗结束信息
    pub fn generate_result(&self) -> String {
        let winner = if !self.challenger.is_alive() && !self.defender.is_alive() {
            "平局"
        } else if !self.defender.is_alive() {
            &self.challenger.name_cn
        } else {
            &self.defender.name_cn
        };

        let mut output = String::new();
        output.push_str(&format!("§C========== PK战斗结束 =========§N\n"));
        output.push_str(&format!("§Y胜利者: {}§N\n", winner));
        output.push_str(&format!("战斗回合: {}\n", self.round));
        output.push_str(&format!("战斗时长: {}秒\n",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64 - self.start_time
        ));
        output.push_str(&format!("\n§H【返回】§N\n[返回房间:look]\n"));

        output
    }

    /// 生成战斗状态（用于前端显示）
    pub fn generate_status(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("§C========== PK战斗进行中 =========§N\n"));
        output.push_str(&format!("§Y回合: {}§N\n\n", self.round));

        // 挑战者状态
        let challenger_hp_color = if self.challenger.hp_percent() > 50 {
            "#00ff00"
        } else if self.challenger.hp_percent() > 20 {
            "#ffff00"
        } else {
            "#ff0000"
        };

        output.push_str(&format!("§Y【挑战者】§N {} (Lv.{})\n",
            self.challenger.name_cn, self.challenger.level));
        output.push_str(&format!("§Y生命: §{}{}/{}§N\n",
            challenger_hp_color, self.challenger.hp, self.challenger.hp_max));

        // 防守者状态
        let defender_hp_color = if self.defender.hp_percent() > 50 {
            "#00ff00"
        } else if self.defender.hp_percent() > 20 {
            "#ffff00"
        } else {
            "#ff0000"
        };

        output.push_str(&format!("\n§R【防守者】§N {} (Lv.{})\n",
            self.defender.name_cn, self.defender.level));
        output.push_str(&format!("§R生命: §{}{}/{}§N\n",
            defender_hp_color, self.defender.hp, self.defender.hp_max));

        // 战斗日志
        if !self.combat_log.is_empty() {
            output.push_str(&format!("\n§H【战斗记录】§N\n"));
            for log_entry in self.combat_log.iter().rev().take(5) {
                output.push_str(log_entry);
                output.push_str("\n");
            }
        }

        // 操作按钮
        output.push_str(&format!("\n§H【操作】§N\n"));
        output.push_str("[继续战斗:pk continue]\n");
        output.push_str("[§Y逃跑§N:escape]\n");
        output.push_str("[§Y投降§N:surrender]\n");

        output
    }
}

/// PK守护进程
pub struct PkDaemon {
    battles: Arc<RwLock<HashMap<String, PkBattle>>>,
}

impl PkDaemon {
    pub fn new() -> Self {
        Self {
            battles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 发起PK挑战
    pub async fn challenge(&self, challenger: CombatStats, defender: CombatStats) -> Result<PkBattle, String> {
        // 检查是否可以发起攻击
        challenger.can_attack(&defender)?;

        // 检查是否已经在战斗中
        let battles = self.battles.read().await;
        if battles.contains_key(&challenger.id) {
            return Err("你正在战斗中！".to_string());
        }
        if battles.contains_key(&defender.id) {
            return Err("对方正在战斗中！".to_string());
        }
        drop(battles);

        // 创建战斗
        let challenger_id = challenger.id.clone();
        let defender_id = defender.id.clone();
        let battle = PkBattle::new(challenger, defender);
        let battle_id = battle.battle_id.clone();

        let mut battles = self.battles.write().await;
        battles.insert(battle_id.clone(), battle.clone());
        battles.insert(challenger_id, battle.clone());
        battles.insert(defender_id, battle.clone());

        Ok(battle)
    }

    /// 获取战斗
    pub async fn get_battle(&self, battle_id: &str) -> Option<PkBattle> {
        self.battles.read().await.get(battle_id).cloned()
    }

    /// 获取玩家当前的战斗
    pub async fn get_player_battle(&self, player_id: &str) -> Option<PkBattle> {
        self.battles.read().await.get(player_id).cloned()
    }

    /// 执行下一个回合
    pub async fn next_round(&self, battle_id: &str) -> Option<CombatRound> {
        let mut battles = self.battles.write().await;
        if let Some(battle) = battles.get_mut(battle_id) {
            if battle.status == CombatStatus::Fighting {
                let round = battle.execute_round();
                Some(round)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 结束战斗
    pub async fn end_battle(&self, battle_id: &str) -> Option<PkBattle> {
        let mut battles = self.battles.write().await;
        battles.remove(battle_id)
    }

    /// 玩家逃跑
    pub async fn escape(&self, player_id: &str) -> Result<String, String> {
        // 先克隆需要的battle数据
        let (battle_id, challenger_id, defender_id, dodge_a, dodge_d) = {
            let battles = self.battles.read().await;
            if let Some(battle) = battles.get(player_id) {
                (
                    battle.battle_id.clone(),
                    battle.challenger.id.clone(),
                    battle.defender.id.clone(),
                    battle.challenger.dodge,
                    battle.defender.dodge,
                )
            } else {
                return Err("你不在战斗中！".to_string());
            }
        };

        // 检查逃跑成功率
        let success_rate = dodge_a as f64 / (dodge_a + dodge_d) as f64;

        if rand::random::<f64>() < success_rate * 0.8 {
            // 逃跑成功 - 移除战斗
            let mut battles = self.battles.write().await;
            battles.remove(&challenger_id);
            battles.remove(&defender_id);
            battles.remove(&battle_id);
            Ok("§Y你成功逃脱了！§N".to_string())
        } else {
            Err("§R你逃跑失败了！§N".to_string())
        }
    }

    /// 投降
    pub async fn surrender(&self, player_id: &str) -> Result<String, String> {
        // 先克隆需要的battle数据
        let (battle_id, challenger_id, defender_id, is_challenger) = {
            let battles = self.battles.read().await;
            if let Some(battle) = battles.get(player_id) {
                (
                    battle.battle_id.clone(),
                    battle.challenger.id.clone(),
                    battle.defender.id.clone(),
                    battle.challenger.id == player_id,
                )
            } else {
                return Err("你不在战斗中！".to_string());
            }
        };

        if is_challenger {
            // 移除战斗
            let mut battles = self.battles.write().await;
            battles.remove(&challenger_id);
            battles.remove(&defender_id);
            battles.remove(&battle_id);
            Ok("§Y你投降了！§N".to_string())
        } else {
            Err("只有发起者可以投降！".to_string())
        }
    }
}

impl Default for PkDaemon {
    fn default() -> Self {
        Self::new()
    }
}

// 全局PK守护进程
lazy_static::lazy_static! {
    pub static ref PKD: Arc<PkDaemon> = Arc::new(PkDaemon::new());
}

/// 便捷函数
pub async fn get_pkd() -> Arc<PkDaemon> {
    PKD.clone()
}
