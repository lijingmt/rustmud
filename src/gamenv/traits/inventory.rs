// gamenv/traits/inventory.rs - 背包特性
// 对应 txpike9/wapmud2/inherit/feature/inventory.pike

use crate::gamenv::item::Item;

/// 背包特性 - 所有有背包的对象都应实现此trait
pub trait Inventory {
    /// 获取背包中的所有物品
    fn items(&self) -> &[Item];

    /// 添加物品到背包
    async fn add_item(&mut self, item: Item) -> Result<(), String>;

    /// 从背包移除物品
    async fn remove_item(&mut self, item_id: &str) -> Result<Item, String>;

    /// 获取指定ID的物品
    fn get_item(&self, item_id: &str) -> Option<&Item>;

    /// 列出背包物品
    fn list_items(&self) -> String {
        let items = self.items();
        if items.is_empty() {
            return "背包是空的。".to_string();
        }

        let mut output = "你的背包：\n".to_string();
        for item in items {
            output.push_str(&format!("  {}\n", item.name_cn));
        }
        output
    }

    /// 获取最大负重
    fn max_encumbrance(&self) -> i32;
}
