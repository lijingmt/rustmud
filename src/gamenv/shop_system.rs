// gamenv/shop_system.rs - 商店系统
// 对应 txpike9 的商店功能

use crate::gamenv::world::{Shop, ShopItem};
use crate::gamenv::player_state::PlayerState;
use crate::gamenv::world::ItemTemplate;
use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 交易结果
#[derive(Clone, Debug)]
pub enum TradeResult {
    /// 交易成功
    Success { message: String, change_gold: i64 },
    /// 交易失败
    Failed(String),
    /// 商店物品不足
    OutOfStock,
    /// 玩家金币不足
    NotEnoughGold,
    /// 背包已满
    InventoryFull,
}

/// 商店交易
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShopTransaction {
    /// 物品ID
    pub item_id: String,
    /// 物品名称
    pub item_name: String,
    /// 数量
    pub count: i32,
    /// 单价
    pub price: i32,
    /// 总价
    pub total: i32,
    /// 交易类型
    pub transaction_type: TransactionType,
}

/// 交易类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    /// 购买
    Buy,
    /// 出售
    Sell,
}

/// 商店系统
pub struct ShopSystem;

impl ShopSystem {
    /// 购买物品
    pub fn buy_item(
        shop: &mut Shop,
        item_id: &str,
        count: i32,
        player: &mut PlayerState,
        world_items: &HashMap<String, ItemTemplate>,
    ) -> TradeResult {
        // 验证数量
        if count <= 0 {
            return TradeResult::Failed("数量必须大于0".to_string());
        }

        // 查找商店物品
        let shop_item = shop.get_item(item_id);
        let shop_item = match shop_item {
            Some(item) => item.clone(),
            None => return TradeResult::Failed("商店没有这个物品".to_string()),
        };

        // 检查库存
        if !shop.has_stock(item_id) {
            return TradeResult::OutOfStock;
        }

        // 额外检查数量
        if let Some(shop_item) = shop.get_item(item_id) {
            if shop_item.stock >= 0 && shop_item.stock < count {
                return TradeResult::OutOfStock;
            }
        }

        // 计算总价
        let total = (shop_item.price as i64) * (count as i64);

        // 检查玩家金币
        if player.gold < total as u64 {
            return TradeResult::NotEnoughGold;
        }

        // 检查背包空间
        if player.inventory.len() >= player.inventory_capacity
            && !player.inventory.contains_key(item_id) {
            return TradeResult::InventoryFull;
        }

        // 扣除金币
        let _ = player.spend_gold(total as u64);

        // 减少库存
        shop.reduce_stock(item_id, count);

        // 添加物品到背包
        if let Some(item_template) = world_items.get(item_id) {
            let _ = player.add_item(item_template, count);
        }

        TradeResult::Success {
            message: format!("你购买了 {} x{}，花费了 {} 金币", shop_item.name, count, total),
            change_gold: -(total as i64),
        }
    }

    /// 出售物品
    pub fn sell_item(
        shop: &Shop,
        item_id: &str,
        count: i32,
        player: &mut PlayerState,
        world_items: &HashMap<String, ItemTemplate>,
    ) -> TradeResult {
        // 验证数量
        if count <= 0 {
            return TradeResult::Failed("数量必须大于0".to_string());
        }

        // 检查玩家是否有该物品
        let player_count = player.get_item_count(item_id);
        if player_count < count {
            return TradeResult::Failed("你没有足够的物品".to_string());
        }

        // 获取物品模板
        let item_template = match world_items.get(item_id) {
            Some(item) => item,
            None => return TradeResult::Failed("物品不存在".to_string()),
        };

        // 计算售价（通常为购买价的50%）
        let sell_price = (item_template.price / 2) as i64;
        let total = sell_price * (count as i64);

        // 从背包移除物品
        let _ = player.remove_item(item_id, count);

        // 添加金币
        player.add_gold(total as u64);

        TradeResult::Success {
            message: format!("你出售了 {} x{}，获得了 {} 金币", item_template.name, count, total),
            change_gold: total as i64,
        }
    }

    /// 格式化商店列表
    pub fn format_shop_list(shop: &Shop, player_gold: u64) -> String {
        let mut output = format!("§C=== {} ===§N\n", shop.name);
        output.push_str(&format!("你的金币: §Y{}§N\n\n", player_gold));

        if shop.items.is_empty() {
            output.push_str("商店目前没有商品。\n");
        } else {
            output.push_str("§H商品列表:§N\n");
            output.push_str(&format!("{:<5} {:<20} {:<10} {:<10}\n", "序号", "商品", "价格", "库存"));
            output.push_str(&format!("{:<5} {:<20} {:<10} {:<10}\n",
                "──", "────", "────", "────"));

            for (idx, shop_item) in shop.items.iter().enumerate() {
                let stock = if shop_item.stock > 999 { "∞" } else { "§" };
                output.push_str(&format!("{:<5} {:<20} {:<10} {:<10}\n",
                    format!("{}", idx + 1),
                    shop_item.name,
                    format!("{}金币", shop_item.price),
                    format!("{}{}", shop_item.stock, stock),
                ));
            }
        }

        output
    }

