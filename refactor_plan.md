# CloverViewer 重构计划

## 1. 重构目标

| 目标 | 描述 |
|------|------|
| **功能模块化** | 清晰划分"图片查看"和"截图"两大核心功能 |
| **状态解耦** | 消除 Viewer 和 Screenshot 之间的状态依赖 |
| **状态集中管理** | 统一状态管理，降低复杂度 |
| **UI 层分离** | 将 Feature UI 和通用 UI 组件分离 |
| **可扩展性** | 新功能可通过实现 Trait 轻松添加 |

## 2. 当前问题分析

### 2.1 UiMode 职责混乱

**位置**: `src/ui/mode.rs`

```rust
#[derive(Clone, PartialEq)]
pub enum UiMode {
    Normal,           // 普通模式
    About,            // 关于窗口 (UI overlay)
    Settings(Config), // 设置窗口 (UI overlay)
    ContextMenu(Pos2), // 右键菜单 (UI overlay)
    Properties,       // 属性面板 (UI overlay)
    Screenshot,       // 截图模式 (Feature mode) ← 语义不同！
}
```

**问题**:
- `UiMode` 同时表示"功能模式"和"UI 覆盖层状态"，概念混淆
- `Screenshot` 与 `About/Settings/Properties` 不是同一层级的概念
- `Settings(Config)` 变体携带状态，不符合最佳实践

### 2.2 状态耦合严重

**位置**: `src/app.rs`

```rust
pub struct AppState {
    pub ui_mode: UiMode,
    pub viewer: ViewerState,        // 查看器状态
    pub screenshot: ScreenshotState, // 截图状态
    pub common: CommonState,
}
```

**问题**:
- `viewer` 和 `screenshot` 并列存在，但同一时间只有一个生效
- `process_hotkey_events()` 直接修改 `ui_mode` 和 `screenshot`
- 热键逻辑与状态修改紧耦合

### 2.3 热键管理耦合

**位置**: `src/core/hotkeys.rs:125`

```rust
pub fn update(&mut self, ui_mode: &UiMode) -> Vec<HotkeyAction> {
    // 根据 ui_mode 动态注册/注销 Ctrl+C 快捷键
    if *ui_mode == UiMode::Screenshot && !self.is_copy_registered {
        // 注册复制快捷键
    }
}
```

**问题**:
- `HotkeyManager` 需要知道当前 UI 模式才能决定行为
- 这导致热键系统与 UI 模式强耦合

### 2.4 目录结构混乱

```
src/ui/
├── mode.rs              ← 业务状态，不属于 UI 层
├── viewer.rs            ← 功能模块
├── screenshot/          ← 功能模块
│   └── ...
├── view/                ← 应该属于 viewer feature
│   └── ...
└── panels/              ├── about.rs      ← 通用面板
    ├── settings.rs      ← 通用面板
    └── properties_panel.rs  ← viewer 专用
```

## 3. 重构方案

### 3.1 新的目录结构

```
src/
├── main.rs
├── app.rs
├── core/
│   ├── mod.rs
│   ├── image_loader.rs
│   ├── business.rs
│   └── hotkeys.rs
├── model/
│   ├── mod.rs
│   ├── state.rs         # AppState, CommonState
│   ├── mode.rs          # AppMode, OverlayMode (新)
│   ├── config.rs
│   ├── image_meta.rs
│   ├── device.rs
│   └── window_state.rs
├── feature/             # 功能模块 (新)
│   ├── mod.rs
│   ├── viewer/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   ├── view/
│   │   │   ├── mod.rs
│   │   │   ├── single_view.rs
│   │   │   ├── grid_view.rs
│   │   │   ├── preview.rs
│   │   │   └── arrows.rs
│   │   └── panels/      # viewer 专用面板
│   │       └── properties_panel.rs
│   └── screenshot/
│       ├── mod.rs
│       ├── state.rs
│       ├── capture.rs
│       ├── draw.rs
│       ├── toolbar.rs
│       ├── magnifier.rs
│       └── color_picker.rs
├── ui/
│   ├── mod.rs
│   ├── menus/
│   ├── widgets/
│   └── resources.rs
├── os/
├── utils/
└── i18n/
```

