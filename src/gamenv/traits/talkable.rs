// gamenv/traits/talkable.rs - 对话特性
// 对应 txpike9 中NPC的对话功能

/// 对话特性 - 所有可对话的对象都应实现此trait
pub trait Talkable {
    /// 与对象对话
    async fn talk(&self, topic: &str) -> String;

    /// 检查是否可以对话
    fn can_talk(&self) -> bool {
        true
    }

    /// 获取对话选项
    fn get_dialogue_options(&self) -> Vec<String> {
        vec!["打招呼".to_string()]
    }
}
