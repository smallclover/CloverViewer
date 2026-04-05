# egui 框架使用模式指南

> 本文档记录 CloverViewer 项目中使用的 egui 编程模式和最佳实践

## 🎯 核心概念

### 即时模式 (Immediate Mode)

egui 是即时模式 GUI 框架，每一帧都重新构建整个 UI：

```rust
// 每帧都执行，不只是初始化
ui.label("Hello");  // 立即绘制
if ui.button("Click").clicked() {
    // 处理点击
}
```

**关键理解：**
- 没有持久化的 UI 对象，只有每帧的描述
- 状态由你管理，egui 只负责绘制
- 响应（Response）只在当前帧有效

## 📐 布局模式

### 标准应用布局

```rust
CentralPanel::default().show(ctx, |ui| {
    // 主内容区域
});

TopBottomPanel::top("menu").show(ctx, |ui| {
    // 顶部菜单
});

TopBottomPanel::bottom("status").show(ctx, |ui| {
    // 底部状态栏
});

SidePanel::left("sidebar").show(ctx, |ui| {
    // 侧边栏
});
```

**注意：** Panel 的 `show` 内部不能再包含同类型的 Panel（如 top 里不能再放 top）。

### 条件渲染

```rust
// ✅ 正确的条件渲染
if self.show_modal {
    Window::new("Modal").show(ctx, |ui| {
        ui.label("内容");
    });
}

// ✅ 使用匹配更清晰
match self.current_overlay {
    OverlayMode::Settings => self.show_settings(ctx),
    OverlayMode::About => self.show_about(ctx),
    _ => {}
}
```

### 自定义组件封装

```rust
/// 绘制带图标的按钮
pub fn draw_icon_button(
    ui: &mut Ui,
    active: bool,
    icon_type: IconType,
    size: f32,
) -> Response {
    let button = ImageButton::new(get_icon_texture(icon_type))
        .frame(active)
        .tint(if active { ACTIVE_COLOR } else { INACTIVE_COLOR });
    
    ui.add_sized([size, size], button)
}

// 使用
if draw_icon_button(ui, true, IconType::Grid, 32.0).clicked() {
    self.mode = ViewMode::Grid;
}
```

## 🎨 绘制模式

### 自定义绘制（截图标注）

```rust
// 获取 painter（相对于整个屏幕的坐标系）
let painter = ui.painter();

// 绘制矩形
painter.rect_stroke(
    rect,
    rounding,
    Stroke::new(width, color),
    StrokeKind::Inside,
);

// 绘制线条
painter.line_segment([start, end], stroke);

// 绘制文字
painter.text(
    pos,
    Align2::CENTER_CENTER,
    text,
    FontId::new(size, FontFamily::Proportional),
    color,
);
```

### 响应交互区域

```rust
// 创建可交互的区域
let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

// 检查交互
if response.clicked() {
    // 点击
}
if response.dragged() {
    // 拖拽中
    let delta = response.drag_delta();
}
if response.hovered() {
    // 悬停
}
```

## 💾 状态管理

### 使用 Context 存储临时数据

```rust
// 存储（仅当前帧有效）
ui.data_mut(|d| d.insert_temp(Id::new("key"), value));

// 读取
let value = ui.data(|d| d.get_temp::<T>(Id::new("key")));
```

### 跨帧持久化

```rust
// 使用 persistent data（egui 0.33+）
ctx.data_mut(|d| {
    d.insert_persisted(Id::new("config"), config);
});

// 或自己管理（项目中使用的方式）
pub struct App {
    config: Arc<Config>,
    // ...
}
```

### 项目中使用的 Config 模式

```rust
// model/config.rs
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub language: Language,
    pub hotkeys: HotkeysConfig,
    // ...
}

// 加载/保存到文件
pub fn load_config() -> Config { /* ... */ }
pub fn save_config(config: &Config) { /* ... */ }

// 在 Context 中临时存储
pub fn get_context_config(ctx: &Context) -> Arc<Config> {
    ctx.data(|d| d.get_temp::<Arc<Config>>(Id::new("config")))
        .unwrap_or_else(|| Arc::new(Config::default()))
}
```

