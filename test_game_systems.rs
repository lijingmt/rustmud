// test_game_systems.rs - 测试游戏系统
// 测试所有已实现的游戏模块

use rustmud::gamenv::combat::*;
use rustmud::gamenv::combat::skill::*;
use rustmud::gamenv::item::*;
use rustmud::gamenv::item::equipment::*;
use rustmud::gamenv::npc::*;
use rustmud::gamenv::quest::*;
use rustmud::gamenv::guild::*;
use rustmud::gamenv::d::*;
use rustmud::gamenv::user::*;

fn main() {
    println!("=== RustMUD 游戏系统测试 ===\n");

    // 测试1: 房间/地图系统
    test_map_system();

    // 测试2: 装备系统
    test_equipment_system();

    // 测试3: 战斗系统
    test_combat_system();

    // 测试4: 技能系统
    test_skill_system();

    // 测试5: 物品系统
    test_item_system();

    // 测试6: NPC系统
    test_npc_system();

    // 测试7: 任务系统
    test_quest_system();

    // 测试8: 帮派系统
    test_guild_system();

    // 测试9: 用户系统
    test_user_system();

    println!("\n=== 所有测试完成 ===");
}

fn test_map_system() {
    println!("【1. 测试地图系统】");
    let mut map_mgr = MapManager::new();
    map_mgr.create_starter_rooms();
    map_mgr.create_beijing_rooms();

    println!("  - 新手村和北京地图已创建");

    // 测试方向解析
    assert_eq!(normalize_direction("北"), "north");
    assert_eq!(normalize_direction("n"), "north");
    assert_eq!(normalize_direction("south"), "south");

    println!("  - 方向解析测试通过: 北->north, n->north, south->south\n");
}

fn test_equipment_system() {
    println!("【2. 测试装备系统】");

    // 测试装备槽位
    let slot = EquipSlot::Weapon;
    println!("  - 装备槽: {}", slot.name());

    // 测试装备境界
    let realm = EquipRealm::from_level(50);
    println!("  - 等级50对应境界: {} {}", realm.color_code(), realm.name());

    // 测试装备属性计算
    let stats = EquipStats::calculate(50, ItemQuality::Rare, EquipSlot::Weapon);
    println!("  - 稀有武器属性: 攻击{} 暴击{}%", stats.attack, stats.crit_rate);

    // 测试装备栏
    let mut equip_slots = EquipmentSlots::default();
    let equip = Equipment::new("iron_sword".to_string(), "铁剑".to_string(), EquipSlot::Weapon);
    equip_slots.equip(EquipSlot::Weapon, equip).unwrap();

    let total_stats = equip_slots.total_stats();
    println!("  - 装备栏总属性: 攻击{}\n", total_stats.attack);
}

fn test_combat_system() {
    println!("【3. 测试战斗系统】");

    // 创建两个战斗单位的属性
    let attacker_stats = CombatStats::for_level(10);
    let defender_stats = CombatStats::for_level(10);

    println!("  - 攻击者: HP{} 攻击{} 防御{}", attacker_stats.hp, attacker_stats.attack, attacker_stats.defense);
    println!("  - 防御者: HP{} 攻击{} 防御{}", defender_stats.hp, defender_stats.attack, defender_stats.defense);

    // 计算伤害
    let result = attacker_stats.calculate_damage(&defender_stats);
    println!("  - 伤害结果: {} (暴击:{}, 未命中:{}, 闪避:{})",
        result.damage, result.is_crit, result.is_miss, result.is_dodge);

    // 渲染伤害描述
    println!("  - 伤害描述: {}\n", result.description());
}

fn test_skill_system() {
    println!("【4. 测试技能系统】");

    // 创建技能管理器
    let skill_mgr = SkillManager::new();

    // 获取技能
    if let Some(skill) = skill_mgr.get_skill("skill_power_strike") {
        println!("  - 技能: {}", skill.name_cn);
        println!("  - 内力消耗: {}", skill.qi_cost);
        println!("  - 冷却时间: {}回合", skill.cooldown);

        // 测试技能检查
        let can_use = skill.can_use(100, 10);
        println!("  - 能否使用(100内力,10级): {}\n", can_use);
    }
}