    /// 获取购买帮助信息
    pub fn get_buy_help() -> &'static str {
        r#"
使用方法:
  buy <序号> [数量] - 购买商品
  sell <物品名> [数量] - 出售物品
  list - 查看商品列表
  done - 离开商店

示例:
  buy 1 - 购买第1件商品（1个）
  buy 2 5 - 购买第2件商品（5个）
  sell 生命药水 3 - 出售3个生命药水
"#
    }

    /// 解析购买命令
    pub fn parse_buy_command(input: &str) -> Result<(usize, i32)> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() || parts.len() > 2 {
            return Err(MudError::InvalidOperation("命令格式错误".to_string()));
        }

        let index: usize = parts[0].parse()
            .map_err(|_| MudError::InvalidOperation("无效的序号".to_string()))?;

        let count = if parts.len() > 1 {
            parts[1].parse().map_err(|_| MudError::InvalidOperation("无效的数量".to_string()))?
        } else {
            1
        };

        Ok((index, count))
    }

    /// 执行商店命令
    pub fn execute_shop_command(
        cmd: &str,
        shop: &mut Shop,
        player: &mut PlayerState,
        world_items: &HashMap<String, ItemTemplate>,
    ) -> Result<String> {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Ok(Self::format_shop_list(shop, player.gold));
        }

        match parts[0] {
            "list" | "l" => {
                Ok(Self::format_shop_list(shop, player.gold))
            }
            "buy" | "b" => {
                if parts.len() < 2 {
                    return Ok("请输入要购买的序号。\n".to_string());
                }

                let rest = parts[1..].join(" ");
                match Self::parse_buy_command(&rest) {
                    Ok((index, count)) => {
                        if index == 0 || index > shop.items.len() {
                            return Ok("无效的序号。\n".to_string());
                        }

                        let item = &shop.items[index - 1];
                        match Self::buy_item(&mut shop.clone(), &item.item_id, count, player, world_items) {
                            TradeResult::Success { message, .. } => {
                                Ok(format!("§G{}§N\n", message))
                            }
                            TradeResult::OutOfStock => {
                                Ok("§R库存不足！§N\n".to_string())
                            }
                            TradeResult::NotEnoughGold => {
                                Ok("§R金币不足！§N\n".to_string())
                            }
                            TradeResult::InventoryFull => {
                                Ok("§R背包已满！§N\n".to_string())
                            }
                            TradeResult::Failed(msg) => {
                                Ok(format!("§R{}§N\n", msg))
                            }
                        }
                    }
                    Err(_) => Ok("§R命令格式错误！§N\n使用: buy <序号> [数量]\n".to_string()),
                }
            }
            "sell" | "s" => {
                if parts.len() < 2 {
                    return Ok("请输入要出售的物品名。\n".to_string());
                }

                let item_name = parts[1];
                let count = if parts.len() > 2 {
                    parts[2].parse().unwrap_or(1)
                } else {
                    1
                };

                // 查找物品ID
                let item_id = world_items.values()
                    .find(|item| item.name.contains(item_name))
                    .map(|item| item.id.clone());

                let item_id = match item_id {
                    Some(id) => id,
                    None => return Ok("§R找不到这个物品！§N\n".to_string()),
                };

                match Self::sell_item(shop, &item_id, count, player, world_items) {
                    TradeResult::Success { message, .. } => {
                        Ok(format!("§G{}§N\n", message))
                    }
                    TradeResult::Failed(msg) => {
                        Ok(format!("§R{}§N\n", msg))
                    }
                    _ => Ok("§R交易失败！§N\n".to_string()),
                }
            }
            "help" | "h" | "?" => {
                Ok(Self::get_buy_help().to_string())
            }
            "done" | "exit" | "quit" | "q" => {
                Ok("感谢光临！\n".to_string())
            }
            _ => {
                Ok("§R未知命令！§N\n输入 help 查看帮助。\n".to_string())
            }
        }
    }
}

/// 商店管理器
pub struct ShopManager {
    /// 所有商店
    shops: HashMap<String, Shop>,
}

impl ShopManager {
    pub fn new() -> Self {
        Self {
            shops: HashMap::new(),
        }
    }

    /// 添加商店
    pub fn add_shop(&mut self, shop: Shop) {
        self.shops.insert(shop.id.clone(), shop);
    }

    /// 获取商店
    pub fn get_shop(&self, shop_id: &str) -> Option<&Shop> {
        self.shops.get(shop_id)
    }

    /// 获取可变商店
    pub fn get_shop_mut(&mut self, shop_id: &str) -> Option<&mut Shop> {
        self.shops.get_mut(shop_id)
    }

    /// 恢复商店库存（定时任务）
    pub fn restock_all(&mut self) {
        for shop in self.shops.values_mut() {
            // TODO: 实现库存恢复逻辑
            shop.restock();
        }
    }
}

impl Shop {
    /// 补货
    pub fn restock(&mut self) {
        for item in &mut self.items {
            if item.stock < 10 {
                item.stock = 999;
            }
        }
    }
}

impl Default for ShopManager {
    fn default() -> Self {
        Self::new()
    }
}
