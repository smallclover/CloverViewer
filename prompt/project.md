# CloverViewer 开发 Prompt 约束（Rust + eframe/egui）

本文件用于约束 **AI 在协助 CloverViewer 项目开发时** 的行为规范。

每次进行相关代码生成、修改、排错、架构设计时，**必须先阅读本文件内容并严格遵守**。

---

## 项目关键技术栈

* Rust edition: **2024**
* GUI: **eframe 0.33.x + egui 0.33.x**
* 截图: **xcap 0.8.x**
* 图像处理: **image 0.25.x（avif-native + dav1d.dll）**
* 剪贴板: **arboard**
* 多线程: **rayon / std::thread**
* Windows API: **windows 0.62.x**

---

## 强制规则（必须遵守）

### 1. 每次回答前必须先阅读本文件

在给出任何代码、修改建议、API 用法之前：

> 必须先参考本文件，确保不违反下面的规则。

---

### 2. 必须使用当前依赖版本的**最新 API**

禁止：

* 使用已废弃（deprecated）的 API
* 使用旧版本教程/博客中的写法
* 使用与当前依赖版本不匹配的示例代码

例如：

| 错误做法                   | 错误原因              |
| ---------------------- |-------------------|
| 使用 `frame.set_visible` | 方法不存在             |
| 使用 `image.rgba()`      | 方法不存在或者使用的是旧版本api |

例子3： 下面的方法有四个参数，而不是三个，请注意
```rust
egui::painter
impl Painter
pub fn rect_stroke(&self, rect: Rect, corner_radius: impl Into<CornerRadius>, stroke: impl Into<Stroke>, stroke_kind: StrokeKind) -> ShapeIdx
```
---

### 3. 当发现“方法不存在 / API 报错”时

**禁止猜测写法**。

必须：

1. 查阅该 crate 的**官方文档 / 源码**
2. 确认当前版本真实存在的方法
3. 按真实 API 给出代码

---

### 4. 严格遵守 eframe/egui 的立即模式模型

禁止：

* 在 UI 线程 `sleep`
* 阻塞 UI 线程


---

### 5. 代码目标

生成的代码必须满足：

* 可直接编译通过
* 与当前 Cargo.toml 完全匹配
* 无过时 API
* 无伪代码
* 无“示例性质”的不完整代码

---

#### 6. 禁止使用已在 egui 0.33 中废弃的 API（重点）

以下 API **在 egui 0.33 已被废弃或行为改变**，严禁再出现在任何生成代码中：

| 禁止使用                           | 必须使用                                         |
|--------------------------------|----------------------------------------------| 
| `ctx.input(i i.screen_rect())` | `ctx.content_rect()` 或 `ctx.viewport_rect()` |  
| 依赖 `screen_rect` 做全屏绘制         | 使用 viewport / content rect                   |

说明：

* `screen_rect` 是旧版本 egui 的遗留概念
* 0.33 开始引入 viewport / content 的严格区分
* 再使用 `screen_rect` 会产生警告甚至逻辑错误

当需要获取绘制区域时：

```rust
let rect = ctx.content_rect();
```

或

```rust
let rect = ctx.viewport_rect();
```

---

#### 避免常见的错误

1. rect_stroke 有四个参数
2. 时刻注意是否需要unwrap或者？来解包返回值
3. xcap的很多返回类型都是XCapResult<T>,需要unwrap来解包

## 目标

确保 AI 生成的所有相关代码：

> **100% 可直接用于 CloverViewer 项目，而不需要再手动排错 API 问题**。


