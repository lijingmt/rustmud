// gamenv/cmds/look.rs - look命令
// 对应 txpike9/wapmud2/cmds/look.pike

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock as TokioRwLock;

use crate::gamenv::world::GameWorld;
use crate::gamenv::core::command::*;
use crate::gamenv::player_state;

/// 查看房间命令
pub async fn look_command(world: &GameWorld, room_id: &str) -> String {
    use crate::gamenv::single::daemons::runtime_npc_d::get_runtime_npc_d;

    if let Some(room) = world.get_room(room_id) {
        let mut output = format!("§Y{}§N\n", room.name);
        output.push_str(&format!("{}\n", room.long.trim()));

        // 初始化房间NPC（如果还没初始化）
        {
            let runtime_npc_d = get_runtime_npc_d().read().await;
            if !room.npcs.is_empty() && runtime_npc_d.get_all_npcs(room_id).is_empty() {
                drop(runtime_npc_d);
                let mut runtime_npc_d_write = get_runtime_npc_d().write().await;
                runtime_npc_d_write.init_room_npcs(room_id, &room.npcs);
            }
        }

        // 获取运行时NPC守护进程
        let runtime_npc_d = get_runtime_npc_d().read().await;

        // 显示NPC - 过滤已死亡的
        let alive_npcs = runtime_npc_d.get_alive_npcs(room_id);

        if !alive_npcs.is_empty() {
            output.push_str("\n§C人物§N\n");
            for npc_id in &alive_npcs {
                if let Some(npc) = world.get_npc(npc_id) {
                    output.push_str(&format!("{}\n", npc.format_short()));
                }
            }
        }

        // 显示怪物
        let alive_monsters: Vec<_> = room.monsters.iter()
            .filter(|m| runtime_npc_d.is_npc_alive(m, room_id))
            .collect();

        if !alive_monsters.is_empty() {
            output.push_str("\n§R怪物§N\n");
            for monster_id in &alive_monsters {
                if let Some(monster) = world.get_npc(monster_id) {
                    output.push_str(&format!("{}\n", monster.short));
                }
            }
        }

        output
    } else {
        "无法找到当前房间。".to_string()
    }
}

/// 查看NPC详情命令
pub async fn look_npc_command(world: &GameWorld, room_id: &str, target: &str) -> String {
    let mut output = String::new();
    let mut found_npc = false;

    if let Some(room) = world.get_room(room_id) {
        // 先检查房间内的NPC
        for npc_id in &room.npcs {
            if let Some(npc) = world.get_npc(npc_id) {
                if npc.id.contains(target) || npc.name.contains(target) || npc.short.contains(target) {
                    found_npc = true;
                    output.push_str(&format!("§Y{}§N\n\n", npc.name));
                    output.push_str(&format!("§C等级§N {}\n", npc.level));
                    output.push_str(&format!("§C描述§N {}\n\n", npc.long));

                    // 操作按钮
                    output.push_str(&format!("§H操作§N\n"));

                    // 对话按钮
                    output.push_str(&format!("[对话:talk {}]\n", npc.id));

                    // 商店按钮（如果是商人）
                    if npc.shop.is_some() {
                        output.push_str(&format!("[商店:shop {}]\n", npc.id));
                    }

                    // 攻击按钮（如果是怪物或敌对NPC）
                    if room.monsters.contains(&npc.id) {
                        output.push_str(&format!("[§R攻击§N:kill {}]\n", npc.id));
                    }

                    // PK按钮
                    output.push_str(&format!("[§RPK§N:pk {}]\n", npc.id));

                    // 任务按钮
                    output.push_str(&format!("[任务:quest {}]\n", npc.id));

                    output.push_str(&format!("\n[返回房间:look]\n"));
                    break;
                }
            }
        }

        // 如果NPC列表中没找到，检查怪物列表
        if !found_npc {
            for monster_id in &room.monsters {
                if let Some(monster) = world.get_npc(monster_id) {
                    if monster.id.contains(target) || monster.name.contains(target) || monster.short.contains(target) {
                        found_npc = true;
                        output.push_str(&format!("§Y{}§N\n\n", monster.name));
                        output.push_str(&format!("§C等级§N {}\n", monster.level));
                        output.push_str(&format!("§C生命值§N {}/{}\n", monster.hp, monster.hp_max));
                        output.push_str(&format!("§C描述§N {}\n\n", monster.long));

                        // 怪物只有攻击选项
                        output.push_str(&format!("§H操作§N\n"));
                        output.push_str(&format!("[§R攻击§N:kill {}]\n", monster.id));

                        output.push_str(&format!("\n[返回房间:look]\n"));
                        break;
                    }
                }
            }
        }
    }

    if found_npc {
        output
    } else {
        "这里没有这个目标。\n".to_string()
    }
}

/// Look 命令处理器（新命令系统）
pub struct LookCommand;

/// 辅助函数：使用 RwLockReadGuard 调用 look_command
async fn look_with_guard(world_guard: &TokioRwLock<GameWorld>, room_id: &str, args: &[String]) -> String {
    let world = world_guard.read().await;
    if args.is_empty() {
        look_command(&world, room_id).await
    } else {
        let target = args.join(" ");
        look_npc_command(&world, room_id, &target).await
    }
}

#[async_trait]
impl CommandHandler for LookCommand {
    async fn handle(&self, ctx: CommandContext) -> CommandResult {
        let world_ref = crate::gamenv::world::get_world();
        let output = look_with_guard(&world_ref, &ctx.room_id, &ctx.args).await;
        CommandResult::from(output)
    }

    fn metadata(&self) -> &CommandMetadata {
        static META: once_cell::sync::Lazy<CommandMetadata> =
            once_cell::sync::Lazy::new(|| {
                CommandMetadata::new(
                    "look",
                    "查看周围环境或指定目标",
                    CommandCategory::Info
                )
                .with_aliases(&["l"])
                .with_args(0, None)
            });
        &META
    }
}

/// 获取 Look 命令实例
pub fn get_command() -> Arc<dyn CommandHandler> {
    Arc::new(LookCommand) as Arc<dyn CommandHandler>
}
