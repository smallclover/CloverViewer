<div align="center">
  <img src="assets/images/clover_viewer.png" width="300" alt="CloverViewer Logo">
  <h1>CloverViewer 画像閲覧・スクリーンショットツール</h1>
  <p>
    Rustで書かれた軽量な画像閲覧・スクリーンショットツール。<br>
    ロゴデザインは、大文字のCとLに由来しています。Cは剣のガード、Lは剣そのものを表しています。
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

## 📖 概要

CloverViewerは、Rustで開発された軽量ツールで、画像閲覧とスクリーンショット機能を兼ね備え、高速かつスムーズなユーザー体験を提供することを目指しています。

## ✨ 機能

### 🖼️ 画像ビューア

*   **2つの表示モード**: グリッド表示（サムネイル）と単一表示（大画像）の切り替え
*   **フォルダ閲覧**: フォルダを開くと自動的にすべての画像を読み込み
*   **クイックナビゲーション**: 左右矢印キーで画像を素早く切り替え
*   **スムーズなズーム**: マウスホイールでのズーム、感度のカスタマイズ可能
*   **ドラッグ＆ドロップ**: 画像やフォルダをウィンドウにドラッグして開く
*   **画像プロパティ**: 詳細情報の表示（名前、日付、パス、サイズ、EXIFメタデータ）
*   **EXIFメタデータ**: カメラのブランド/モデル、焦点距離、絞り、ISOなどの撮影パラメータを読み取り・表示
*   **コンテキストメニュー**: 画像のコピー、パスのコピー、プロパティの表示
*   **クリップボード連携**: ワンクリックで画像をクリップボードにコピー

### 📸 スクリーンショットツール

*   **マルチモニター対応**: 複数ディスプレイの自動検出とスクリーンショット対応
*   **範囲選択**: 自由にスクリーンショット範囲を選択
*   **注釈ツール**:
    *   矩形、円形
    *   矢印注釈
    *   自由描画の鉛筆
    *   モザイクぼかし
    *   テキスト注釈
*   **OCR文字認識**: WindowsネイティブOCRエンジンを使用、ワンクリックで画像からテキストを抽出
*   **虫眼鏡**: マウス位置の座標とカラー値をリアルタイム表示、Ctrl+Cで色をコピー
*   **カラーピッカー**: ツールアイコンを長押しでカラーパレットを開き、描画色をカスタマイズ
*   **元に戻す**: Ctrl+Zで最後の描画操作を元に戻す
*   **クイック保存**: デスクトップに保存、または直接クリップボードにコピー

### ⚙️ システム機能

*   **シングルインスタンス**: named mutexにより複数起動を防止
*   **システムトレイ**: ウィンドウを閉じるときにシステムトレイに最小化するオプション
*   **グローバルショートカット**:
    *   デフォルトAlt+Sでスクリーンショットを素早く開始
    *   カスタムショートカットのサポート
    *   トレイ最小化時でも機能するグローバルホットキー
*   **自動起動**: Windowsレジストリを介して実装、設定パネルで変更可能
*   **トレイに最小化**: 閉じる動作のカスタマイズ（トレイに最小化またはアプリケーション終了）
*   **Toast通知**: 成功、エラー、読み込み状態の即時フィードバック
*   **ポータブル設定**: 設定は`%APPDATA%/CloverViewer/`に保存、自動的にexeディレクトリにフォールバック
*   **多言語対応**: 中国語、英語、日本語のインターフェース
*   **設定パネル**: 言語、ショートカット、ズーム感度、自動起動、トレイ最小化などの視覚的な設定

## 📸 スクリーンショット

|                               グリッドモード                                |                                詳細モード                                 |
|:--------------------------------------------------------------------:|:--------------------------------------------------------------------:|
|     <img src="screenshot/宫格模式.png" width="400" alt="Grid Mode">      |    <img src="screenshot/详情模式.png" width="400" alt="Detail Mode">     |
|                          **スクリーンショットモード 1**                          |                          **スクリーンショットモード 2**                          |
| <img src="screenshot/截图模式1.png" width="400" alt="Screenshot Mode 1"> | <img src="screenshot/截图模式2.png" width="400" alt="Screenshot Mode 2"> |

## 🖼️ 対応フォーマット

一般的な画像フォーマットをサポートしています：
*   PNG
*   JPEG / JPG
*   GIF
*   BMP
*   WebP
*   TIFF
*   ~~AVIF~~ (dav1d.dllが必要だが、現在は未対応)

## 🔧 技術スタック

*   **言語**: Rust (Edition 2024)
*   **GUIフレームワーク**: egui / eframe
*   **画像処理**: image、imageproc、tiny-skia
*   **EXIF解析**: kamadak-exif
*   **スクリーンキャプチャ**: xcap
*   **OCR**: Windows.Media.Ocr (ネイティブAPI)
*   **システム連携**: tray-icon、global-hotkey、single-instance、winreg
*   **シリアライゼーション**: serde / serde_json
*   **国際化**: sys-locale

## 📦 インストール

[GitHub Releases](https://github.com/smallclover/CloverViewer/releases) から最新版のインストーラーをダウンロードしてください。

## 🛠️ 開発環境のセットアップ

[Rust](https://www.rust-lang.org/tools/install) ツールチェーンがインストールされていることを確認し、以下のコマンドを実行してプロジェクトをビルドします：

```shell
# リリースビルド
cargo build --release

# インストーラーのパッケージ
cargo packager --release
```

## 🔄 継続的インテグレーション

プロジェクトはGitHub Actionsを使用してCIを実施しています。すべてのプッシュとPRに対して以下を自動実行します：

*   `cargo fmt --check` — コードフォーマットチェック
*   `cargo clippy --all-targets -- -D warnings` — 静的解析
*   `cargo test --all-targets` — ユニットテスト

## ⌨️ ショートカットキー

### 画像ビューア

| ショートカット | 機能 |
|--------|------|
| ← | 前の画像 |
| → | 次の画像 |
| スクロールホイール | 画像のズーム |

### スクリーンショットツール

| ショートカット | 機能 |
|----------|----------|
| Alt+S (カスタマイズ可能) | スクリーンショットを開始 |
| Esc | スクリーンショットを終了 |
| Ctrl+Z | 最後の描画を元に戻す |
| Ctrl+C | カラー値をコピー（虫眼鏡有効時） |

## 🤝 謝辞

本プロジェクトの開発において、多大なる支援をいただいた **ChatGPT**、**DeepSeek**、**Gemini AI** および **MiMo** に感謝いたします。

## 📄 ライセンス

このプロジェクトは、[MITライセンス](LICENSE)の下で公開されています。
