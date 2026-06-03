# Claude Code 桌面宠物 - 详细设计方案

## 项目概述

开发一个跨平台（Windows/macOS/Linux）的 Rust 桌面宠物应用，以卡通玄凤鹦鹉形象实时显示 Claude Code 的工作状态。

---

## 1. 技术架构

### 1.1 整体架构

```
┌──────────────────────────────────────────────────────────────┐
│                       Claude Code                            │
│                          │                                   │
│          ┌───────────────┼───────────────┐                   │
│          │               │               │                   │
│          ▼               ▼               ▼                   │
│   UserPromptSubmit  PreToolUse     PostToolUse               │
│   Notification      Stop            SubAgentStop             │
│          │               │               │                   │
│          └───────────────┼───────────────┘                   │
│                          ▼                                   │
│                  hooks 调用状态脚本                           │
│                          │                                   │
│                          ▼                                   │
│              curl POST JSON 到本地 HTTP 服务                 │
└──────────────────────────────────────────────────────────────┘
                           │  HTTP POST (127.0.0.1:9527/status)
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                    Rust 桌面宠物                              │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              状态接收模块 (Monitor)                   │    │
│  │  ┌─────────────────────┐  ┌───────────────────────┐ │    │
│  │  │ HTTP Server         │  │ 状态解析              │ │    │
│  │  │ (tiny_http, 同步)   │  │ (serde)               │ │    │
│  │  └──────────┬──────────┘  └──────────┬────────────┘ │    │
│  │             └────────────────────────┘              │    │
│  │                          ▼                          │    │
│  │                   PetState 状态机                   │    │
│  └─────────────────────────┬───────────────────────────┘    │
│                            ▼                                 │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              动画模块 (Animation)                     │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │    │
│  │  │ 精灵图加载  │  │ 帧管理      │  │ 状态切换    │ │    │
│  │  │ (image)     │  │ (timer)     │  │ (state)     │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘ │    │
│  └─────────────────────────┬───────────────────────────┘    │
│                            ▼                                 │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              渲染模块 (Renderer)                      │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │    │
│  │  │ 透明窗口    │  │ 像素渲染    │  │ 窗口管理    │ │    │
│  │  │ (winit)     │  │ (pixels)    │  │ (drag)      │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘ │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              系统集成模块 (System)                    │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │    │
│  │  │ 系统托盘    │  │ 开机启动    │  │ 配置管理    │ │    │
│  │  │ (tray-icon) │  │ (autostart) │  │ (config)    │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘ │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 技术栈

| 组件 | 选择 | 版本 | 用途 |
|------|------|------|------|
| 窗口管理 | `winit` | 0.30 | 跨平台透明窗口 |
| 2D渲染 | `pixels` | 0.14 | 像素级渲染 |
| 图片加载 | `image` | 0.25 | PNG/GIF 解码 |
| HTTP 服务 | `tiny_http` | 0.12 | 本地 HTTP 状态接收 |
| 系统托盘 | `tray-icon` | 0.19 | 托盘菜单 |
| 进程检测 | `sysinfo` | 0.31 | 检测 Claude 进程 |
| 序列化 | `serde` + `serde_json` | 1.x | JSON 解析 |
| 路径管理 | `dirs` | 5.0 | 跨平台目录路径 |

---

## 2. 状态监听设计

### 2.1 状态定义

```rust
/// 宠物状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetState {
    /// Claude Code 未运行或已关闭
    Sleeping,
    /// 会话活跃，等待用户输入（SessionStart / Stop）
    Idle,
    /// 正在执行工具（PreToolUse）
    Working,
    /// 思考中（UserPromptSubmit / PostToolUse）
    Thinking,
    /// 等待用户权限确认（PermissionRequest）
    PendingApproval,
    /// 收到通知（Notification）
    Notify(String),
    /// 子代理工作中（SubagentStart）
    SubAgentWorking,
    /// 出错（PostToolUseFailure / StopFailure）
    Error,
}

impl Default for PetState {
    fn default() -> Self {
        PetState::Sleeping
    }
}
```

### 2.2 状态 JSON 格式

HTTP POST 请求体（`POST http://127.0.0.1:9527/status`）：