### 3.2 核心抽象

#### 3.2.1 AppMode vs OverlayMode

```rust
// src/model/mode.rs

/// 应用级功能模式 - 顶层状态机
#[derive(Clone, PartialEq, Debug)]
pub enum AppMode {
    Viewer,     // 图片查看器
    Screenshot, // 截图工具
}

/// UI 覆盖层状态 - 仅在 Viewer 模式下使用
#[derive(Clone, PartialEq)]
pub enum OverlayMode {
    None,
    About,
    Settings { config: Config },  // 使用 Config 副本
    ContextMenu(Pos2),
    Properties,
}
```

**设计理由**:
- `AppMode` 表示顶层功能模式，互斥
- `OverlayMode` 仅在 `Viewer` 模式下作为子状态

#### 3.2.2 Feature Trait

```rust
// src/feature/mod.rs

mod viewer;
mod screenshot;

pub use viewer::ViewerFeature;
pub use screenshot::ScreenshotFeature;

use eframe::egui::Context;
use crate::core::hotkeys::HotkeyAction;
use crate::model::mode::AppMode;
use crate::model::state::CommonState;

pub trait Feature {
    /// 更新 Feature 状态
    fn update(&mut self, ctx: &Context, common: &mut CommonState);

    /// 返回当前功能模式
    fn mode(&self) -> AppMode;

    /// 处理热键事件
    /// 返回 Some(AppMode) 表示需要切换到该模式
    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode>;
}
```

### 3.3 状态重构

#### 重构前

```rust
// src/model/state.rs

pub struct AppState {
    pub ui_mode: UiMode,
    pub viewer: ViewerState,
    pub screenshot: ScreenshotState,
    pub common: CommonState,
}
```

#### 重构后

```rust
// src/model/state.rs

pub struct AppState {
    pub mode: AppMode,         // 功能模式
    pub common: CommonState,   // 共享状态
}

pub struct CommonState {
    pub path_sender: Sender<PathBuf>,
    pub path_receiver: Receiver<PathBuf>,
    pub toast_system: ToastSystem,
    pub toast_manager: ToastManager,
    pub hotkey_manager: HotkeyManager,
    pub window_state: WindowState,
    pub device_info: DeviceInfo,
}
```

### 3.4 热键重构

#### 重构前

```rust
// src/core/hotkeys.rs

pub fn update(&mut self, ui_mode: &UiMode) -> Vec<HotkeyAction> {
    // 依赖 UiMode 判断行为
}
```

#### 重构后

```rust
// src/core/hotkeys.rs

pub enum HotkeyAction {
    SwitchToScreenshot { prev_state: WindowPrevState },
    RequestScreenshotCopy,
}

impl HotkeyManager {
    /// 返回热键事件列表，不再需要传入 UiMode
    pub fn update(&mut self) -> Vec<HotkeyAction> {
        // 内部处理所有状态判断
    }
}
```

**关键变更**:
- `update()` 不再需要 `&UiMode` 参数
- 热键管理器内部维护必要状态（如 `is_copy_registered`）
- 通过 `prev_state` 携带窗口状态信息

### 3.5 代码示例

#### ViewerFeature

