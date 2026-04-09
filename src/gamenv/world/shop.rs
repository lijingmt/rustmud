// gamenv/world/shop.rs - 商店系统

use serde::{Deserialize, Serialize};

/// 商店
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shop {
    /// 商店ID
    pub id: String,
    /// 商店名称
    pub name: String,
    /// 商品列表
    pub items: Vec<ShopItem>,
}

/// 商店商品
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShopItem {
    /// 物品ID
    pub item_id: String,
    /// 物品名称
    pub name: String,
    /// 价格
    pub price: i32,
    /// 库存数量 (-1表示无限)
    pub stock: i32,
}

impl Shop {
    /// 新建商店
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            items: vec![],
        }
    }

    /// 添加商品
    pub fn add_item(&mut self, item_id: String, name: String, price: i32, stock: i32) {
        self.items.push(ShopItem {
            item_id,
            name,
            price,
            stock,
        });
    }

    /// 获取商品
    pub fn get_item(&self, item_id: &str) -> Option<&ShopItem> {
        self.items.iter().find(|i| i.item_id == item_id)
    }

    /// 检查是否有库存
    pub fn has_stock(&self, item_id: &str) -> bool {
        if let Some(item) = self.get_item(item_id) {
            item.stock == -1 || item.stock > 0
        } else {
            false
        }
    }

    /// 减少库存
    pub fn reduce_stock(&mut self, item_id: &str, count: i32) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.item_id == item_id) {
            if item.stock == -1 {
                return true;
            }
            if item.stock >= count {
                item.stock -= count;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 格式化商品列表
    pub fn format_items(&self) -> String {
        let mut result = format!("=== {} ===\n", self.name);

        for (idx, item) in self.items.iter().enumerate() {
            let stock_text = if item.stock == -1 {
                "无限".to_string()
            } else {
                format!("库存:{}", item.stock)
            };

            result.push_str(&format!(
                "{}. {} - {}金币 ({})\n",
                idx + 1,
                item.name,
                item.price,
                stock_text
            ));
        }

        result
    }
}
