# CloverViewer 自定义 Skill 命令

本文档定义可在项目中使用的自定义 skill 命令。这些命令可以通过 `/skill-name` 的形式触发（如果配置了 hooks），或作为提示词模板使用。

---

## `/add-feature` - 添加新功能

### 用途
添加一个新的功能模块到项目中

### 参数
- `name`: 功能名称（英文，snake_case）
- `type`: 功能类型 (`viewer` | `screenshot` | `system`)
- `description`: 功能描述

### 执行步骤
1. 在 `src/feature/` 下创建 `name/` 目录
2. 创建 `mod.rs` 并实现 `Feature` trait
3. 如果是 screenshot 工具，在 `toolbar.rs` 添加按钮
4. 在 `src/feature/mod.rs` 中导出
5. 在 `app.rs` 中注册到 Feature 集合
6. 更新 `CLAUDE.md` 记录新功能

### 模板
```rust
// src/feature/name/mod.rs
use crate::core::hotkeys::HotkeyAction;
use crate::feature::Feature;
use crate::model::mode::AppMode;
use crate::model::state::CommonState;
use eframe::egui::Context;

pub struct NameFeature {
    // 状态字段
}

impl NameFeature {
    pub fn new() -> Self {
        Self { }
    }
}

impl Default for NameFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl Feature for NameFeature {
    fn update(&mut self, ctx: &Context, common: &mut CommonState, mode: &mut AppMode) {
        if *mode != AppMode::Name {
            return;
        }
        
        // 实现更新逻辑
    }
    
    fn handle_hotkey(&mut self, action: HotkeyAction) -> Option<AppMode> {
        None
    }
}
```

---

## `/fix-bug` - 修复 Bug

### 用途
系统性地调试和修复问题

### 参数
- `symptom`: 症状描述
- `area`: 可能影响的区域 (可选)

### 执行步骤
1. **收集信息**
   - 阅读相关代码文件
   - 检查最近的 git 提交
   - 查看日志输出点

2. **定位问题**
   - 添加 `tracing::debug!()` 日志
   - 使用 `cargo run` 复现
   - 缩小问题范围

3. **修复**
   - 编写最小修复
   - 确保不引入新问题
   - 遵循 `coding-standards.md`

4. **验证**
   - 测试修复是否有效
   - 检查边界情况
   - 确保其他功能正常

### 检查清单
- [ ] 问题已复现
- [ ] 根因已确定
- [ ] 修复已验证
- [ ] 日志已清理（临时调试日志）
- [ ] 符合编码规范

---

## `/add-i18n` - 添加多语言文本

### 用途
添加新的国际化文本

### 参数
- `key`: 文本键名
- `zh`: 中文文本
- `en`: 英文文本
- `ja`: 日文文本
- `category`: 分类 (`menu` | `tooltip` | `message` | etc.)

### 执行步骤
1. 在 `src/i18n/lang.rs` 的 `TextBundle` struct 中添加字段
2. 在 `ZH_TEXT` / `EN_TEXT` / `JA_TEXT` 中添加翻译
3. 在使用处通过 `get_i18n_text(ctx).key` 访问

### 示例
```rust
// TextBundle 添加
pub tooltip_new_feature: &'static str,

// ZH_TEXT
pub const ZH_TEXT: TextBundle = TextBundle {
    tooltip_new_feature: "新功能",
    // ...
};

// 使用
ui.button("...").on_hover_text(text.tooltip_new_feature);
```

---

## `/add-setting` - 添加设置选项

### 用途
在设置面板添加新的配置项

### 参数
- `name`: 设置项名称
- `type`: 数据类型 (`bool` | `string` | `number` | `enum`)
- `default`: 默认值

### 执行步骤
1. **修改模型** (`src/model/config.rs`)
   - 在 `Config` struct 中添加字段
   - 添加 `default_xxx()` 函数
   - 更新 `Default` trait 实现

2. **修改 UI** (`src/ui/widgets/settings.rs`)
   - 在 `render_settings_window` 添加控件
   - 处理值变更

3. **应用配置**
   - 如需要热键，在 `core/hotkeys.rs` 处理
   - 在相关功能中读取配置

4. **添加 i18n**
   - 在 `lang.rs` 添加设置项标签

### 模板
```rust
// config.rs
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Config {
    pub new_setting: bool,
    // ...
}

fn default_new_setting() -> bool {
    true
}

// settings.rs
ui.checkbox(&mut config.new_setting, text.settings_new_setting);
```

---

## `/add-screenshot-tool` - 添加截图标注工具

### 用途
添加新的截图标注工具（如新增形状、笔刷等）

### 参数
- `name`: 工具名称
- `icon`: 图标类型
- `has_color`: 是否支持颜色选择

### 执行步骤
1. **添加枚举** (`src/feature/screenshot/capture/actions.rs`)
   ```rust
   pub enum ScreenshotTool {
       Rect, Circle, /* ... */ NewTool,
   }
   ```

2. **添加工具栏按钮** (`toolbar.rs`)
   ```rust
   let is_new = state.current_tool == Some(ScreenshotTool::NewTool);
   let btn = draw_icon_button(ui, is_new, IconType::NewTool, 32.0);
   if btn.clicked() { state.current_tool = Some(ScreenshotTool::NewTool); }
   handle_tool_interaction(ui, &btn, ScreenshotTool::NewTool, state);
   ```

3. **实现绘制** (`canvas/draw.rs`)
   ```rust
   match tool {
       ScreenshotTool::NewTool => self.draw_new_tool(painter, shape),
       // ...
   }
   ```

4. **添加 i18n**
   - 在 `lang.rs` 添加工具提示文本

---

## `/optimize-image` - 优化图片处理

### 用途
优化图片加载、显示或缓存逻辑

### 执行步骤
1. 检查当前图片加载方式（`core/image_loader.rs`）
2. 分析性能瓶颈
3. 可能的优化方向：
   - 使用 `zune-jpeg` 替代默认 JPEG 解码器
   - 调整 LRU 缓存大小
   - 实现缩略图预加载
   - 使用 `rayon` 并行处理

---

## `/update-readme` - 更新文档

### 用途
同步更新所有 README 文件

### 执行步骤
1. 检查代码变更（新增功能、移除功能）
2. 更新 `README.md`（中文）
3. 同步更新 `README.en.md`（英文）
4. 同步更新 `README.ja.md`（日文）
5. 检查截图是否需要更新

### 注意
- 保持三语言文档同步
- 功能列表必须一致
- 快捷键表格需更新

---

## 使用建议

### 在 Claude Code 中使用

可以在 `settings.json` 中配置 hooks，例如：

```json
{
  "hooks": {
    "before-task": [
      "read ai/CLAUDE.md",
      "read ai/skills.md"
    ]
  }
}
```

或者直接引用：

```
我想添加一个新的设置选项，请按照 ai/skills.md 中的 `/add-setting` skill 执行：
- name: auto_save_screenshot
- type: bool
- default: false
```

---

*新发现的模式应添加到此文档*
