// gamenv/traits/fight.rs - 战斗特性
// 对应 txpike9/wapmud2/inherit/feature/fight.pike

/// 战斗特性 - 所有可战斗的对象都应实现此trait
pub trait Fight {
    /// 获取当前生命值
    fn hp(&self) -> i32;

    /// 获取最大生命值
    fn hp_max(&self) -> i32;

    /// 获取生命值百分比
    fn hp_percent(&self) -> i32 {
        let max = self.hp_max();
        if max == 0 { return 0; }
        (self.hp() * 100 / max).max(0).min(100)
    }

    /// 是否存活
    fn is_alive(&self) -> bool {
        self.hp() > 0
    }

    /// 受到伤害
    async fn take_damage(&mut self, damage: i32) -> bool;

    /// 恢复生命
    async fn heal(&mut self, amount: i32);

    /// 获取攻击力
    fn attack_power(&self) -> i32;

    /// 获取防御力
    fn defense(&self) -> i32;

    /// 获取等级
    fn level(&self) -> u32;
}
