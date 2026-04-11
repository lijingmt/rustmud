// gamenv/traits/movable.rs - 移动特性
// 对应 txpike9/wapmud2/inherit/feature/ (move相关)

/// 移动特性 - 所有可移动的对象都应实现此trait
pub trait Movable {
    /// 移动到指定方向
    async fn move_to(&mut self, direction: &str) -> Result<String, String>;

    /// 获取当前位置
    fn current_room(&self) -> &str;

    /// 检查是否可以移动到指定方向
    fn can_move(&self, direction: &str) -> bool;
}