```rust
// src/feature/viewer/mod.rs

use super::{AppMode, Feature, HotkeyAction};
use crate::core::business::ViewerState;
use crate::model::mode::OverlayMode;
use crate::model::state::CommonState;
use eframe::egui::Context;

pub struct ViewerFeature {
    state: ViewerState,
    overlay: OverlayMode,
}

impl ViewerFeature {
    pub fn new() -> Self {
        Self {
            state: ViewerState::new(),
            overlay: OverlayMode::None,
        }
    }
}

impl Feature for ViewerFeature {
    fn update(&mut self, ctx: &Context, common: &mut CommonState) {
        // 处理图片加载结果
        if self.state.process_load_results(ctx) {
            ctx.request_repaint();
        }

        // 处理新路径
        if let Ok(path) = common.path_receiver.try_recv() {
            self.state.open_new_context(ctx.clone(), path);
        }

        // 绘制 UI
        self.draw(ctx, common);
    }

    fn mode(&self) -> AppMode {
        AppMode::Viewer
    }

    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode> {
        match action {
            HotkeyAction::SwitchToScreenshot { prev_state } => {
                Some(AppMode::Screenshot)
            }
            _ => None,
        }
    }
}

impl ViewerFeature {
    fn draw(&self, ctx: &Context, common: &CommonState) {
        // 绘制顶部/底部面板
        // 绘制中央视图 (single/grid)
        // 处理 overlay (about/settings/context_menu/properties)
    }
}
```

#### ScreenshotFeature

```rust
// src/feature/screenshot/mod.rs

use super::{AppMode, Feature, HotkeyAction};
use crate::model::state::CommonState;
use crate::ui::screenshot::state::{ScreenshotState, WindowPrevState};
use eframe::egui::Context;

pub struct ScreenshotFeature {
    state: ScreenshotState,
}

impl ScreenshotFeature {
    pub fn new(prev_state: WindowPrevState) -> Self {
        Self {
            state: ScreenshotState::new(prev_state),
        }
    }
}

impl Feature for ScreenshotFeature {
    fn update(&mut self, ctx: &Context, common: &mut CommonState) {
        handle_screenshot_system(ctx, &mut self.state, common);
    }

    fn mode(&self) -> AppMode {
        AppMode::Screenshot
    }

    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode> {
        match action {
            HotkeyAction::RequestScreenshotCopy => {
                self.state.copy_requested = true;
                None
            }
            _ => None,
        }
    }
}
```

#### CloverApp

```rust
// src/app.rs

pub struct CloverApp {
    state: AppState,
    config: Arc<Config>,
    _tray: TrayIcon,
    current_feature: Box<dyn Feature>,
}

impl CloverApp {
    pub fn new(cc: &eframe::CreationContext<'_>, start_path: Option<PathBuf>, config: Config) -> Self {
        let state = AppState::new(&cc.egui_ctx, visible, allow_quit, hwnd_isize);
        let mut viewer_feature = ViewerFeature::new();

        if let Some(path) = start_path {
            viewer_feature.state.open_new_context(cc.egui_ctx.clone(), path);
        }

        Self {
            state,
            config: config_arc,
            _tray: tray,
            current_feature: Box::new(viewer_feature),
        }
    }
}

impl eframe::App for CloverApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        self.handle_cache_win_pos(ctx, frame);
        update_context_config(ctx, &self.config);

        // 处理热键事件
        let actions = self.state.common.hotkey_manager.update();
        for action in actions {
            if let Some(new_mode) = self.current_feature.handle_hotkey(action) {
                self.switch_feature(new_mode);
            }
        }

        // 更新当前 Feature
        self.current_feature.update(ctx, &mut self.state.common);
    }
}

impl CloverApp {
    fn switch_feature(&mut self, mode: AppMode) {
        match mode {
            AppMode::Viewer => {
                self.current_feature = Box::new(ViewerFeature::new());
            }
            AppMode::Screenshot => {
                let prev_state = self.state.common.window_state.take_prev_state();
                self.current_feature = Box::new(ScreenshotFeature::new(prev_state));
            }
        }
    }
}
```

## 4. 重构步骤

### Phase 1: 目录重组

| 步骤 | 操作 | 影响文件 |
|------|------|---------|
| 1.1 | 创建 `src/feature/` 目录 | 新建 |
| 1.2 | 移动 `src/ui/screenshot/` → `src/feature/screenshot/` | `src/ui/mod.rs` |
| 1.3 | 移动 `src/ui/viewer.rs` → `src/feature/viewer/mod.rs` | `src/ui/mod.rs` |
| 1.4 | 移动 `src/ui/view/` → `src/feature/viewer/view/` | `src/feature/viewer/mod.rs` |
| 1.5 | 移动 `src/ui/panels/properties_panel.rs` → `src/feature/viewer/panels/` | `src/ui/panels/mod.rs` |
| 1.6 | 创建 `src/model/mode.rs`，移动 `UiMode` 相关代码 | 多个文件 |