```json
{
    "state": "Working",
    "timestamp": 1717411200000,
    "session_id": "abc123",
    "message": null
}
```

端口可通过环境变量 `CLAUDE_POKE_PORT` 覆盖，默认 9527。

### 2.3 通信策略

```
Hook 脚本 → curl / Invoke-RestMethod → HTTP POST
                                          │
                                          ▼
                         tiny_http 本地服务 (127.0.0.1:9527)
                                          │
                                          ▼
                              mpsc channel → 主事件循环

优点：
  ├─ 零轮询，即时到达
  ├─ 依赖极简（tiny_http，同步，无异步运行时）
  ├─ hook 端用 curl 即可，无额外工具依赖
  └─ 便于未来桥接 BLE/WiFi（daemon 模式）
```

### 2.4 跨平台状态脚本

**状态更新脚本** `set-status.sh`（Linux/macOS）:
```bash
#!/bin/bash
PORT="${CLAUDE_POKE_PORT:-9527}"
TIMESTAMP=$(date +%s%N | cut -c1-13)
SESSION_ID="${CLAUDE_SESSION_ID:-unknown}"
STATE="${1:-Sleeping}"
MESSAGE="${2:-}"

STATE_JSON="\"$STATE\""
if [ "$STATE" = "Notify" ] && [ -n "$MESSAGE" ]; then
    STATE_JSON="{\"Notify\": \"$MESSAGE\"}"
fi

JSON="{\"state\":$STATE_JSON,\"timestamp\":$TIMESTAMP,\"session_id\":\"$SESSION_ID\",\"message\":\"$MESSAGE\"}"

curl -s -X POST "http://127.0.0.1:${PORT}/status" \
    -H "Content-Type: application/json" \
    -d "$JSON" \
    > /dev/null 2>&1 &
```

**状态更新脚本** `set-status.ps1`（Windows）:
```powershell
$port = if ($env:CLAUDE_POKE_PORT) { $env:CLAUDE_POKE_PORT } else { "9527" }
$timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$sessionId = if ($env:CLAUDE_SESSION_ID) { $env:CLAUDE_SESSION_ID } else { "unknown" }

# Handle Notify state with message as nested object
if ($args[0] -eq "Notify" -and $args[1]) {
    $stateValue = @{ Notify = $args[1] }
} else {
    $stateValue = $args[0]
}

$json = @{
    state = $stateValue
    timestamp = $timestamp
    session_id = $sessionId
    message = $args[1]
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "http://127.0.0.1:${port}/status" `
        -Method Post -Body $json -ContentType "application/json" `
        -TimeoutSec 2 -ErrorAction Stop | Out-Null
} catch {
    # Daemon not running — silently ignore
}
```

---

## 3. Claude Code Hooks 配置

### 3.1 settings.json 配置模板

**Linux/macOS** (`~/.claude/settings.json`):
```json
{
    "hooks": {
        "UserPromptSubmit": [
            {
                "hooks": [
                    {
                        "type": "command",
                        "command": "/path/to/set-status.sh Waiting"
                    }
                ]
            }
        ],
        "PreToolUse": [
            {
                "matcher": ".*",
                "hooks": [
                    {
                        "type": "command",
                        "command": "/path/to/set-status.sh Working"
                    }
                ]
            }
        ],
        "PostToolUse": [
            {
                "matcher": ".*",
                "hooks": [
                    {
                        "type": "command",
                        "command": "/path/to/set-status.sh Thinking"
                    }
                ]
            }
        ],
        "Notification": [
            {
                "hooks": [
                    {
                        "type": "command",
                        "command": "/path/to/set-status.sh Notify \"$MESSAGE\""
                    }
                ]
            }
        ],
        "Stop": [
            {
                "hooks": [
                    {
                        "type": "command",
                        "command": "/path/to/set-status.sh Stopped"
                    }
                ]
            }
        ],
        "SubAgentStart": [
            {
                "hooks": [
                    {
                        "type": "command",
                        "command": "/path/to/set-status.sh SubAgentWorking"
                    }
                ]
            }
        ]
    }
}
```

---

