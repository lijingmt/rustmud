# iOS 单机版 Roadmap

## 项目概述

将 Rust MUD 游戏引擎打包为 iOS 单机应用，保留完整的游戏逻辑和数据，无需网络连接即可游玩。

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────┐
│         iOS App (.ipa)                  │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │   Swift/Objective-C 层          │   │
│  │   - AppDelegate                 │   │
│  │   - WebView Controller          │   │
│  └──────────────┬──────────────────┘   │
│                 │                       │
│  ┌──────────────▼──────────────────┐   │
│  │   Rust 静态库 (librustmud.a)     │   │
│  │   - Tokio Runtime               │   │
│  │   - Axum HTTP Server            │   │
│  │   - WebSocket                   │   │
│  │   - 游戏引擎核心                 │   │
│  │   - 文件数据加载                 │   │
│  └──────────────┬──────────────────┘   │
│                 │                       │
│  ┌──────────────▼──────────────────┐   │
│  │   WebView (Vue 前端)             │   │
│  │   http://localhost:4000          │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │   游戏数据 (Bundle Resources)    │   │
│  │   - data/world/rooms_data.json   │   │
│  │   - data/world/npcs_data.json    │   │
│  │   - data/world/items_data.json   │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue.js (现有代码) + WKWebView |
| 后端 | Rust (现有代码) |
| 通信 | HTTP/WebSocket (localhost) |
| 数据存储 | JSON 文件 + 内存 |
| iOS 集成 | Swift + FFI |

### 为什么可行？

1. **Rust 使用 LLVM** - 与 Swift/Clang 共享后端，生成兼容的 ARM64 机器码
2. **iOS 允许本地服务器** - 可以绑定 localhost 端口
3. **纯文件存储** - 无需数据库，简化部署
4. **零架构改动** - 保留完整的 HTTP API

## 实现步骤

### Phase 1: Rust 端改造

#### 1.1 添加 iOS 启动函数

创建 `src/ios_launcher.rs`:

```rust
use std::ffi::CString;
use std::os::raw::c_char;

/// 启动 MUD 服务器（供 iOS 调用）
#[no_mangle]
pub extern "C" fn rustmud_start_server() -> *mut c_char {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime");

        rt.block_on(async {
            // 启动 HTTP API 服务器
            if let Err(e) = rustmud::gamenv::http_api::start_server().await {
                eprintln!("Server error: {}", e);
            }
        });
    });

    CString::new("Server started").unwrap().into_raw()
}

/// 停止服务器
#[no_mangle]
pub extern "C" fn rustmud_stop_server() {
    // TODO: 实现优雅关闭
}
```

#### 1.2 修改 Cargo.toml

```toml
[lib]
name = "rustmud"
path = "src/lib.rs"
crate-type = ["staticlib", "cdylib"]

[[bin]]
name = "rustenv"
path = "src/main.rs"
```

#### 1.3 移除数据库依赖

```toml
# 删除或注释掉 sqlx 依赖
# sqlx = { version = "0.8", features = ["runtime-tokio", "mysql", "chrono", "uuid"] }
```

#### 1.4 修改 src/lib.rs

```rust
pub mod ios_launcher;
pub mod gamenv;
// ... 其他模块
```

### Phase 2: 编译 iOS 静态库

```bash
# 安装 iOS target
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim

# 编译静态库
cargo build --target aarch64-apple-ios --release

# 生成文件: target/aarch64-apple-ios/release/librustmud.a
```

### Phase 3: iOS 项目创建

#### 3.1 项目结构

```
RustMudiOS/
├── RustMudiOS/
│   ├── AppDelegate.swift
│   ├── WebViewController.swift
│   ├── Info.plist
│   └── RustMud-Bridging-Header.h
├── RustMudiOS.xcodeproj
└── RustMud.xcworkspace
```

#### 3.2 Bridging Header

```objectivec
// RustMud-Bridging-Header.h

#ifndef RustMud_Bridging_Header_h
#define RustMud_Bridging_Header_h

void rustmud_start_server(void);
void rustmud_stop_server(void);

#endif
```

#### 3.3 AppDelegate.swift

