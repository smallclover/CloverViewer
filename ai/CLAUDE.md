# CloverViewer - Claude Code 开发指南

## 🎯 项目概览

CloverViewer 是一个基于 Rust + egui/eframe 开发的轻量级图片查看与截图工具，仅支持 Windows 平台。

- **技术栈**: Rust, egui 0.33, eframe 0.33, Windows API
- **架构模式**: 模块化 Feature 架构 + 状态管理
- **特殊依赖**: Windows OCR API, xcap (截图), global-hotkey (全局热键)

## 🏗️ 项目结构

```
src/
├── main.rs              # 入口，初始化日志、单实例检查
├── app.rs               # 应用主循环，模式切换
├── core/                # 核心基础设施
│   ├── business.rs      # 图片加载、ViewerState 业务逻辑
│   ├── config_manager.rs# 配置管理（保存/加载）
│   ├── hotkeys.rs       # 全局热键管理
│   ├── image_loader.rs  # 异步图片加载
│   └── logging.rs       # 日志系统初始化
├── feature/             # 功能模块（Feature 架构）
│   ├── screenshot/      # 截图功能
│   │   ├── canvas/      # 画布绘制系统
│   │   ├── capture/     # 屏幕捕获逻辑
│   │   ├── ocr/         # OCR 文字识别
│   │   ├── toolbar.rs   # 工具栏
│   │   └── ...
│   └── viewer/          # 图片查看器功能
│       ├── grid_view.rs # 网格视图
│       ├── single_view.rs # 单图视图
│       └── ...
├── i18n/                # 国际化
│   └── lang.rs          # 三语言文本定义
├── model/               # 数据模型
│   ├── config.rs        # 配置数据结构
│   ├── image_meta.rs    # 图片元数据
│   ├── mode.rs          # 应用模式枚举
│   └── state.rs         # 公共状态
├── ui/                  # UI 组件
│   ├── widgets/         # 可复用组件
│   └── resources.rs     # 图标、字体资源
├── os/                  # 操作系统相关
│   └── window.rs        # Win32 API 封装
└── utils/               # 工具函数
```

## 🔑 关键概念

### Feature 模式

每个主要功能实现 `Feature` trait：

```rust
pub trait Feature {
    /// 每帧更新逻辑
    fn update(&mut self, ctx: &Context, common: &mut CommonState, mode: &mut AppMode);
    
    /// 处理全局热键
    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode>;
}
```

主应用在 `app.rs` 中协调多个 Feature，通过 `AppMode` 切换当前活动功能。

### 状态分层

- **CommonState**: 跨功能共享状态（OCR 状态、Toast 通知、路径通道等）
- **FeatureState**: 各功能私有状态（如 ViewerState、ScreenshotState）
- **Config**: 持久化配置，通过 egui context 临时存储

### 国际化 (i18n)

所有用户可见文本必须通过 `get_i18n_text(ctx)` 获取：

```rust
let text = get_i18n_text(ctx);
ui.label(text.menu_file);
```

添加新文本步骤：
1. 在 `TextBundle` struct 中添加字段（lang.rs）
2. 在 ZH_TEXT/EN_TEXT/JA_TEXT 中提供三语言翻译

## 🛠️ 常见任务

### 添加新的截图标注工具

1. 在 `feature/screenshot/capture/actions.rs` 添加工具枚举变体
2. 在 `toolbar.rs` 添加工具按钮（复制现有模式）
3. 在 `canvas/draw.rs` 实现绘制逻辑
4. 在 `i18n/lang.rs` 添加工具提示文本

### 添加新的设置选项

1. 在 `model/config.rs` 的 `Config` struct 添加字段
2. 在 `ui/widgets/settings.rs` 添加 UI 控件
3. 如需要热键，在 `core/hotkeys.rs` 处理
4. 在 `i18n/lang.rs` 添加设置项标签

### 添加新的图片格式支持

1. 在 `model/image_meta.rs` 的 `SUPPORTED_IMAGE_EXTENSIONS` 添加扩展名
2. 确保 `image` crate 的 features 包含对应格式解码器
3. 测试该格式的 EXIF 读取是否正常

### 修改 UI 布局

- 使用 egui 的 `CentralPanel`、`TopBottomPanel`、`SidePanel` 进行布局
- 自定义组件放在 `ui/widgets/` 目录
- 遵循 egui 的即时模式范式，不要在 draw 中修改状态

## ⚠️ 重要约束

1. **Windows 专属**: 可以自由使用 Win32 API，无需考虑跨平台
2. **单实例应用**: 已集成 `single_instance`，不需要处理多实例
3. **OCR 依赖 Windows**: OCR 功能使用 Windows.Media.Ocr，仅在 Win10+ 可用
4. **全局热键限制**: 热键注册可能失败，必须有降级处理
5. **图片内存**: 使用 LRU 缓存限制内存占用，大图片会自动缩略

## 🧪 调试技巧

- 日志系统已初始化，使用 `tracing::info!()` / `tracing::debug!()`
- egui 自带调试工具：`ctx.set_debug_on_hover(true)`
- 截图调试：设置 `screenshot_hides_main_window = false` 可见主窗口

## 📦 发布构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 打包安装程序
cargo packager --release
```

---

*本文档应与 INDEX.md 中的其他专项文档配合使用*
