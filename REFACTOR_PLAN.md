# RustMUD 架构重构计划
## 参考 txpike9 经典MUDLib架构

## txpike9 目录结构分析

```
pikenv/wapmud2/
├── cmds/           # 命令文件 - 每个命令一个.pike文件
│   ├── look.pike
│   ├── attack.pike
│   ├── inventory.pike
│   ├── skill.pike
│   └── ...
├── clone/          # 可克隆对象模板
│   ├── weapon/
│   ├── armor/
│   ├── food/
│   └── ...
├── inherit/        # 继承模块
│   └── feature/    # 特性模块 (mixin-style组合)
│       ├── fight.pike      # 战斗特性
│       ├── inventory.pike  # 背包特性
│       ├── skills.pike     # 技能特性
│       ├── equip.pike      # 装备特性
│       ├── exits.pike      # 出口特性
│       └── char.pike       # 角色特性
├── single/         # 单例对象 (守护进程)
│   ├── master.pike         # 玩家对象
│   ├── user.pike           # 用户对象
│   ├── room.pike           # 房间对象
│   ├── npc.pike            # NPC对象
│   ├── weapon.pike         # 武器对象
│   └── daemons/            # 守护进程目录
│       ├── bangd.pike      # 帮派守护
│       ├── viewd.pike      # 视图守护
│       └── ...
├── d/              # 地图数据 (rooms data)
│   ├── xinshoucun/
│   ├── xiuwu/
│   └── ...
└── templates/      # 模板数据
```

## Rust实现架构

```
src/gamenv/
├── traits/                    # Trait特性系统 (对应 inherit/feature)
│   ├── mod.rs
│   ├── fight.rs              # Fight trait - 战斗能力
│   ├── inventory.rs          # Inventory trait - 背包能力
│   ├── skills.rs             # Skills trait - 技能能力
│   ├── equip.rs              # Equip trait - 装备能力
│   ├── movable.rs            # Movable trait - 移动能力
│   └── talkable.rs           # Talkable trait - 对话能力
│
├── entities/                  # 实体模块 (使用trait组合)
│   ├── mod.rs
│   ├── character.rs          # 角色基类
│   ├── player.rs             # 玩家 (Fight + Inventory + Skills + Equip)
│   ├── npc.rs                # NPC (Fight + Equip + Talkable)
│   └── room.rs               # 房间 (Movable + Exits)
│
├── clone/                     # 可克隆对象模板 (对应 txpike9/clone/)
│   ├── mod.rs
│   ├── weapon/
│   │   ├── mod.rs
│   │   ├── base.rs           # 基础武器
│   │   └── data.rs           # 武器数据
│   ├── armor/
│   ├── food/
│   └── medicine/
│
├── cmds/                      # 命令模块 (对应 txpike9/cmds/)
│   ├── mod.rs                # 命令路由
│   ├── look.rs               # look命令
│   ├── attack.rs             # attack命令
│   ├── pk.rs                 # pk战斗命令
│   ├── move.rs               # 移动命令
│   ├── inventory.rs          # 背包命令
│   ├── skills.rs             # 技能命令
│   ├── school.rs             # 门派命令
│   └── ...
│
├── single/                    # 单例对象 (对应 txpike9/single/)
│   ├── mod.rs
│   ├── master.rs             # 玩家对象
│   ├── user.rs               # 用户对象
│   ├── room.rs               # 房间对象
│   ├── npc.rs                # NPC对象
│   └── daemons/              # 守护进程目录
│       ├── mod.rs
│       ├── pkd.rs            # PK守护
│       ├── itemd.rs          # 物品守护
│       ├── bangd.rs          # 帮派守护
│       ├── schoold.rs        # 门派守护
│       ├── skilld.rs         # 技能守护
│       └── ...
│
├── world/                     # 世界数据 (对应 txpike9/d/)
│   ├── mod.rs
│   ├── rooms/                # 房间数据
│   │   ├── xinshoucun/
│   │   ├── xiuwu/
│   │   └── ...
│   └── npcs/                 # NPC数据
│
├── output/                    # 输出格式化
│   ├── mod.rs
│   ├── mud.rs                # MUD格式 (颜色代码)
│   └── json.rs               # JSON格式
│
└── http_api/                  # HTTP API (仅路由)
    ├── mod.rs                # 路由定义
    └── handlers.rs           # 请求处理 (转发到cmds)
```

## Trait系统设计

### Fight Trait
```rust
pub trait Fight {
    async fn attack(&mut self, target: &str) -> Result<(), String>;
    async fn take_damage(&mut self, damage: i32) -> bool;
    fn hp(&self) -> i32;
    fn hp_max(&self) -> i32;
    fn is_alive(&self) -> bool;
}
```

### Inventory Trait
```rust
pub trait Inventory {
    async fn add_item(&mut self, item: Item) -> Result<(), String>;
    async fn remove_item(&mut self, item_id: &str) -> Result<Item, String>;
    fn items(&self) -> &[Item];
    async fn list_items(&self) -> String;
}
```

### Skills Trait
```rust
pub trait Skills {
    async fn learn_skill(&mut self, skill_id: &str) -> Result<(), String>;
    async fn use_skill(&mut self, skill_id: &str) -> Result<(), String>;
    fn skills(&self) -> &HashMap<String, u32>;
}
```

## 已完成
- ✅ 分析 txpike9 目录结构
- ✅ 创建 cmds/ 目录结构
- ✅ 创建基础命令文件

## 进行中
- 🔄 设计 Trait 特性系统
- 🔄 设计 entities/ 模块

## 待完成
- ⏳ 创建 clone/ 模块 (可克隆对象)
- ⏳ 创建 single/daemons/ 模块
- ⏳ 创建 output/ 格式化模块
- ⏳ 创建 traits/ 特性系统
- ⏳ 创建 entities/ 实体模块
- ⏳ 重构 HTTP API 使用新模块
- ⏳ 测试和部署

## 实现步骤

1. **Phase 1: 模块分离** (当前阶段)
   - ✅ 创建 cmds/ 目录
   - ⏳ 创建 clone/ 目录
   - ⏳ 创建 traits/ 目录
   - ⏳ 创建 entities/ 目录

2. **Phase 2: Trait系统**
   - 实现 Fight trait
   - 实现 Inventory trait
   - 实现 Skills trait
   - 实现 Equip trait

3. **Phase 3: 实体重构**
   - 重构 Player 使用 trait 组合
   - 重构 NPC 使用 trait 组合
   - 重构 Room 使用 trait 组合

4. **Phase 4: 输出分离**
   - 创建统一的输出格式化器
   - 分离 MUD 输出和 JSON 输出

5. **Phase 5: HTTP API 简化**
   - HTTP API 只负责路由
   - 命令逻辑转移到 cmds/
   - 业务逻辑转移到 entities/ 和 daemons/
