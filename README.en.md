<div align="center">
  <img src="assets/images/clover_viewer.png" width="300" alt="CloverViewer Logo">
  <h1>CloverViewer Image Viewer & Screenshot Tool</h1>
  <p>
    A lightweight image viewer and screenshot tool written in Rust.<br>
    The logo design is inspired by the capital letters C and L—C forms the guard of a sword, and L is the sword itself.
  </p>
  <p>
    <img src="https://img.shields.io/badge/version-0.0.19-2E7D32" alt="Version">
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

---

## 📖 Introduction

CloverViewer is a lightweight tool developed in Rust, combining image browsing and screen capturing capabilities, designed to provide a fast and smooth user experience.

## ✨ Features

### 🖼️ Image Viewer

*   **Dual View Modes**: Switch between Grid view (thumbnails) and Single view (full image)
*   **Folder Browsing**: Automatically load all images when opening a folder
*   **Quick Navigation**: Use Left/Right arrow keys to switch images quickly
*   **Smooth Zooming**: Mouse wheel zoom with customizable sensitivity
*   **Drag & Drop**: Open images or folders by dragging them into the window
*   **Image Properties**: View detailed information (name, date, path, dimensions, EXIF metadata)
*   **EXIF Metadata**: Read and display camera brand/model, focal length, aperture, ISO, and other shooting parameters
*   **Context Menu**: Copy image, copy path, view properties
*   **Clipboard Integration**: One-click copy image to clipboard

### 📸 Screenshot Tool

*   **Multi-Monitor Support**: Automatically detect and support screenshots across multiple displays
*   **Area Selection**: Freely select the screenshot region
*   **Annotation Tools**:
    *   Rectangle and circle shapes
    *   Arrow annotations
    *   Freehand pencil drawing
    *   Mosaic blur
    *   Text annotations
*   **OCR Text Recognition**: Based on Windows native OCR engine, extract text from images with one click
*   **Magnifier**: Real-time display of mouse position coordinates and color value, support Ctrl+C to copy color
*   **Color Picker**: Long-press tool icon to open color palette, customize drawing colors
*   **Undo**: Ctrl+Z to undo the last drawing operation
*   **Quick Save**: Save to desktop or copy directly to clipboard

### ⚙️ System Features

*   **Single Instance**: Named mutex prevents multiple instances from running simultaneously
*   **System Tray**: Option to minimize to system tray when closing window
*   **Global Hotkeys**:
    *   Default Alt+S to quickly start screenshot
    *   Support custom hotkeys
    *   Global hotkeys that work even when minimized to tray
*   **Auto Start**: Implemented via Windows registry, configurable in the settings panel
*   **Minimize to Tray**: Configurable close behavior (minimize to tray or exit application)
*   **Toast Notifications**: Instant feedback for success, error, and loading states
*   **Portable Configuration**: Settings stored in `%APPDATA%/CloverViewer/`, automatically falls back to the exe directory
*   **Multi-language Support**: Chinese, English, and Japanese interfaces
*   **Settings Panel**: Visual configuration for language, hotkeys, zoom sensitivity, auto start, minimize to tray, and more

## 📸 Screenshots

|                              Grid Mode                               |                             Detail Mode                              |
|:--------------------------------------------------------------------:|:--------------------------------------------------------------------:|
|     <img src="screenshot/宫格模式.png" width="400" alt="Grid Mode">      |    <img src="screenshot/详情模式.png" width="400" alt="Detail Mode">     |
|                        **Screenshot Mode 1**                         |                        **Screenshot Mode 2**                         |
| <img src="screenshot/截图模式1.png" width="400" alt="Screenshot Mode 1"> | <img src="screenshot/截图模式2.png" width="400" alt="Screenshot Mode 2"> |

## 🖼️ Supported Formats

Supports common image formats:
*   PNG
*   JPEG / JPG
*   GIF
*   BMP
*   WebP
*   TIFF
*   ~~AVIF~~ (requires dav1d.dll, currently not enabled)

## 🔧 Tech Stack

*   **Language**: Rust (Edition 2024)
*   **GUI Framework**: egui / eframe
*   **Image Processing**: image, imageproc, tiny-skia
*   **EXIF Parsing**: kamadak-exif
*   **Screen Capture**: xcap
*   **OCR**: Windows.Media.Ocr (native API)
*   **System Integration**: tray-icon, global-hotkey, single-instance, winreg
*   **Serialization**: serde / serde_json
*   **Internationalization**: sys-locale

## 📦 Installation

Go to [GitHub Releases](https://github.com/smallclover/CloverViewer/releases) to download the latest installer.

## 🛠️ Development Environment Setup

Make sure you have the [Rust](https://www.rust-lang.org/tools/install) toolchain installed, then run the following commands to build the project:

```shell
# Build release
cargo build --release

# Package installer
cargo packager --release
```

## 🔄 Continuous Integration

The project uses GitHub Actions for CI. Every push and PR automatically runs:

*   `cargo fmt --check` — Code formatting check
*   `cargo clippy --all-targets -- -D warnings` — Static analysis
*   `cargo test --all-targets` — Unit tests

## ⌨️ Keyboard Shortcuts

### Image Viewer

| Shortcut | Function |
|----------|----------|
| ← | Previous image |
| → | Next image |
| Scroll Wheel | Zoom image |

### Screenshot Tool

| Shortcut | Function |
|----------|----------|
| Alt+S (customizable) | Start screenshot |
| Esc | Exit screenshot |
| Ctrl+Z | Undo last drawing |
| Ctrl+C | Copy color value (when magnifier is enabled) |

## 🤝 Acknowledgements

Special thanks to **Gemini AI**, **DeepSeek**, **MiMo**, and **ChatGPT** for the valuable assistance provided during the development of this project.

## 📄 License

This project is licensed under the [MIT License](LICENSE).