### Phase 2: Feature Trait

| 步骤 | 操作 | 文件 |
|------|------|------|
| 2.1 | 定义 `AppMode` 和 `OverlayMode` | `src/model/mode.rs` |
| 2.2 | 定义 `Feature` trait | `src/feature/mod.rs` |
| 2.3 | 实现 `ViewerFeature` | `src/feature/viewer/mod.rs` |
| 2.4 | 实现 `ScreenshotFeature` | `src/feature/screenshot/mod.rs` |

### Phase 3: 状态解耦

| 步骤 | 操作 | 文件 |
|------|------|------|
| 3.1 | 简化 `AppState`，移除 `viewer`, `screenshot`, `ui_mode` | `src/model/state.rs` |
| 3.2 | 将 `OverlayMode` 移到 `ViewerFeature` 内部 | `src/feature/viewer/mod.rs` |
| 3.3 | 改造 `HotkeyManager.update()` 不再依赖 `UiMode` | `src/core/hotkeys.rs` |

### Phase 4: App 整合

| 步骤 | 操作 | 文件 |
|------|------|------|
| 4.1 | 在 `CloverApp` 中使用 `Box<dyn Feature>` | `src/app.rs` |
| 4.2 | 实现模式切换逻辑 | `src/app.rs` |
| 4.3 | 移除旧的 `ui_mode` 处理 | `src/app.rs` |

## 5. 关键设计决策

### 5.1 WindowPrevState 传递

截图功能需要知道进入截图前的窗口状态（Normal/Minimized/Tray）。

**方案**: 在切换到 Screenshot 模式时，从 `WindowState` 获取并传递给 `ScreenshotFeature`。

```rust
// WindowState 新增方法
impl WindowState {
    pub fn take_prev_state(&mut self) -> WindowPrevState {
        // 获取并重置 prev_state
    }
}
```

### 5.2 ScreenshotFeature 的创建

**方案**: `ScreenshotFeature::new(prev_state)` 接收之前的窗口状态。

这确保了退出截图模式时能正确恢复窗口状态。

### 5.3 双重 dispatch 处理热键

```
HotkeyManager::update()
    ↓ Vec<HotkeyAction>
CloverApp::update()
    ↓ current_feature.handle_hotkey()
    ↓ Option<AppMode>
CloverApp::switch_feature()
```

这种设计：
- `HotkeyManager` 不需要知道当前是哪个 Feature
- Feature 自己决定如何响应热键
- App 负责 Feature 之间的切换

## 6. 预期收益

| 收益 | 描述 |
|------|------|
| **高内聚** | 每个 Feature 封装自己的状态和逻辑 |
| **低耦合** | Feature 之间通过 trait 接口交互 |
| **易测试** | Feature 可独立单元测试 |
| **易扩展** | 新功能只需实现 `Feature` trait |
| **结构清晰** | 代码按功能组织，职责分明 |
| **状态集中** | `AppState` 只管理全局共享状态 |

## 7. 风险与注意事项

| 风险 | 缓解措施 |
|------|---------|
| 热键行为改变 | 保持现有热键功能不变，逐步迁移 |
| 状态丢失 | 模式切换时保存/恢复必要状态 |
| 编译时间长 | 分阶段提交，每阶段确保可编译 |
| 回归风险 | 每阶段完成后进行功能测试 |

## 8. 实施顺序

```
Phase 1: 目录重组
  ↓
Phase 2: Feature Trait 定义
  ↓
Phase 3: 状态解耦
  ↓
Phase 4: App 整合
```

每个Phas做完之后等待我的确认，再进行下一步