## 4. 玄凤鹦鹉形象设计

### 4.1 角色设定

```
名称：Claude（可自定义）
种类：卡通玄凤鹦鹉
风格：像素风，32x32 或 64x64 基础尺寸
配色：
  ├─ 身体：奶油白 #FFF8E7
  ├─ 头冠：亮黄 #FFD700
  ├─ 腮红：粉红 #FF9999
  ├─ 翅膀：浅灰 #E0E0E0
  └─ 眼睛：黑色 + 白色高光
```

### 4.2 状态动画设计

| 状态 | 动作描述 | 帧数 | 帧率 |
|------|----------|------|------|
| **Sleeping** | 站立闭眼，轻微呼吸起伏，头顶冒 ZZZ | 6帧 | 1fps |
| **Idle** | 左右转头张望，偶尔眨眼 | 8帧 | 4fps |
| **Working** | 低头快速啄键盘，身上冒汗珠 | 8帧 | 8fps |
| **Thinking** | 仰头看天，头顶冒问号/灯泡泡泡 | 6帧 | 3fps |
| **PendingApproval** | 犹豫姿态，睁大眼睛，冒冷汗，头顶跳动感叹号 | 6帧 | 2fps |
| **Notify** | 跳跃 + 翅膀扇动，头顶感叹号，音符和闪光 | 6帧 | 6fps |
| **SubAgentWorking** | 主鸟与迷你同伴鸟协作，之间有连接线 | 8帧 | 6fps |
| **Error** | 受惊姿态，头顶冒叉号，炸毛 | 4帧 | 2fps |
| **Stopped** | 收起翅膀站定，缓缓闭眼 | 4帧 | 2fps |

### 4.3 动画帧序列示例（Sleeping）

```
帧1: 身体完全放松，眼睛闭合
帧2: 身体微微上浮（吸气）
帧3: 身体回到原位（呼气），ZZZ 第1帧
帧4: 身体微微上浮，ZZZ 第2帧（上升）
帧5: 身体回到原位，ZZZ 第3帧（继续上升并消失）
帧6: 同帧1（循环）
```

### 4.4 AI 生成提示词

**基础形象：**
```
A cute cartoon cockatiel bird mascot, pixel art style, 64x64 pixels,
cream white body, bright yellow crest, pink blush on cheeks,
big expressive black eyes with white highlights,
small orange beak, simple wings, retro game sprite,
transparent background, clean pixel edges, chibi proportions
```

**各状态追加词：**

| 状态 | 追加提示词 |
|------|-----------|
| Sleeping | `sleeping peacefully with eyes closed, tiny ZZZ floating above head in soft blue, gentle breathing pose with body slightly bobbing, relaxed crest feathers laying flat` |
| Idle | `looking left and right curiously with head tilting, crest raised alertly, friendly expression with occasional blink, standing upright on perch, gentle swaying` |
| Working | `pecking at a tiny keyboard intensely, sweat drops on forehead, focused determined face with narrowed eyes, rapid typing motion, small sparkles around beak` |
| Thinking | `looking up at sky with thought bubble containing question marks and lightbulb, one foot slightly raised, contemplative pose with tilted head, crest feathers curling thoughtfully` |
| PendingApproval | `frozen in hesitant pose with wide worried eyes and raised eyebrows, bouncing exclamation mark above head, single sweat drop on cheek, wings slightly tucked nervously, body leaning away slightly as if unsure whether to proceed` |
| Notify | `jumping excitedly with wings spread wide, exclamation mark above head glowing bright, happy squawk with open beak, musical notes and sparkles floating around, crest feathers fully extended upward` |
| SubAgentWorking | `two cockatiels working together side by side, main bird pointing with wing at tiny screen while smaller companion bird nods, a glowing connection line between them like a data link, sparkles and small gear icons around both, the companion is a miniature version with slightly different crest color, collaborative energetic pose` |
| Error | `startled pose with feathers all puffed out, X marks above head in red, worried shaking expression with eyes wide open, small lightning bolt sparks around body, crest feathers standing straight up in shock` |
| Stopped | `standing still with wings folded close to body, eyes slowly closing in calm acceptance, crest feathers gently drooping downward, body slightly smaller and settled, peaceful wind-down pose with a small sigh effect, fading energy particles` |

