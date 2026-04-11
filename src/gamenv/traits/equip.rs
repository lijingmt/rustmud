// gamenv/traits/equip.rs - 装备特性
// 对应 txpike9/wapmud2/inherit/feature/equip.pike

use crate::gamenv::item::Item;

/// 装备槽位
#[derive(Debug, Clone, Copy)]
pub enum EquipSlot {
    Weapon,
    Armor,
    Helmet,
    Boots,
    Accessory,
}

/// 装备特性 - 所有可装备物品的对象都应实现此trait
pub trait Equip {
    /// 装备物品
    async fn equip_item(&mut self, item: Item) -> Result<(), String>;

    /// 卸下装备
    async fn unequip_item(&mut self, slot: EquipSlot) -> Result<Item, String>;

    /// 获取指定槽位的装备
    fn get_equipped(&self, slot: EquipSlot) -> Option<&Item>;

    /// 获取所有装备
    fn get_all_equipped(&self) -> Vec<(EquipSlot, &Item)>;
}
