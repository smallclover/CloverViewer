# CloverViewer

CloverViewer 是一个用 Rust (eframe/egui) 编写的 Windows 桌面图片查看器。

## 项目结构

```
src/
├── main.rs          # 入口点
├── app.rs           # 主应用逻辑
├── core/            # 核心功能
│   ├── image_loader.rs  # 图片加载
│   ├── business.rs     # 业务逻辑
│   └── hotkeys.rs      # 全局快捷键
├── model/           # 数据模型
│   ├── state.rs     # 应用状态
│   ├── config.rs    # 配置管理
│   ├── image_meta.rs # 图片元数据
│   └── device.rs    # 设备信息
├── ui/              # UI 组件
│   ├── view/        # 视图模式 (single/grid/arrows)
│   ├── panels/      # 侧边栏 (settings/about/properties)
│   ├── menus/       # 菜单
│   ├── widgets/     # 通用组件 (toast/modal/loading)
│   └── screenshot/  # 截图功能
├── os/              # 平台特定代码
├── utils/           # 工具函数
└── i18n/            # 国际化
```

## 构建与运行

```bash
# 开发模式
cargo run

# 发布模式
cargo build --release
```

打包时会自动执行 `cargo build --release`，生成的可执行文件在 `target/release/`。

## 技术栈

- **UI 框架**: eframe 0.33.3 / egui 0.33.3
- **图片处理**: image 0.25.9 (支持 PNG, JPEG, GIF, BMP, WebP, TIFF, AVIF)
- **依赖库**:
  - `arboard` - 剪贴板
  - `xcap` - 截图
  - `global-hotkey` - 全局快捷键
  - `tray-icon` - 系统托盘
  - `single-instance` - 单实例运行

## 注意事项

- edition = "2024" (Rust 2024 edition)
- 需要 `lib/dav1d.dll` 运行 (已包含在 resources 中)
- 打包时会自动复制 dav1d.dll 到安装目录
