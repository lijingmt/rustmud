// gamenv/npc/merchant.rs - 商人NPC
// 对应 txpike9/gamenv/clone/npc/ 中的商人

use crate::core::*;
use crate::gamenv::npc::npc::{Npc, NpcType};
use serde::{Deserialize, Serialize};

/// 商人类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MerchantType {
    /// 武器商
    Weapon,
    /// 防具商
    Armor,
    /// 药品商
    Medicine,
    /// 杂货商
    General,
    /// 当铺
    Pawnshop,
    /// 银行
    Bank,
}

impl MerchantType {
    /// 获取商人类型名称
    pub fn name(&self) -> &str {
        match self {
            MerchantType::Weapon => "武器商",
            MerchantType::Armor => "防具商",
            MerchantType::Medicine => "药品商",
            MerchantType::General => "杂货商",
            MerchantType::Pawnshop => "当铺",
            MerchantType::Bank => "钱庄",
        }
    }
}

/// 商店物品
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    /// 物品模板ID
    pub item_id: String,
    /// 售价
    pub price: u64,
    /// 收购价
    pub buyback_price: u64,
    /// 库存数量 (0表示无限)
    pub stock: u32,
}

/// 商人NPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Merchant {
    /// 基础NPC数据
    #[serde(flatten)]
    pub npc: Npc,
    /// 商人类型
    pub merchant_type: MerchantType,
    /// 出售物品列表
    pub shop_items: Vec<ShopItem>,
    /// 收购折扣 (0.0 - 1.0)
    pub buy_discount: f32,
}

impl Merchant {
    /// 创建新商人
    pub fn new(name: String, name_cn: String, merchant_type: MerchantType) -> Self {
        let npc = Npc::new(name.clone(), name_cn.clone(), NpcType::Merchant, 1);

        Self {
            npc,
            merchant_type,
            shop_items: Vec::new(),
            buy_discount: 0.5, // 默认5折收购
        }
    }

    /// 添加商品
    pub fn add_shop_item(&mut self, item_id: String, price: u64, stock: u32) {
        let buyback_price = (price as f32 * self.buy_discount) as u64;
        self.shop_items.push(ShopItem {
            item_id,
            price,
            buyback_price,
            stock,
        });
    }

    /// 设置收购折扣
    pub fn with_buy_discount(mut self, discount: f32) -> Self {
        self.buy_discount = discount.max(0.0).min(1.0);
        self
    }

    /// 获取商品信息
    pub fn get_item_info(&self, item_id: &str) -> Option<&ShopItem> {
        self.shop_items.iter().find(|item| item.item_id == item_id)
    }

    /// 渲染商店列表
    pub fn render_shop_list(&self) -> String {
        let mut result = format!("=== {} ===\n", self.merchant_type.name());
        result.push_str(&format!("店主: {}\n", self.npc.base.name_cn));

        for (idx, item) in self.shop_items.iter().enumerate() {
            let stock_str = if item.stock == 0 {
                "无限".to_string()
            } else {
                item.stock.to_string()
            };
            result.push_str(&format!("{}. {} - {} 金 (库存: {})\n",
                idx + 1,
                item.item_id,
                item.price,
                stock_str
            ));
        }

        result.push_str("\n§c[buy <编号> <数量>] - 购买物品§r\n");
        result.push_str("§c[sell <物品名>] - 出售物品§r\n");

        result
    }

    /// 购买物品
    pub fn buy_item(&mut self, item_id: &str, quantity: u32, player_money: u64) -> Result<(u64, u32)> {
        if let Some(item) = self.get_item_info(item_id) {
            // 检查库存
            if item.stock != 0 && item.stock < quantity {
                return Err(MudError::InvalidOperation("库存不足".to_string()));
            }

            let total_cost = item.price * quantity as u64;
            if total_cost > player_money {
                return Err(MudError::InvalidOperation("金钱不足".to_string()));
            }

            // 减少库存
            if item.stock != 0 {
                if let Some(shop_item) = self.shop_items.iter_mut().find(|i| i.item_id == item_id) {
                    shop_item.stock -= quantity;
                }
            }

            Ok((total_cost, quantity))
        } else {
            Err(MudError::NotFound("商品不存在".to_string()))
        }
    }

    /// 出售物品
    pub fn sell_item(&self, item_value: u64) -> u64 {
        (item_value as f32 * self.buy_discount) as u64
    }
}

/// 预设商人列表
pub fn create_preset_merchants() -> Vec<Merchant> {
    vec![
        // 武器商
        {
            let mut merchant = Merchant::new(
                "merchant_weapon_beijing".to_string(),
                "铁匠".to_string(),
                MerchantType::Weapon,
            );
            merchant.npc.base.desc = "一位经验丰富的铁匠，可以打造各种武器。".to_string();
            merchant.add_shop_item("item/iron_sword".to_string(), 100, 99);
            merchant.add_shop_item("item/steel_blade".to_string(), 500, 10);
            merchant.add_shop_item("item/jade_sword".to_string(), 2000, 5);
            merchant.npc.base.add_dialogue("客官，来看看这把好剑！".to_string());
            merchant
        },

        // 防具商
        {
            let mut merchant = Merchant::new(
                "merchant_armor_beijing".to_string(),
                "裁缝".to_string(),
                MerchantType::Armor,
            );
            merchant.npc.base.desc = "一位手艺精湛的裁缝。".to_string();
            merchant.add_shop_item("item/cloth_shirt".to_string(), 50, 99);
            merchant.add_shop_item("item/leather_armor".to_string(), 200, 20);
            merchant.add_shop_item("item/iron_armor".to_string(), 1000, 5);
            merchant
        },

        // 药品商
        {
            let mut merchant = Merchant::new(
                "merchant_medicine".to_string(),
                "药商".to_string(),
                MerchantType::Medicine,
            );
            merchant.npc.base.desc = "出售各种药材和药品。".to_string();
            merchant.add_shop_item("med/hp_small".to_string(), 10, 999);
            merchant.add_shop_item("med/hp_medium".to_string(), 50, 200);
            merchant.add_shop_item("med/qi_small".to_string(), 15, 500);
            merchant.npc.base.add_dialogue("用药需谨慎啊！".to_string());
            merchant
        },
    ]
}
