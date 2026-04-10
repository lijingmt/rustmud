// examples/test_rhai.rs - Rhai 脚本引擎测试
//
// 运行: cargo run --example test_rhai

use rustmud::script::{EquipStats, ScriptEngine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let engine = ScriptEngine::new();

    println!("=== Rhai 脚本引擎测试 ===\n");

    // ========================================
    // 测试1: 简单表达式
    // ========================================
    println!("【测试1】简单表达式:");
    let result: i64 = engine.eval("1 + 1")?;
    println!("  1 + 1 = {}\n", result);

    // ========================================
    // 测试2: 使用游戏 API
    // ========================================
    println!("【测试2】游戏 API:");
    engine.eval::<()>(
        r#"
            log("Hello from Rhai!");
            send_msg(1001, "欢迎来到 RustMUD!");
            let hp = query_hp(1001);
            debug("玩家 HP: " + hp);
        "#,
    )?;
    println!();

    // ========================================
    // 测试3: 装备脚本示例
    // ========================================
    println!("【测试3】装备脚本:");

    let equip_script = r#"
        // 装备属性
        fn get_stats() {
            // 返回装备属性
            #{  // Rhai 的 map 字面量语法
                name: "神剑·天诛",
                equip_type: "weapon",
                attack: 5000,
                defense: 0,
                special: ["吸血10%", "暴击20%"]
            }
        }

        // 攻击时触发
        fn on_hit(target_id, damage) {
            // 10% 概率触发暴击
            if random_percent() <= 10 {
                let crit_damage = damage * 2;
                log("暴击！造成 " + crit_damage + " 点伤害");
                return crit_damage;
            }

            // 吸血效果
            let leech = damage / 10;
            add_hp(12345, leech);  // 假设 12345 是攻击者 ID
            log("吸血 " + leech + " 点");

            damage
        }

        // 装备时触发
        fn on_equip(player_id) {
            send_msg(player_id, "你装备了 " + equip_name(999) + "，感觉力量涌上心头！");
        }

        // 卸下时触发
        fn on_remove(player_id) {
            send_msg(player_id, "你卸下了装备，力量消散。");
        }
    "#;

    let ast = engine.compile(equip_script)?;

    // 获取装备属性
    let stats: EquipStats = engine.call_fn(&ast, "get_stats", ())?;
    println!("  装备名称: {}", stats.name);
    println!("  装备类型: {}", stats.equip_type);
    println!("  攻击力: {}", stats.attack);
    println!("  特效: {}", stats.special.join(", "));
    println!();

    // 测试攻击特效
    println!("【测试4】测试攻击特效:");
    let damage = 1000;
    println!("  原始伤害: {}", damage);

    // 模拟多次攻击，看看是否有暴击
    for i in 1..=5 {
        let result = engine.call_fn::<i64>(&ast, "on_hit", (100, damage))?;
        println!("  第{}次攻击伤害: {}", i, result);
    }
    println!();

    // ========================================
    // 测试5: 房间脚本示例
    // ========================================
    println!("【测试5】房间脚本:");

    let room_script = r#"
        fn on_enter(player_id) {
            let name = query_name(player_id);
            log(name + " 进入了房间");

            // 进入房间回复 HP
            let current_hp = query_hp(player_id);
            if current_hp < 100 {
                add_hp(player_id, 10);
                send_msg(player_id, "你感觉体力正在恢复...");
            }
        }

        fn on_leave(player_id) {
            let name = query_name(player_id);
            log(name + " 离开了房间");
        }
    "#;

    let room_ast = engine.compile(room_script)?;
    let _: () = engine.call_fn(&room_ast, "on_enter", (1001,))?;
    let _: () = engine.call_fn(&room_ast, "on_leave", (1001,))?;
    println!();

    // ========================================
    // 测试6: 玩家打造装备时的临时脚本
    // ========================================
    println!("【测试6】玩家打造装备（自定义脚本）:");

    // 玩家输入的脚本（比如通过 forge 命令）
    let player_script = r#"
        fn get_stats() {
            #{
                name: "自定义魔剑",
                equip_type: "weapon",
                attack: 888,
                defense: 0,
                special: ["自定义特效"]
            }
        }

        fn on_hit(target_id, damage) {
            // 简单的反伤效果
            damage + random(10, 50)
        }
    "#;

    let player_ast = engine.compile(player_script)?;
    let player_stats: EquipStats = engine.call_fn(&player_ast, "get_stats", ())?;
    println!("  玩家打造的装备: {}", player_stats.name);
    println!("  攻击力: {}", player_stats.attack);

    // 测试自定义特效
    let custom_damage = engine.call_fn::<i64>(&player_ast, "on_hit", (200, 500))?;
    println!("  测试伤害（带随机加成）: {}", custom_damage);
    println!();

    // ========================================
    // 测试7: 复杂逻辑 - 打造系统
    // ========================================
    println!("【测试7】打造系统脚本:");

    let forge_script = r#"
        // 计算打造成功率
        fn calc_success_rate(player_level, material_quality) {
            let base_rate = 50;
            let level_bonus = player_level * 2;
            let material_bonus = material_quality * 5;
            let rate = base_rate + level_bonus + material_bonus;

            if rate > 95 { rate = 95; }
            rate
        }

        // 执行打造
        fn forge(player_id, player_level, material_name, material_quality) {
            let rate = calc_success_rate(player_level, material_quality);
            let roll = random_percent();

            log("打造成功率: " + rate + "%，骰子: " + roll);

            if roll <= rate {
                send_msg(player_id, "恭喜！使用 " + material_name + " 打造成功！");
                return true;
            } else {
                send_msg(player_id, "很遗憾，打造失败了...");
                return false;
            }
        }
    "#;

    let forge_ast = engine.compile(forge_script)?;

    // 模拟打造
    let success: bool = engine.call_fn(&forge_ast, "forge", (1001, 50, "寒铁", 8))?;
    println!("  打造结果: {}", if success { "成功" } else { "失败" });
    println!();

    println!("=== 所有测试完成 ===");

    Ok(())
}