---

## 5. 项目结构

```
claude-pet/
├── Cargo.toml
├── src/
│   ├── main.rs                  # 程序入口
│   ├── app.rs                   # 应用主循环协调
│   ├── config.rs                # 配置管理
│   │
│   ├── state/                   # 状态管理
│   │   ├── mod.rs
│   │   ├── pet_state.rs         # PetState 定义
│   │   └── state_machine.rs     # 状态转换逻辑
│   │
│   ├── monitor/                 # 状态接收
│   │   ├── mod.rs
│   │   ├── http_server.rs       # HTTP 服务（tiny_http）
│   │   └── process_detector.rs  # Claude 进程检测
│   │
│   ├── animation/               # 动画系统
│   │   ├── mod.rs
│   │   ├── sprite_sheet.rs      # 精灵图加载
│   │   ├── frame_manager.rs     # 帧管理
│   │   └── animation_map.rs     # 状态→动画映射
│   │
│   ├── render/                  # 渲染系统
│   │   ├── mod.rs
│   │   ├── window.rs            # 透明窗口创建
│   │   ├── renderer.rs          # pixels 渲染
│   │   └── drag.rs              # 拖动 + 边缘吸附
│   │
│   └── system/                  # 系统集成
│       ├── mod.rs
│       ├── tray.rs              # 系统托盘
│       └── autostart.rs         # 开机启动
│
├── assets/
│   ├── sprites/                 # 精灵图
│   │   ├── sleeping.png         # Sprite sheet
│   │   ├── waiting.png          # (对应 Idle 状态)
│   │   ├── working.png
│   │   ├── thinking.png
│   │   ├── pending_approval.png # (对应 PendingApproval 状态)
│   │   ├── notify.png
│   │   ├── subagent.png         # (对应 SubAgentWorking 状态)
│   │   ├── error.png
│   │   └── stopped.png
│   ├── icon/
│   │   ├── icon.ico             # Windows 图标
│   │   ├── icon.icns            # macOS 图标
│   │   └── icon.png             # Linux 图标
│   └── tray/
│       ├── tray.ico
│       └── tray.png
│
├── hooks/                       # Claude Code 配置
│   ├── set-status.sh            # Linux/macOS 状态脚本
│   ├── set-status.ps1           # Windows 状态脚本
│   └── settings-template.json   # Hooks 配置模板
│
├── scripts/
│   └── generate_sprites.py      # 精灵图生成脚本
│
└── README.md
```

---

## 6. 核心模块实现

### 6.1 状态文件结构

```rust
// src/state/pet_state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PetState {
    Sleeping,
    Idle,
    Working,
    Thinking,
    PendingApproval,
    Notify(String),
    SubAgentWorking,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusFile {
    pub state: PetState,
    pub timestamp: u64,
    pub session_id: String,
    pub message: Option<String>,
}

impl StatusFile {
    pub fn is_expired(&self, timeout_ms: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now - self.timestamp > timeout_ms
    }
}
```

### 6.2 状态机实现

```rust
// src/state/state_machine.rs
use super::pet_state::PetState;

pub struct StateMachine {
    current: PetState,
    previous: Option<PetState>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current: PetState::Sleeping,
            previous: None,
        }
    }

    pub fn transition(&mut self, new_state: PetState) -> bool {
        if self.current == new_state {
            return false; // 无变化
        }
        
        // 状态优先级：高优先级状态不会被低优先级覆盖
        let priority = |s: &PetState| match s {
            PetState::Error => 6,
            PetState::Notify(_) => 5,
            PetState::PendingApproval => 4,
            PetState::SubAgentWorking => 3,
            PetState::Working => 2,
            PetState::Thinking => 1,
            PetState::Idle => 0,
            PetState::Sleeping => 0,
        };
        
        if priority(&new_state) >= priority(&self.current) {
            self.previous = Some(self.current.clone());
            self.current = new_state;
            true
        } else {
            false
        }
    }

    pub fn current(&self) -> &PetState {
        &self.current
    }
}
```