```swift
import UIKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {

        // 启动 Rust 服务器
        startRustServer()

        // 创建窗口
        window = UIWindow(frame: UIScreen.main.bounds)

        // 创建 WebView Controller
        let webVC = WebViewController()
        window?.rootViewController = webVC
        window?.makeKeyAndVisible()

        return true
    }

    private func startRustServer() {
        DispatchQueue.global(qos: .background).async {
            rustmud_start_server()
            print("Rust MUD server started on localhost:4000")
        }

        // 等待服务器启动
        Thread.sleep(forTimeInterval: 1.0)
    }

    func applicationWillTerminate(_ application: UIApplication) {
        rustmud_stop_server()
    }
}
```

#### 3.4 WebViewController.swift

```swift
import WebKit

class WebViewController: UIViewController, WKNavigationDelegate {

    private var webView: WKWebView!

    override func viewDidLoad() {
        super.viewDidLoad()

        let config = WKWebViewConfiguration()
        config.preferences.javaScriptEnabled = true

        webView = WKWebView(frame: view.bounds, configuration: config)
        webView.navigationDelegate = self
        view.addSubview(webView)

        loadLocalFrontend()
    }

    private func loadLocalFrontend() {
        // 加载打包到 Bundle 的 Vue 文件
        if let url = Bundle.main.url(
            forResource: "index",
            withExtension: "html",
            subdirectory: "web_vue/dist"
        ) {
            webView.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent())
        }
    }
}
```

### Phase 4: Xcode 配置

#### 4.1 Build Settings

```
Header Search Paths:
- $(PROJECT_DIR)/rust/target/aarch64-apple-ios/release
- $(PROJECT_DIR)/rust/src

Other Linker Flags:
- -lrustmud
- -framework Security
```

#### 4.2 Build Phases

**Link Binary With Libraries:**
- `librustmud.a`
- `Security.framework`
- `WebKit.framework`

**Copy Bundle Resources:**
- `data/world/rooms_data.json`
- `data/world/npcs_data.json`
- `data/world/items_data.json`
- `web_vue/dist/` (整个目录)

### Phase 5: 打包和测试

```bash
# Archive
xcodebuild archive -scheme RustMudiOS -archivePath build/RustMudiOS.xcarchive

# Export
xcodebuild -exportArchive -archivePath build/RustMudiOS.xcarchive \
  -exportPath build/RustMudiOS -exportOptionsPlist ExportOptions.plist
```

## 数据持久化方案

### 存档位置

iOS 沙盒 Documents 目录：

```
RustMudiOS/
└── Documents/
    └── saves/
        ├── save_001.json
        ├── save_002.json
        └── save_003.json
```

### 存档格式

```json
{
  "player": {
    "id": "uuid",
    "name": "玩家名",
    "level": 10,
    "exp": 5000,
    "hp": 100,
    "max_hp": 100,
    "location": "room_id_123"
  },
  "world_state": {
    "completed_quests": ["quest_001"],
    "killed_npcs": ["npc_456"]
  },
  "timestamp": 1713825600
}
```

## 性能考虑

| 指标 | 目标 |
|------|------|
| App 大小 | < 20 MB |
| 启动时间 | < 2 秒 |
| 内存占用 | < 100 MB |
| 命令响应 | < 50ms |

## 已知限制

1. **后台运行** - App 切到后台时服务器可能暂停
2. **持久化** - 需要手动保存，应用可能被杀死
3. **多设备同步** - 单机版不支持

## 后续扩展

### 可选功能

- [ ] Game Center 集成
- [ ] iCloud 存档同步
- [ ] 可选的在线模式
- [ ] IAP 内购系统

### 跨平台

- [ ] Android 版本 (完全相同的 Rust 后端)
- [ ] macOS 版本 (Catalyst)

## 参考

- [napi-rs](https://napi.rs/) - React Native + Rust
- [cargo-mobile](https://github.com/brainiumlabs/cargo-mobile) - Rust 移动开发工具
- [uniffi-rs](https://mozilla.github.io/uniffi-rs/) - Mozilla 的跨平台 FFI

## 更新日志

- 2025-04-22: 初始 roadmap，讨论 iOS 单机版架构方案
