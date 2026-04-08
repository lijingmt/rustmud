# RustMUD - A Rust MUD Engine

## 概述

RustMUD 是 txpike9 MUD 引擎的 1:1 Rust 移植版本。该项目旨在用 Rust 的高性能和安全性重写 txpike9，同时保持相同的架构和设计模式，方便未来维护。

## 架构

```
rustmud/
├── src/
│   ├── main.rs           # 主入口
│   ├── core/             # 核心类型系统
│   │   ├── mod.rs        # 核心模块 (ObjectId, Frame, Backtrace)
│   │   ├── object.rs     # 对象系统 (对应 Pike 的 object)
│   │   ├── mapping.rs    # Mapping 类型 (对应 Pike 的 mapping)
│   │   ├── array.rs      # Array 类型 (对应 Pike 的 array)
│   │   ├── value.rs      # Value 类型 (对应 Pike 的 mixed)
│   │   ├── error.rs      # 错误处理 (对应 Pike 的 handle_error)
│   │   └── program.rs    # 程序加载 (对应 Pike 的 program)
│   ├── pikenv/           # Pike 环境 (对应 txpike9/pikenv/)
│   │   ├── mod.rs
│   │   ├── pikenv.rs     # 主服务器入口 (对应 pikenv.pike)
│   │   ├── conn.rs       # 连接处理 (对应 conn.pike)
│   │   ├── connd.rs      # 连接管理器 (对应 connd.pike)
│   │   ├── efuns.rs      # 内置函数 (对应 efuns.pike)
│   │   ├── config.rs     # 配置系统
│   │   └── gc_manager.rs # GC 管理器
│   └── gamenv/           # 游戏环境 (对应 txpike9/gamenv/)
│       ├── mod.rs
│       ├── master.rs     # 主控制器 (对应 master.pike)
│       ├── user.rs       # 用户对象 (对应 clone/user)
│       ├── cmds.rs       # 命令系统
│       ├── daemons.rs    # Daemon 系统
│       ├── inherit.rs    # 继承基类
│       ├── d.rs          # 房间/世界
│       ├── clone.rs      # 可克隆对象
│       └── data.rs       # 数据定义
```

## 对应关系

| txpike9 (Pike) | rustmud (Rust) | 说明 |
|----------------|----------------|------|
| pikenv.pike | pikenv/pikenv.rs | 主入口 |
| conn.pike | pikenv/conn.rs | 连接处理 |
| efuns.pike | pikenv/efuns.rs | 内置函数 |
| master.pike | gamenv/master.rs | 主控制器 |
| object | core/object.rs | 对象系统 |
| mapping | core/mapping.rs | 键值映射 |
| save_object() | serde Serialize | 对象序列化 |
| restore_object() | serde Deserialize | 对象反序列化 |

## 功能对比

| 功能 | Pike 版本 | Rust 版本 | 优势 |
|------|-----------|-----------|------|
| 性能 | 解释执行 | 编译执行 (AOT) | 接近 C/C++ |
| 并发 | 单线程 + 协程 | 多线程 + 异步 (Tokio) | 真正的多核利用 |
| 内存 | 引用计数 + GC | 编译时安全 | 无 GC 暂停 |
| 类型 | 动态类型 | 静态类型 | 编译时检查 |
| 部署 | Pike 运行时 | 单一二进制 | 简化部署 |

## 快速开始

```bash
# 编译
cargo build --release

# 运行
cargo run --release

# 设置环境变量
export GAME_AREA=tx01
export ROOT=/path/to/mudlib
export PORT=9999
```

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| GAME_AREA | tx01 | 游戏区号 |
| ROOT | 当前目录 | Mudlib 根目录 |
| PORT | 9999 | 监听端口 |
| IP | 0.0.0.0 | 监听 IP |
| LOG_PREFIX | 9999 | 日志文件前缀 |

## 依赖

- **tokio** - 异步运行时
- **serde** - 序列化/反序列化
- **sqlx** - 数据库
- **axum** - HTTP API

## 开发状态

当前为初始开发阶段，已完成核心框架的搭建。

## 许可证

MIT License

## 参考

- 原项目: [txpike9](https://github.com/your-org/txpike9)
