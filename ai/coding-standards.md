# CloverViewer 编码规范与避坑指南

## 📐 命名规范

### Rust 标准命名

| 类型 | 规范 | 示例 |
|------|------|------|
| 模块/文件 | snake_case | `grid_view.rs`, `hotkeys.rs` |
| 结构体/枚举 | PascalCase | `ViewerState`, `ScreenshotTool` |
| 函数/方法 | snake_case | `load_image()`, `handle_input()` |
| 变量 | snake_case | `current_index`, `stroke_width` |
| 常量 | SCREAMING_SNAKE_CASE | `DEFAULT_ZOOM`, `MAX_CACHE_SIZE` |
| Trait | PascalCase | `Feature`, `Drawable` |
| 生命周期 | 'a, 'b | `fn foo<'a>(val: &'a str)` |

### 项目特定命名

- **UI 组件文件**: 使用动词/名词形式，如 `draw_menu`, `render_toolbar`
- **Feature 模块**: 以 `_feature` 结尾，如 `ViewerFeature`, `ScreenshotFeature`
- **状态结构体**: 以 `State` 结尾，如 `ScreenshotState`, `OcrState`

## 🎨 代码组织

### 模块导出规范

```rust
// mod.rs 只导出公共接口
pub use self::viewer::ViewerFeature;
pub use self::screenshot::ScreenshotFeature;

// 内部模块保持私有
mod viewer;
mod screenshot;
```

### use 语句排序

```rust
// 1. 标准库
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

// 2. 第三方 crate（按字母序）
use eframe::egui;
use egui::{Context, Color32};

// 3. 本项目模块
use crate::core::config::Config;
use crate::model::mode::AppMode;
```

## ⚠️ 避坑指南

### egui 即时模式陷阱

**❌ 错误 - 在 draw 中直接修改状态：**
```rust
ui.label("Count: {}"), self.count);
if ui.button("+").clicked() {
    self.count += 1; // 这在 egui 中没问题，但要注意...
}
```

**❌ 严重错误 - 基于上一帧状态做条件判断：**
```rust
if self.is_open {
    // 这个条件可能基于过时的状态！
    ui.text_edit_singleline(&mut self.text);
}
```

**✅ 正确 - 使用 ui.memory() 或立即处理：**
```rust
let response = ui.button("Toggle");
if response.clicked() {
    self.is_open = !self.is_open;
}

// 直接绘制，不要基于过时的状态做条件
if self.is_open {
    ui.text_edit_singleline(&mut self.text);
}
```

### egui Context 发送陷阱

**❌ 错误 - 跨线程直接 clone Context：**
```rust
std::thread::spawn(move || {
    // 错误！ctx 不能安全地跨线程发送
    ctx.request_repaint();
});
```

**✅ 正确 - 使用通道或线程安全的方式：**
```rust
// 使用 mpsc 通道传递信号
let (tx, rx) = mpsc::channel();
std::thread::spawn(move || {
    // 工作...
    tx.send(result).ok();
});

// 在主线程处理
if let Ok(result) = rx.try_recv() {
    ctx.request_repaint();
}
```

### 借用检查器陷阱

**❌ 错误 - 同时持有可变和不可变引用：**
```rust
let state = &mut self.state;
let other = &self.config; // 错误！state 和 config 同属于 self
state.update(other);
```

**✅ 正确 - 提前 clone 或分离生命周期：**
```rust
let config = self.config.clone();
self.state.update(&config);

// 或者使用临时作用域
let value = {
    let other = &self.config;
    other.value
};
self.state.update(value);
```

### Windows API 陷阱

**❌ 错误 - 不检查 COM 初始化：**
```rust
// 某些 Windows API 需要 COM 初始化
let ocr = OcrEngine::new()?; // 可能失败！
```

**✅ 正确 - 确保初始化并在失败时优雅降级：**
```rust
pub fn init_ocr() -> Option<OcrEngine> {
    // 确保 COM 已初始化（通常在调用点保证）
    OcrEngine::new().ok()
}

// 使用时检查
if let Some(ref engine) = self.ocr_engine {
    // 使用引擎
} else {
    // 显示"OCR 不可用"提示
}
```

### 图片加载陷阱

**❌ 错误 - 在主线程加载大图：**
```rust
// 阻塞主线程，UI 卡顿
let img = image::open(path)?; // 错误！
```

**✅ 正确 - 使用异步加载：**
```rust
// 项目中已实现 image_loader 模块
// 使用 mpsc 通道，在后台线程加载
let (tx, rx) = mpsc::channel();
std::thread::spawn(move || {
    let result = image::open(path);
    tx.send(result).ok();
});

// 在 update 中检查接收
if let Ok(result) = rx.try_recv() {
    self.handle_loaded_image(result);
}
```

## 🔄 错误处理规范

### 分级处理策略

| 场景 | 处理方式 | 示例 |
|------|----------|------|
| 用户操作失败 | Toast 提示，继续运行 | 图片加载失败 |
| 可恢复错误 | 降级处理，记录日志 | OCR 引擎失败 |
| 致命错误 | panic! 或优雅退出 | 配置目录无法创建 |

### 错误提示必须 i18n

```rust
// ❌ 错误
ui.label("Failed to load image");

// ✅ 正确
let text = get_i18n_text(ctx);
ui.label(text.viewer_error);
```

## 📏 代码风格

### 函数长度

- **理想**: 20-30 行以内
- **警告**: 超过 50 行考虑拆分
- **必须拆分**: 超过 100 行

### match 表达式

```rust
// 简单 match 保持内联
match result {
    Ok(v) => v,
    Err(_) => default,
}

// 复杂逻辑使用命名函数
match result {
    Ok(v) => self.handle_success(v),
    Err(e) => self.handle_error(e),
}
```

### 注释规范

```rust
/// 文档注释：解释函数用途
/// 
/// # Arguments
/// * `ctx` - egui 上下文
/// * `path` - 图片路径
/// 
/// # Returns
/// 加载成功返回 TextureHandle
pub fn load_image(&mut self, ctx: &Context, path: PathBuf) -> Option<TextureHandle> {
    // 行内注释：解释为什么，而不是做什么
    // 使用 LRU 避免内存无限增长
    self.cache.put(path, texture);
}
```

## 🧪 测试建议

- 业务逻辑尽量抽取为纯函数，方便测试
- UI 逻辑通过分离状态机来测试
- 使用 `tracing::info!()` 添加关键路径日志，便于调试

---

*违反本规范的代码应在 Review 中指出并修正*
