<div align="center">
  <img src="assets/images/clover_viewer.png" width="300" alt="CloverViewer Logo">
  <h1>CloverViewer 三叶草图片查看与截图工具</h1>
  <p>
    基于 Rust 编写的轻量级图片查看与截图工具。<br>
    图标设计源自大写字母 C 和 L —— C 是剑的护手，L 是剑本身。
  </p>
  <p>
    <img src="https://img.shields.io/badge/version-0.0.20-2E7D32" alt="Version">
    <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
    <a href="https://github.com/smallclover/CloverViewer/actions/workflows/ci.yml"><img src="https://github.com/smallclover/CloverViewer/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  </p>
  <p>
    <a href="https://github.com/smallclover"><img src="https://img.shields.io/badge/Author-smallclover-green" alt="Author"></a>
  </p>
  <p>
    <a href="https://openai.com"><img src="https://img.shields.io/badge/Powered%20by-ChatGPT-10A37F" alt="ChatGPT"></a>
    <a href="https://www.deepseek.com"><img src="https://img.shields.io/badge/Powered%20by-DeepSeek-2589BD" alt="DeepSeek"></a>
    <a href="https://gemini.google.com"><img src="https://img.shields.io/badge/Powered%20by-Gemini-8E75B2" alt="Gemini"></a>
    <a href="https://mimo-ai.com"><img src="https://img.shields.io/badge/Powered%20by-MiMo-FF6B35" alt="MiMo"></a>
  </p>
  <p>
    <a href="README.md">中文</a> | <a href="README.en.md">English</a> | <a href="README.ja.md">日本語</a>
  </p>
</div>

CloverViewer is a free, open-source, lightweight screenshot tool and image viewer for Windows, written in Rust. Features include screen capture, annotation, OCR, EXIF metadata, multi-monitor support. CloverViewer 是一款免费开源的 Windows 截图与图片查看工具，使用 Rust 编写，支持截图标注、OCR 文字识别、EXIF 元数据等功能。

---

## 📖 简介 Introduction

CloverViewer 是一个基于 Rust 开发的轻量级工具，集图片浏览与屏幕截图功能于一体，旨在提供快速、流畅的使用体验。

## ✨ 功能特性 Features

### 🖼️ 图片查看器

*   **双视图模式**：支持网格视图（缩略图）和单图视图（大图）切换
*   **文件夹浏览**：打开文件夹自动加载其中所有图片
*   **快速导航**：键盘左右方向键快速切换图片
*   **流畅缩放**：鼠标滚轮缩放图片，支持自定义缩放灵敏度
*   **拖拽打开**：直接拖拽图片或文件夹到窗口即可打开
*   **图片属性**：查看图片详细信息（名称、日期、路径、尺寸、EXIF 元数据）
*   **EXIF 元数据**：读取并显示相机品牌/型号、焦距、光圈、ISO 等拍摄参数
*   **右键菜单**：支持复制图片、复制路径、查看属性
*   **剪贴板集成**：一键复制图片到剪贴板

### 📸 截图工具

*   **多显示器支持**：自动识别并支持跨显示器截图
*   **选区截图**：自由选择截图区域
*   **标注工具**：
    *   矩形、圆形框选
    *   箭头标注
    *   自由铅笔涂鸦
    *   马赛克模糊
    *   文字标注
*   **OCR 文字识别**：基于 Windows 原生 OCR 引擎，一键提取图片中的文字
*   **放大镜**：实时显示鼠标位置坐标和色值，支持 Ctrl+C 复制颜色
*   **颜色选择器**：长按工具图标打开调色盘，自定义绘制颜色
*   **撤销操作**：Ctrl+Z 撤销上一步绘制
*   **快捷保存**：支持保存到桌面或直接复制到剪贴板

### ⚙️ 系统功能

*   **单实例运行**：通过 named mutex 防止程序重复打开
*   **系统托盘**：关闭窗口时可选最小化到系统托盘
*   **全局快捷键**：
    *   默认 Alt+S 快速启动截图
    *   支持自定义快捷键
    *   托盘状态下也能唤起的全局热键
*   **开机自启动**：通过 Windows 注册表实现，可在设置面板中配置
*   **最小化到托盘**：可配置的关闭行为（最小化到托盘或退出程序）
*   **Toast 通知**：操作成功、错误、加载状态的即时反馈
*   **便携配置**：配置存储在 `%APPDATA%/CloverViewer/`，自动回退到 exe 目录
*   **多语言支持**：中文、英文、日文三语言界面
*   **设置面板**：可视化配置语言、快捷键、缩放灵敏度、开机自启动、最小化到托盘等选项

## 📸 截图预览 Screenshots

|                           宫格模式                           |                           详情模式                           |
|:--------------------------------------------------------:|:--------------------------------------------------------:|
|  <img src="screenshot/宫格模式.png" width="400" alt="宫格模式">  |  <img src="screenshot/详情模式.png" width="400" alt="详情模式">  |
|                        **截图模式 1**                        |                        **截图模式 2**                        |
| <img src="screenshot/截图模式1.png" width="400" alt="截图模式1"> | <img src="screenshot/截图模式2.png" width="400" alt="截图模式2"> |

## 🖼️ 支持的格式 Supported Formats

支持常见的图片格式：
*   PNG
*   JPEG / JPG
*   GIF
*   BMP
*   WebP
*   TIFF
*   ~~AVIF~~ (需要 dav1d.dll，目前暂未启用)

## 🔧 技术栈 Tech Stack

*   **语言**：Rust (Edition 2024)
*   **GUI 框架**：egui / eframe
*   **图片处理**：image、imageproc、tiny-skia
*   **EXIF 解析**：kamadak-exif
*   **屏幕截图**：xcap
*   **OCR**：Windows.Media.Ocr (原生 API)
*   **系统集成**：tray-icon、global-hotkey、single-instance、winreg
*   **序列化**：serde / serde_json
*   **国际化**：sys-locale

## 📦 安装 Installation

前往 [GitHub Releases](https://github.com/smallclover/CloverViewer/releases) 下载最新版本的安装包。

## 🛠️ 开发环境搭建 Development Setup

确保已安装 [Rust](https://www.rust-lang.org/tools/install) 工具链，然后执行以下命令构建项目：

```shell
# 构建发行版
cargo build --release

# 打包安装文件
cargo packager --release
```

## 🔄 持续集成 CI

项目使用 GitHub Actions 进行 CI，每次推送和 PR 会自动执行：

*   `cargo fmt --check` — 代码格式检查
*   `cargo clippy --all-targets -- -D warnings` — 静态分析
*   `cargo test --all-targets` — 单元测试

## ⌨️ 快捷键 Keyboard Shortcuts

### 图片查看器

| 快捷键 | 功能 |
|--------|------|
| ← | 上一张图片 |
| → | 下一张图片 |
| 滚轮 | 缩放图片 |

### 截图工具

| 快捷键 | 功能 |
|--------|------|
| Alt+S (可自定义) | 启动截图 |
| Esc | 退出截图 |
| Ctrl+Z | 撤销上一步绘制 |
| Ctrl+C | 复制颜色值（在放大镜开启时） |

## 🤝 致谢 Acknowledgements

特别感谢 **ChatGPT**、**DeepSeek**、**Gemini AI** 和 **MiMo** 在本项目开发过程中提供的宝贵协助。

## 📄 开源协议 License

本项目遵循 [MIT License](LICENSE) 开源协议。
