# CloverViewer 架构决策记录

本文档记录项目中的关键架构决策及其原因，帮助理解设计选择并为未来变更提供上下文。

---

## ADR-001: 使用 egui/eframe 作为 GUI 框架

**状态**: 已接受

**上下文**: 需要一个轻量、高性能的 Rust GUI 框架，支持自定义绘制（截图标注需要）。

**考虑的选项**:
- **egui/eframe**: Rust 原生，即时模式，轻量，易于嵌入
- **Tauri**: Web 技术栈，但截图/OCR 集成复杂
- **Iced**: Elm 架构，但生态较新，自定义绘制支持有限
- **tauri + Rust 后端**: 过于复杂，启动慢

**决策**: 使用 egui 0.33 + eframe 0.33

**原因**:
1. 纯 Rust 实现，无需 WebView，二进制体积小
2. 即时模式架构适合截图工具这种需要频繁重绘的场景
3. 原生支持自定义绘制（Painter API），实现标注工具简单
4. 与 Windows API 集成直接（COM、OCR）
5. 编译速度快，开发体验好

**权衡**:
- ✅ 高性能、小体积
- ✅ 完全控制渲染
- ❌ 控件库不如 Web 丰富（需要自己实现部分组件）
- ❌ 非原生外观（但截图工具不在意这个）

---

## ADR-002: Feature 模块化架构

**状态**: 已接受

**上下文**: 应用有两个主要功能（图片查看、截图），需要清晰分离且能共享基础设施。

**决策**: 使用 Feature trait + 状态机模式

**结构**:
```
App
├── ViewerFeature (AppMode::Viewer)
├── ScreenshotFeature (AppMode::Screenshot)
└── CommonState (共享状态)
```

**原因**:
1. 每个功能独立维护，修改不互相影响
2. 通过 AppMode 显式切换，状态管理清晰
3. 共享 CommonState 处理跨功能需求（OCR、Toast）
4. 易于测试（可以单独测试每个 Feature）

**实现要点**:
- `Feature` trait 统一定义接口
- `app.rs` 作为协调器，不处理具体业务逻辑
- 热键通过 `handle_hotkey` 分发到对应 Feature

---

## ADR-003: 使用 Windows 原生 OCR

**状态**: 已接受

**上下文**: 需要 OCR 功能提取截图中的文字。

**考虑的选项**:
- **Windows.Media.Ocr**: Windows 10+ 内置，无需额外依赖
- **Tesseract**: 开源，但需要额外库和训练数据
- **在线 API**: 需要网络，有隐私问题

**决策**: 使用 Windows.Media.Ocr COM API

**原因**:
1. 零额外依赖，Windows 用户开箱即用
2. 支持中文、英文、日文（项目所需语言）
3. 离线工作，保护隐私
4. 性能足够（在后台线程运行）

**实现细节**:
- 封装在 `feature/screenshot/ocr/` 模块
- 使用 `windows` crate 调用 COM API
- 异步处理避免阻塞 UI
- 失败时显示友好错误提示

**限制**:
- 仅 Windows 10/11 可用
- 依赖 Windows 语言包
- 识别质量中等（不如 Tesseract 训练后）

---

## ADR-004: 单实例应用模式

**状态**: 已接受

**上下文**: 图片查看器通常是单实例应用，用户不希望打开多个窗口。

**决策**: 使用 `single-instance` crate 限制单实例

**实现**:
```rust
let instance = SingleInstance::new("CloverViewer")
    .expect("Failed to create single instance guard");
if !instance.is_single() {
    return Ok(()); // 已有实例运行，直接退出
}
```

**原因**:
1. 简单可靠，使用命名互斥量实现
2. 符合用户习惯（类似 Windows 照片应用）
3. 避免资源浪费

**未来考虑**: 可以扩展为"新实例向旧实例发送消息打开新图片"，但目前非必需。

---

## ADR-005: 异步图片加载 + LRU 缓存

**状态**: 已接受

**上下文**: 图片查看器需要加载大图，不能阻塞 UI 线程。

**决策**: 使用 mpsc 通道 + 后台线程加载，LRU 缓存限制内存

**架构**:
```
UI Thread                    Background Thread
     |                              |
     |--- send(path) -------------->|
     |                              |  load image
     |<-- send(TextureHandle) ------|
     |                              |
```

**原因**:
1. 图片加载不卡顿 UI
2. LRU 缓存避免重复加载和内存无限增长
3. 使用 `rayon` 并行解码多张图片（网格视图）

**缓存策略**:
- 默认缓存 30 张最近查看的图片
- 缓存的是 GPU Texture（不是原始数据）
- 网格视图使用低分辨率缩略图

