// examples/test_equip_full.rs - 完整装备系统测试
// 运行: cargo run --example test_equip_full

use rustmud::equip::{
    EquipSystem, Equipment, EquipSlot, EquipQuality, EquipRealm,
    EquipStats, EquipEffect, EQUIP_SYSTEM,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== txpike9 装备系统测试（JSON配置驱动）===\n");

    // 初始化日志
    tracing_subscriber::fmt::init();

    // 初始化装备系统
    let mut system = EquipSystem::new();

    // ========================================
    // 1. 从 JSON 加载配置
    // ========================================
    println!("【1】加载 JSON 配置...");

    // 加载装备模板
    let templates_json = std::fs::read_to_string("data/equipment/templates.json")?;
    system.load_templates(&templates_json)?;

    // 加载套装配置
    let suits_json = std::fs::read_to_string("data/equipment/suits.json")?;
    system.load_suits(&suits_json)?;

    // 加载打造配方
    let recipes_json = std::fs::read_to_string("data/equipment/recipes.json")?;
    system.load_recipes(&recipes_json)?;

    // 加载材料配置
    let materials_json = std::fs::read_to_string("data/equipment/materials.json")?;
    system.load_materials(&materials_json)?;

    println!("  配置加载完成！\n");

    // ========================================
    // 2. 查看装备模板
    // ========================================
    println!("【2】可用装备模板:");
    for template in system.list_templates() {
        let stats = template.calc_stats();
        println!("  [{}] {} {} - 攻击:{} 防御:{}",
            template.id,
            template.quality.color_code(),
            template.name,
            stats.attack,
            stats.defense
        );
    }
    println!();

    // ========================================
    // 3. 品质系统测试
    // ========================================
    println!("【3】品质系统测试:");

    for quality in [
        EquipQuality::Common,
        EquipQuality::Uncommon,
        EquipQuality::Rare,
        EquipQuality::Epic,
        EquipQuality::Legendary,
        EquipQuality::Mythic,
    ] {
        println!("  {} {}: {}倍属性",
            quality.color_code(),
            quality.zh_name(),
            quality.multiplier()
        );
    }
    println!();

    // ========================================
    // 4. 境界系统测试
    // ========================================
    println!("【4】境界系统测试:");
    let test_levels = [5, 15, 35, 70, 120, 180, 250, 400, 700, 1500];
    for level in test_levels {
        let realm = EquipRealm::from_level(level);
        println!("  等级 {} -> {} {}",
            level,
            realm.color_code(),
            realm.zh_name()
        );
    }
    println!();

    // ========================================
    // 5. 从模板创建装备
    // ========================================
    println!("【5】从模板创建装备:");

    if let Some(template) = system.get_template("weapon_dragon_slayer") {
        let equip = Equipment::from_template(template, "player_001");
        println!("{}", equip.describe());
    }

    // ========================================
    // 6. 强化系统测试
    // ========================================
    println!("【6】强化系统测试:");

    let mut equip = {
        let template = system.get_template("weapon_frost_sword").unwrap();
        Equipment::from_template(template, "player_001")
    };

    println!("  初始属性: 攻击={}", equip.final_stats().attack);

    // 强化到 +10
    for _ in 0..10 {
        equip.reinforce().unwrap();
    }
    println!("  强化到 +10: 攻击={} (预期{} * 2.0 = {})",
        equip.final_stats().attack,
        equip.base_stats.attack,
        equip.base_stats.attack as f64 * 2.0
    );

    println!("  {}", equip.display_name());
    println!();

    // ========================================
    // 7. 战斗计算测试
    // ========================================
    println!("【7】战斗计算测试:");

    let legend_equip = {
        let template = system.get_template("weapon_dragon_slayer").unwrap();
        let mut e = Equipment::from_template(template, "player_001");
        // 强化到 +5
        for _ in 0..5 { e.reinforce().unwrap(); }
        e
    };

    println!("  装备: {}", legend_equip.display_name());
    println!("  属性: 攻击={} 暴击={}%",
        legend_equip.final_stats().attack,
        legend_equip.final_stats().crit_rate
    );

    let base_damage = 500;
    let target_defense = 200;

    println!("  基础伤害: {}, 目标防御: {}", base_damage, target_defense);
    println!("  模拟10次攻击:");
    for i in 1..=10 {
        let damage = legend_equip.calc_damage(base_damage, target_defense);
        let life_steal = legend_equip.after_attack(damage);
        println!("    第{}次: 造成{}伤害, 吸血{}HP", i, damage, life_steal);
    }
    println!();

    // ========================================
    // 8. 打造系统测试
    // ========================================
    println!("【8】打造系统测试:");

    println!("  可用配方:");
    for recipe in system.list_recipes() {
        println!("    [{}] {} - 成功率:{}% 等级要求:{}",
            recipe.id,
            recipe.name,
            recipe.success_rate,
            recipe.level_req
        );
    }
    println!();

    // 模拟打造（尝试打造屠龙刀，成功率30%）
    println!("  尝试打造屠龙刀（成功率30%）...");
    for i in 1..=5 {
        match system.forge("recipe_dragon_slayer", "player_001") {
            Ok(equip) => {
                println!("    第{}次尝试: 打造成功！", i);
                println!("    获得装备: {}", equip.display_name());
                break;
            }
            Err(_) => {
                println!("    第{}次尝试: 打造失败！", i);
            }
        }
    }
    println!();

    // ========================================
    // 9. 神话装备测试
    // ========================================
    println!("【9】神话装备测试:");

    if let Some(template) = system.get_template("weapon_mythic_blade") {
        let equip = Equipment::from_template(template, "player_001");
        println!("{}", equip.describe());

        println!("  战力测试:");
        let damage = equip.calc_damage(1000, 500);
        println!("    对500防御目标造成: {}点伤害", damage);
    }

    println!("\n=== 测试完成 ===");

    Ok(())
}
