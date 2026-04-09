// examples/test_equip.rs - 装备系统测试
// 运行: cargo run --example test_equip

use rustmud::equip::{
    ForgeSystem, EquipDesign, EquipType, EquipQuality, Effect,
    FORGE_SYSTEM,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 装备系统测试 ===\n");

    // 初始化打造系统
    let mut forge = ForgeSystem::new();

    // ========================================
    // 1. 加载装备模板
    // ========================================
    println!("【1】加载装备模板...");

    let templates_json = r#"
[
    {
        "id": "weapon_sword",
        "name": "长剑",
        "equip_type": "weapon",
        "base_attack": 100,
        "base_defense": 0,
        "allowed_materials": ["iron", "steel", "cold_iron", "mithril", "dragon_bone"],
        "max_slots": 3,
        "available_effects": ["吸血", "暴击", "破甲"]
    },
    {
        "id": "armor_plate",
        "name": "板甲",
        "equip_type": "armor",
        "base_attack": 0,
        "base_defense": 150,
        "allowed_materials": ["iron", "steel", "cold_iron"],
        "max_slots": 2,
        "available_effects": ["反伤", "闪避", "hp_bonus"]
    },
    {
        "id": "ring_magic",
        "name": "魔法戒指",
        "equip_type": "ring",
        "base_attack": 20,
        "base_defense": 10,
        "allowed_materials": ["mithril", "jade", "dragon_scale"],
        "max_slots": 1,
        "available_effects": ["hp_bonus", "攻击加成", "防御加成"]
    }
]
"#;

    forge.load_templates(templates_json)?;
    println!("  已加载 {} 个装备模板\n", forge.list_templates().len());

    // ========================================
    // 2. 加载材料数据
    // ========================================
    println!("【2】加载材料数据...");

    let materials_json = r#"
[
    {
        "name": "iron",
        "base_attack": 10,
        "base_defense": 5,
        "quality_bonus": 1.0
    },
    {
        "name": "steel",
        "base_attack": 25,
        "base_defense": 15,
        "quality_bonus": 1.5
    },
    {
        "name": "cold_iron",
        "base_attack": 50,
        "base_defense": 20,
        "quality_bonus": 2.0
    },
    {
        "name": "mithril",
        "base_attack": 80,
        "base_defense": 30,
        "quality_bonus": 3.0
    },
    {
        "name": "dragon_bone",
        "base_attack": 150,
        "base_defense": 50,
        "quality_bonus": 4.0
    },
    {
        "name": "jade",
        "base_attack": 5,
        "base_defense": 40,
        "quality_bonus": 2.5
    },
    {
        "name": "dragon_scale",
        "base_attack": 30,
        "base_defense": 100,
        "quality_bonus": 4.0
    }
]
"#;

    forge.load_materials(materials_json)?;
    println!("  已加载材料库\n");

    // ========================================
    // 3. 玩家打造普通装备
    // ========================================
    println!("【3】玩家打造普通装备...");

    let design = EquipDesign {
        template_id: "weapon_sword".to_string(),
        custom_name: None,  // 使用默认名称
        materials: vec!["iron".to_string()],
        effects: vec![],
        quality: None,  // 自动计算品质
    };

    let equip = forge.forge(&design, "player_001")?;
    println!("  打造成功！");
    println!("  {} - 攻击: {}, 防御: {}, 品质: {}\n",
        equip.name, equip.attack, equip.defense, equip.quality.zh_name());

    // ========================================
    // 4. 打造精良武器（带自定义名称）
    // ========================================
    println!("【4】打造精良武器...");

    let design = EquipDesign {
        template_id: "weapon_sword".to_string(),
        custom_name: Some("屠龙刀".to_string()),
        materials: vec!["steel".to_string(), "cold_iron".to_string()],
        effects: vec![
            Effect::LifeSteal { percent: 10 },
            Effect::Critical { chance: 15 },
        ],
        quality: None,
    };

    let equip = forge.forge(&design, "player_001")?;
    println!("{}", equip.describe());

    // ========================================
    // 5. 打造传说装备
    // ========================================
    println!("【5】打造传说装备...");

    let design = EquipDesign {
        template_id: "weapon_sword".to_string(),
        custom_name: Some("神剑·天诛".to_string()),
        materials: vec!["dragon_bone".to_string(), "mithril".to_string()],
        effects: vec![
            Effect::LifeSteal { percent: 20 },
            Effect::Critical { chance: 25 },
            Effect::ArmorBreak { percent: 30 },
        ],
        quality: None,
    };

    let equip = forge.forge(&design, "player_001")?;
    println!("{}", equip.describe());

    // ========================================
    // 6. 测试战斗计算
    // ========================================
    println!("【6】测试战斗计算...");

    let base_damage = 500;
    let target_defense = 200;

    println!("  基础伤害: {}, 目标防御: {}", base_damage, target_defense);

    // 模拟10次攻击（因为有暴击）
    println!("  模拟攻击结果:");
    for i in 1..=5 {
        let damage = equip.calc_damage(base_damage, target_defense);
        let life_steal = equip.after_attack(damage);
        println!("    第{}次: 造成{}伤害, 吸血{}HP", i, damage, life_steal);
    }
    println!();

    // ========================================
    // 7. 打造防御装备
    // ========================================
    println!("【7】打造防御装备...");

    let design = EquipDesign {
        template_id: "armor_plate".to_string(),
        custom_name: Some("龙鳞甲".to_string()),
        materials: vec!["dragon_scale".to_string()],
        effects: vec![
            Effect::Thorns { percent: 15 },
            Effect::Dodge { chance: 10 },
            Effect::HpBonus { value: 500 },
        ],
        quality: None,
    };

    let armor = forge.forge(&design, "player_001")?;
    println!("{}", armor.describe());

    // ========================================
    // 8. 打造饰品
    // ========================================
    println!("【8】打造饰品...");

    let design = EquipDesign {
        template_id: "ring_magic".to_string(),
        custom_name: Some("翡翠护身符".to_string()),
        materials: vec!["jade".to_string()],
        effects: vec![
            Effect::HpBonus { value: 1000 },
            Effect::DefenseBonus { value: 50 },
        ],
        quality: None,
    };

    let ring = forge.forge(&design, "player_001")?;
    println!("{}", ring.describe());

    // ========================================
    // 9. 查看可用模板
    // ========================================
    println!("【9】可用的装备模板:");
    for template in forge.list_templates() {
        println!("  - {} ({}): 攻击{}, 防御{}",
            template.id,
            template.name,
            template.base_attack,
            template.base_defense
        );
    }

    println!("\n=== 测试完成 ===");

    Ok(())
}