---

## ADR-006: 全局热键使用 global-hotkey crate

**状态**: 已接受

**上下文**: 截图工具需要全局快捷键（即使应用不在前台）。

**决策**: 使用 `global-hotkey` crate + 自定义配置

**架构**:
- `core/hotkeys.rs` 管理热键注册/注销
- 支持自定义快捷键（从 Config 解析）
- 截图时动态注册/注销复制热键

**原因**:
1. `global-hotkey` 跨平台（虽然项目仅 Windows）
2. 支持解析字符串热键（如 "Alt+S"）
3. 与 egui 集成简单（通过 mpsc 通道）

**注意事项**:
- 热键注册可能失败（已被其他应用占用），需要错误处理
- 从托盘唤起需要特殊处理（Win32 API）

---

## ADR-007: 三语言硬编码 i18n

**状态**: 已接受

**上下文**: 需要支持中文、英文、日文三种界面语言。

**决策**: 硬编码在 `lang.rs` 中，不使用外部文件

**实现**:
```rust
pub struct TextBundle {
    pub menu_file: &'static str,
    // ...
}

pub const ZH_TEXT: TextBundle = TextBundle { /* ... */ };
pub const EN_TEXT: TextBundle = TextBundle { /* ... */ };
pub const JA_TEXT: TextBundle = TextBundle { /* ... */ };
```

**原因**:
1. 简单，无需文件 IO 和错误处理
2. 编译时检查，不会遗漏翻译
3. 二进制独立，无外部资源依赖
4. 三种语言量可控（约 100 条文本）

**权衡**:
- ✅ 简单可靠
- ✅ 运行时无文件读取开销
- ❌ 添加新文本需要改代码
- ❌ 非开发者无法翻译

**适用场景**: 固定少量语言的桌面应用。如果是更多语言或需要社区翻译，应考虑 gettext 或 fluent。

---

## ADR-008: 截图使用独立 Viewport

**状态**: 已接受

**上下文**: 截图需要全屏覆盖所有显示器，且要有透明效果。

**决策**: 使用 `ctx.show_viewport_immediate` 创建独立窗口

**实现**:
```rust
ctx.show_viewport_immediate(
    ViewportId::from("screenshot"),
    ViewportBuilder::default()
        .with_transparent(true)
        .with_decorations(false)
        .with_fullsize(true),
    |ctx| {
        // 截图绘制逻辑
    },
);
```

**原因**:
1. 可以覆盖所有显示器（多屏支持）
2. 透明背景，可以看到桌面内容
3. 无边框，全沉浸式体验
4. 退出截图时销毁窗口，不污染主窗口状态

---

## ADR-009: 配置存储在可执行文件旁

**状态**: 已接受

**上下文**: 需要保存用户设置（语言、热键、窗口大小等）。

**决策**: 使用 JSON 文件存储在可执行文件同目录

**实现**:
```rust
fn get_config_path() -> PathBuf {
    let mut path = env::current_exe().unwrap_or_default();
    path.set_file_name("config.json");
    path
}
```

**原因**:
1. 简单，无需处理 Windows 注册表或 AppData 路径
2. 便携，配置文件随程序移动
3. 易于备份和重置（直接删文件）

**权衡**:
- ✅ 实现简单
- ✅ 便携
- ❌ 多用户共享配置（对于单用户工具不是问题）
- ❌ 需要写入权限（但程序目录通常可写）

---

## ADR-010: 图像解码使用 image crate + zune-jpeg

**状态**: 已接受

**上下文**: 需要支持多种图片格式，且 JPEG 是最常见的格式。

**决策**: 使用 `image` crate 作为主解码器，`zune-jpeg` 加速 JPEG

**Cargo.toml**:
```toml
image = { version = "0.25", features = ["png", "jpeg", "gif", "bmp", "webp", "tiff"], default-features = false }
zune-jpeg = "0.5"
```

**原因**:
1. `image` crate 是 Rust 生态最成熟的图像库
2. `zune-jpeg` 是快速的 JPEG 解码器（替代 image 自带的）
3. 禁用默认特性减少编译时间和二进制大小

**AVIF 考虑**: 原本计划支持 AVIF，但 `dav1d` 依赖复杂，暂时禁用。

---

## 总结

核心架构原则：

1. **简单优先**: 选择最简单的方案，除非有明确需求需要复杂性
2. **Windows 原生**: 充分利用 Windows API，不担心跨平台
3. **性能敏感**: 图片加载异步，UI 不卡顿
4. **用户体验**: 全局热键、系统托盘、流畅动画
5. **独立运行**: 单文件、无依赖、无安装

---

*架构决策应在变更时更新*
