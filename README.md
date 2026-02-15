<div align="center">
  <img src="assets/images/clover_viewer.png" width="300" alt="CloverViewer Logo">
  <h1>CloverViewer 三叶草图片查看与截图工具</h1>
  <p>
    基于 Rust 编写的轻量级图片查看与截图工具。<br>
    图标设计源自大写字母 C 和 L —— C 是剑的护手，L 是剑本身。
  </p>
  <p>
    <a href="README.md">中文</a> | <a href="README.en.md">English</a> | <a href="README.ja.md">日本語</a>
  </p>
</div>

---

## 📖 简介

CloverViewer 是一个基于 Rust 开发的轻量级工具，集图片浏览与屏幕截图功能于一体，旨在提供快速、流畅的使用体验。

## ✨ 功能特性

*   **浏览文件夹图片**：方便快捷地浏览文件夹内的所有图片。
*   **查看图片详情**：支持查看图片大图以及详细属性信息。
*   **复制图片**：支持将图片复制到剪贴板。
*   **截图功能**：内置截图工具，方便截取屏幕内容。

## 📸 截图预览

|                           宫格模式                           |                           详情模式                           |
|:--------------------------------------------------------:|:--------------------------------------------------------:|
|  <img src="screenshot/宫格模式.png" width="400" alt="宫格模式">  |  <img src="screenshot/详情模式.png" width="400" alt="详情模式">  |
|                        **截图模式 1**                        |                        **截图模式 2**                        |
| <img src="screenshot/截图模式1.png" width="400" alt="截图模式1"> | <img src="screenshot/截图模式2.png" width="400" alt="截图模式2"> |

## 🖼️ 支持的格式

支持常见的图片格式，包括但不限于：
*   PNG
*   JPEG / JPG
*   GIF
*   BMP
*   WebP
*   TIFF
*   AVIF

## 🛠️ 开发环境搭建

本项目依赖 `dav1d` 库以支持 AVIF 格式图片。在 Windows 环境下编译，建议使用 `vcpkg` 管理 C/C++ 依赖。

### 1. 安装 vcpkg 和 dav1d

请确保已安装 Git。

```powershell
# 1. 克隆 vcpkg 仓库 (建议安装在 C:\vcpkg，也可自定义路径)
git clone https://github.com/microsoft/vcpkg.git C:\vcpkg

# 2. 进入目录并运行引导脚本
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# 3. 安装 dav1d (64位 Windows 版本)
.\vcpkg install dav1d:x64-windows
```

### 2. 安装 pkg-config

你需要安装 `pkg-config` 以便 Rust 构建脚本能找到系统库。

推荐使用 Chocolatey 安装 `pkgconfiglite`：

> **提示**：如果没有安装 Chocolatey，请在**管理员权限**的 PowerShell 中运行以下命令安装：
> ```powershell
> Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
> ```

安装 pkg-config：
```powershell
choco install pkgconfiglite
```
*或者手动下载 `pkg-config.exe` 并将其添加到系统 PATH 环境变量中。*

### 3. 配置环境变量

为了让构建工具找到库文件，需要配置以下环境变量：

*   **`VCPKG_ROOT`**: 指向你的 vcpkg 安装目录 (例如 `C:\vcpkg`)。
*   **`PKG_CONFIG_PATH`**: 指向 vcpkg 的 pkgconfig 目录。
    *   通常路径为: `C:\vcpkg\installed\x64-windows\lib\pkgconfig`

### 4. 构建项目

环境配置完成后，清理并重新构建项目：

```shell
cargo clean
cargo build
```

## 🤝 致谢

特别感谢 **Gemini AI** 在本项目开发过程中提供的宝贵协助。

## 📄 开源协议

本项目遵循 [MIT License](LICENSE) 开源协议。