fn test_item_system() {
    println!("【5. 测试物品系统】");

    // 创建物品
    let item = Item::new("health_potion".to_string(), "生命药水".to_string(), ItemType::Medicine)
        .with_quality(ItemQuality::Uncommon)
        .with_level(10)
        .with_quantity(5)
        .with_max_stack(99);

    println!("  - 物品名称: {}", item.render_name());

    // 测试物品品质
    let quality = ItemQuality::from_level(50);
    println!("  - 等级50对应品质: {}", quality.name());

    // 测试物品堆叠
    let item2 = Item::new("health_potion".to_string(), "生命药水".to_string(), ItemType::Medicine)
        .with_quality(ItemQuality::Uncommon)
        .with_level(10)
        .with_quantity(3)
        .with_max_stack(99);

    let can_stack = item.can_stack_with(&item2);
    println!("  - 能否堆叠: {}\n", can_stack);
}

fn test_npc_system() {
    println!("【6. 测试NPC系统】");

    // 创建NPC (使用NpcBase)
    let npc = NpcBase::new("npc_shopkeeper".to_string(), "店小二".to_string(), NpcType::Normal);
    println!("  - NPC名称: {} (Lv.{})", npc.name_cn, npc.level);

    // 创建怪物 (使用Monster::new with 3 params)
    let monster = Monster::new("slime".to_string(), "史莱姆".to_string(), 5);
    println!("  - 怪物: {} (Lv.{}, {})", monster.npc.base.name_cn, monster.npc.base.level, monster.rarity.name());

    // 渲染怪物属性 - 使用combat字段
    println!("  - 怪物HP: {} 攻击: {}", monster.npc.combat.hp, monster.npc.combat.attack);
}

fn test_quest_system() {
    println!("【7. 测试任务系统】");

    // 创建任务
    let quest = Quest::new("quest_test".to_string(), "测试任务".to_string(), QuestType::Main)
        .with_description("这是一个测试任务。".to_string())
        .with_objective(QuestObjective::new(QuestObjectiveType::KillMonster, "slime".to_string(), 10));

    println!("  - 任务: {} ({})", quest.name_cn, quest.id);
    println!("  - 任务描述: {}", quest.description);

    // 测试任务接受
    let mut quest_clone = quest.clone();
    quest_clone.accept().unwrap();
    println!("  - 任务状态: {:?}", quest_clone.status);

    // 测试任务类型
    println!("  - 任务类型: {:?}", quest_clone.quest_type);

    println!("  - 任务渲染:\n{}", quest_clone.render_info());
}

fn test_guild_system() {
    println!("【8. 测试帮派系统】");

    // 创建帮派
    let guild = Guild::new("guild_test".to_string(), "测试帮派".to_string(), "player1".to_string(), "帮主".to_string());
    println!("  - 帮派名称: {}", guild.name);
    println!("  - 帮主: {}", guild.leader_id);
    println!("  - 最大成员数: {}", guild.max_members);
    println!("  - 当前成员数: {}", guild.member_count());

    // 测试帮派职位
    println!("  - 帮主权限: 可解散={}, 可邀请={}", GuildRank::Leader.can_disband(), GuildRank::Leader.can_invite());

    // 测试玩家帮派数据
    let mut player_guild = PlayerGuildData::default();
    player_guild.join_guild("guild_test".to_string());
    println!("  - 玩家已加入帮派: {}\n", player_guild.has_guild());
}

fn test_user_system() {
    println!("【9. 测试用户系统】");

    // 创建用户
    let mut user = User::new("testuser".to_string());
    println!("  - 用户名: {}", user.name_cn);
    println!("  - 等级: {}", user.level);
    println!("  - HP: {}/{}", user.hp, user.hp_max);
    println!("  - 内力: {}/{}", user.qi, user.qi_max);

    // 测试物品添加
    let item = Item::new("gold".to_string(), "金币".to_string(), ItemType::Money);
    user.add_item(item).unwrap();

    // 测试装备
    let weapon_item = Item::new("iron_sword".to_string(), "铁剑".to_string(), ItemType::Weapon)
        .with_quality(ItemQuality::Common)
        .with_level(1);
    user.add_item(weapon_item).unwrap();

    // 测试战斗属性
    let total_stats = user.get_total_stats();
    println!("  - 总攻击力: {}", total_stats.attack);
    println!("  - 总防御力: {}", total_stats.defense);

    // 测试Combatant trait
    println!("  - 玩家名称: {}", user.get_name());
    println!("  - 玩家等级: {}", user.get_level());
    println!("  - 玩家存活: {}", user.is_alive());

    // 渲染玩家状态
    println!("  - 玩家状态:\n{}", user.render_status());
}
