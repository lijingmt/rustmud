# txpike9 装备系统（Rust 实现）

## 系统概述

完全对应 txpike9 的装备系统，使用 JSON 配置驱动。

## 核心概念

### 1. 装备位置 (EquipSlot)

| 位置 | 说明 |
|------|------|
| weapon | 武器 |
| helmet | 头盔 |
| armor | 衣服/盔甲 |
| gloves | 手套 |
| boots | 鞋子 |
| belt | 腰带 |
| amulet | 护身符 |
| ring | 戒指（可装备2个） |

### 2. 装备品质 (EquipQuality)

| 品质 | 颜色 | 倍率 |
|------|------|------|
| Common | 灰色 #888888 | 1x |
| Uncommon | 绿色 #1eff00 | 2x |
| Rare | 蓝色 #0070dd | 4x |
| Epic | 紫色 #a335ee | 8x |
| Legendary | 橙色 #ff8000 | 16x |
| Mythic | 红色 #ff0000 | 32x |

### 3. 装备境界 (EquipRealm)

| 境界 | 颜色代码 | 等级范围 |
|------|----------|----------|
| 凡人 | §w (白色) | 0-10 |
| 筑基 | §g (绿色) | 11-30 |
| 金丹 | §b (蓝色) | 31-60 |
| 元婴 | §p (紫色) | 61-100 |
| 化神 | §o (橙色) | 101-150 |
| 炼虚 | §r (红色) | 151-200 |
| 合体 | §dr (暗红) | 201-300 |
| 大乘 | §y (金色) | 301-500 |
| 渡劫 | §rb (彩虹) | 501-1000 |
| 大道 | §qc (七彩) | 1000+ |

### 4. 装备属性 (EquipStats)

```json
{
  "attack": 100,        // 攻击力
  "defense": 50,        // 防御力
  "hp_max": 200,        // 生命值上限
  "qi_max": 100,        // 内力值上限
  "shen_max": 50,       // 精神值上限
  "crit_rate": 15,      // 暴击率 (%)
  "crit_damage": 150,   // 暴击伤害 (%)
  "hit_rate": 10,       // 命中率
  "dodge_rate": 10,     // 闪避率
  "fire_resist": 20,    // 火抗
  "ice_resist": 20,     // 冰抗
  "lightning_resist": 20, // 雷抗
  "poison_resist": 20   // 毒抗
}
```

### 5. 强化系统

- 强化等级: +0 ~ +15
- 强化公式: `最终属性 = 基础属性 × (1 + 强化等级 × 0.1)`
- 示例: +5 强化 = 1.5倍属性, +10 强化 = 2.0倍属性

### 6. 装备特效 (EquipEffect)

| 特效类型 | 参数 | 说明 |
|----------|------|------|
| life_steal | percent | 攻击时吸血 |
| critical | chance | 暴击率加成 |
| critical_damage | bonus | 暴击伤害加成 |
| armor_break | percent | 破甲 |
| thorns | percent | 反伤 |
| dodge | chance | 闪避率加成 |
| hit | chance | 命中率加成 |
| bonus_damage | value | 额外固定伤害 |
| bonus_damage_percent | percent | 额外百分比伤害 |
| damage_reduction | percent | 减伤 |
| hp_regen | value | HP每秒恢复 |
| qi_regen | value | 内力每秒恢复 |
| cooldown_reduction | percent | 技能冷却减少 |
| stun_chance | chance, duration | 眩晕概率 |
| silence_chance | chance, duration | 沉默概率 |
| crowd_control_immunity | - | 免疫控制 |
| move_speed | bonus | 移动速度 |
| exp_bonus | percent | 经验加成 |
| drop_bonus | percent | 掉落加成 |
| gold_bonus | percent | 金币加成 |
| custom | name, script | 自定义脚本 |

## JSON 配置文件

### 1. 装备模板 (templates.json)

```json
{
  "id": "weapon_dragon_slayer",
  "name": "屠龙刀",
  "slot": "weapon",
  "quality": "legendary",
  "level_req": 60,
  "base_stats": {
    "attack": 100,
    "crit_rate": 20
  },
  "effects": [
    { "type": "life_steal", "percent": 15 },
    { "type": "armor_break", "percent": 20 }
  ],
  "suit_id": "suit_dragon_slayer",
  "description": "传说中的屠龙神刀"
}
```

### 2. 套装配置 (suits.json)

```json
{
  "id": "suit_dragon_slayer",
  "name": "屠龙套装",
  "items": ["weapon_dragon_slayer", "armor_dragon_scale"],
  "bonuses": {
    "2": {
      "description": "2件套：屠龙之力",
      "stats": { "attack": 50, "defense": 50 },
      "effects": ["对龙类敌人伤害+30%"]
    }
  }
}
```

### 3. 打造配方 (recipes.json)

```json
{
  "id": "recipe_dragon_slayer",
  "name": "屠龙刀配方",
  "template_id": "weapon_dragon_slayer",
  "materials": {
    "dragon_scale": 5,
    "dragon_bone": 3
  },
  "success_rate": 30,
  "level_req": 60,
  "gold_cost": 10000,
  "partial_refund": false
}
```

### 4. 材料配置 (materials.json)

```json
{
  "id": "dragon_scale",
  "name": "龙鳞",
  "material_type": "rare_material",
  "rarity": 8,
  "price": 5000,
  "description": "真龙脱落鳞片，极其珍贵"
}
```

## 使用示例

### 初始化系统

```rust
use rustmud::equip::{EquipSystem, EQUIP_SYSTEM};

let mut system = EquipSystem::new();

// 从 JSON 加载
system.load_templates(include_str!("data/equipment/templates.json"))?;
system.load_suits(include_str!("data/equipment/suits.json"))?;
system.load_recipes(include_str!("data/equipment/recipes.json"))?;
system.load_materials(include_str!("data/equipment/materials.json"))?;
```

### 创建装备

```rust
use rustmud::equip::{Equipment, EquipSystem};

// 从模板创建
let template = system.get_template("weapon_dragon_slayer")?;
let equip = Equipment::from_template(template, player_id);

// 强化装备
equip.reinforce()?;  // +1
equip.reinforce()?;  // +2
```

### 战斗计算

```rust
// 计算伤害
let damage = equip.calc_damage(base_damage, target_defense);

// 吸血
let heal = equip.after_attack(damage);
```

### 打造装备

```rust
// 根据配方打造
let equip = system.forge("recipe_dragon_slayer", player_id)?;
```

## 文件结构

```
src/equip/mod.rs          # 装备系统核心代码
data/equipment/
  ├── templates.json      # 装备模板
  ├── suits.json          # 套装配置
  ├── recipes.json        # 打造配方
  └── materials.json      # 材料配置
examples/test_equip_full.rs  # 测试示例
```

## 与 txpike9 的对应关系

| txpike9 (Pike) | Rust 实现 |
|----------------|-----------|
| ItemType | EquipSlot |
| ItemQuality | EquipQuality |
| EquipRealm | EquipRealm |
| EquipmentStats | EquipStats |
| reinforce | reinforce() |
| save_object/restore_object | serde Serialize/Deserialize |

## 运行测试

```bash
cargo run --example test_equip_full
```