### 6.3 HTTP 状态接收实现

```rust
// src/monitor/http_server.rs
use crate::state::StatusFile;
use std::io::Read;
use std::sync::mpsc;

pub struct HttpServer {
    _server: tiny_http::Server,
}

impl HttpServer {
    pub fn start(port: u16, tx: mpsc::Sender<StatusFile>) -> anyhow::Result<Self> {
        let addr = ([127, 0, 0, 1], port).into();
        let server = tiny_http::Server::http(addr)?;
        let server_ref = server.clone();

        std::thread::spawn(move || {
            for mut request in server_ref.incoming_requests() {
                if request.method() != &tiny_http::Method::Post
                    || request.url() != "/status"
                {
                    let _ = request.respond(
                        tiny_http::Response::from_string("Not Found").with_status_code(404)
                    );
                    continue;
                }

                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).ok();

                if let Ok(status) = StatusFile::from_json(&body) {
                    let _ = tx.send(status);
                    let _ = request.respond(tiny_http::Response::empty(204));
                }
            }
        });

        Ok(Self { _server: server })
    }
}
```

### 6.4 配置管理

```rust
// src/config.rs
pub const DEFAULT_HTTP_PORT: u16 = 9527;

pub fn get_http_port() -> u16 {
    std::env::var("CLAUDE_POKE_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_HTTP_PORT)
}
```

---

## 7. 多平台构建配置

### 7.1 Cargo.toml

```toml
[package]
name = "claude-pet"
version = "0.1.0"
edition = "2021"

[dependencies]
winit = "0.30"
pixels = "0.14"
image = { version = "0.25", features = ["png", "gif"] }
tiny_http = "0.12"
tray-icon = "0.19"
sysinfo = "0.31"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "5"
anyhow = "1"
log = "0.4"
env_logger = "0.11"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winuser", "wingdi"] }

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.26"
objc = "0.2"

[profile.release]
opt-level = 3
lto = true
```

### 7.2 多平台构建脚本

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - uses: actions/upload-artifact@v4
        with:
          name: claude-pet-${{ matrix.os }}
          path: target/release/claude-pet*
```

---

## 8. 实现阶段

### Phase 1：状态接收（核心）
- [x] 项目结构搭建
- [x] PetState 定义
- [x] 状态 JSON 格式
- [x] HTTP 状态接收模块（tiny_http）
- [ ] Claude 进程检测

### Phase 2：渲染框架
- [ ] 透明窗口创建
- [ ] pixels 渲染循环
- [ ] 窗口拖动
- [ ] 边缘吸附

### Phase 3：动画系统
- [ ] 精灵图加载
- [ ] 帧管理器
- [ ] 状态→动画映射

### Phase 4：系统集成
- [ ] 系统托盘
- [ ] 开机启动选项
- [ ] 配置文件

### Phase 5：打磨
- [ ] 玄凤鹦鹉精灵图制作
- [ ] Hooks 脚本完善
- [ ] 多平台测试
- [ ] 打包发布

---

## 9. 验证方式

### 9.1 单元测试

```bash
cargo test
```

### 9.2 手动测试状态切换

```bash
# Linux/macOS — 使用 curl 发送状态
curl -X POST http://127.0.0.1:9527/status \
  -H "Content-Type: application/json" \
  -d '{"state":"Working","timestamp":1717411200000,"session_id":"test","message":null}'

# Windows PowerShell
Invoke-RestMethod -Uri "http://127.0.0.1:9527/status" -Method Post \
  -Body '{"state":"Working","timestamp":1717411200000,"session_id":"test","message":null}' \
  -ContentType "application/json"

# 或直接运行 hook 脚本
./hooks/set-status.sh Working
```

### 9.3 集成测试

1. 启动 claude-pet
2. 运行 Claude Code 并执行操作
3. 观察宠物状态是否实时变化

---

## 10. 后续扩展

- **ESP32 移植**：HTTP daemon 模式可桥接 BLE（btleplug）推送到 ESP 屏幕
- **多宠物支持**：不同项目显示不同宠物
- **自定义皮肤**：用户可替换精灵图
- **声音效果**：玄凤鹦鹉叫声