## 🖱️ 交互模式

### 拖拽实现（截图选区）

```rust
let response = ui.allocate_response(ui.available_size(), Sense::drag());

if response.drag_started() {
    self.drag_start = response.interact_pointer_pos();
}

if response.dragged() {
    if let Some(current) = response.interact_pointer_pos() {
        self.current_rect = Rect::from_two_pos(self.drag_start.unwrap(), current);
    }
}

if response.drag_released() {
    self.finalize_selection();
}
```

### 长按交互（颜色选择器）

```rust
let button = ui.button("Tool");

// 检测长按
if button.is_pointer_button_down_on() {
    if let Some(press_time) = ui.input(|i| i.pointer.press_start_time()) {
        let duration = ui.input(|i| i.time) - press_time;
        if duration > 0.6 {
            self.show_color_picker = true;
        }
    }
}
```

### 处理原始输入

```rust
// 键盘
if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
    self.prev_image();
}

// 滚轮缩放
let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
if scroll != 0.0 {
    self.zoom += scroll * 0.1;
}

// 拖拽文件
if let Some(path) = ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone())) {
    self.open_file(path);
}
```

## 🪟 Window 管理

### 模态窗口

```rust
let mut open = true;
Window::new("Settings")
    .open(&mut open)  // 关闭按钮会设置 open = false
    .resizable(false)
    .collapsible(false)
    .show(ctx, |ui| {
        // 内容
    });

if !open {
    self.show_settings = false;  // 同步到状态
}
```

### 设置窗口约束

```rust
// 限制窗口最小尺寸
ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(Vec2::new(750.0, 550.0)));

// 恢复最小尺寸
ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(Vec2::new(100.0, 100.0)));
```

## 🎭 动画与效果

### 加载动画

```rust
// 使用旋转的 spinner
ui.spinner();

// 自定义动画
let angle = ctx.input(|i| i.time) as f32;
let rotation = emath::Rot2::from_angle(angle);
```

### Toast 通知系统

项目中已实现：

```rust
// 显示 Toast
common.toast_manager.show(Toast {
    text: text.copied_message.into(),
    duration: Duration::from_secs(2),
});

// 在 draw 中更新
toast_system.update(ctx);
```

## 🔧 性能优化

### 1. 避免每帧重分配

```rust
// ❌ 每帧分配新 Vec
let items: Vec<_> = (0..100).collect();

// ✅ 复用存储
self.items.clear();
self.items.extend(0..100);
```

### 2. 纹理缓存

```rust
// 缓存已加载的图片纹理
if let Some(tex) = self.texture_cache.get(&path) {
    ui.image(tex);
} else {
    // 异步加载
}
```

### 3. 按需重绘

```rust
// 当需要动画或更新时主动请求重绘
ctx.request_repaint();

// 限制重绘率
ctx.request_repaint_after(Duration::from_millis(100));
```

### 4. 使用 show_viewport 处理复杂场景

```rust
// 截图功能使用独立 viewport 避免主窗口干扰
ctx.show_viewport_immediate(
    ViewportId::from("screenshot"),
    ViewportBuilder::default()
        .with_transparent(true)
        .with_decorations(false),
    |ctx| {
        // 截图绘制逻辑
    },
);
```

## 📚 常用 Snippets

### 带提示的按钮

```rust
ui.button("Save")
    .on_hover_text("Save to desktop (Ctrl+S)");
```

### 禁用状态

```rust
ui.add_enabled(!self.is_loading, Button::new("Load"));
```

### 水平布局

```rust
ui.horizontal(|ui| {
    ui.label("Name:");
    ui.text_edit_singleline(&mut self.name);
});
```

### 网格布局

```rust
ui.columns(3, |columns| {
    for (i, item) in items.iter().enumerate() {
        columns[i % 3].label(item);
    }
});
```

### 滚动区域

```rust
ScrollArea::vertical().show(ui, |ui| {
    for item in &self.items {
        ui.label(item);
    }
});
```

---

*更多模式请参考项目中已实现的功能模块*
