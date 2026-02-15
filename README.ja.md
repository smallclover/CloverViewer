<div align="center">
  <img src="assets/images/clover_viewer.png" width="300" alt="CloverViewer Logo">
  <h1>CloverViewer (クローバービューア)</h1>
  <p>
    Rustで書かれた軽量な画像ビューア。<br>
    ロゴデザインは、大文字のCとLに由来しています。Cは剣のガード、Lは剣そのものを表しています。
  </p>
  <p>
    <a href="README.md">中文</a> | <a href="README.en.md">English</a> | <a href="README.ja.md">日本語</a>
  </p>
</div>

---

## 📖 概要

CloverViewerは、Rustで開発された画像表示ツールで、高速かつスムーズな画像閲覧体験を提供することを目指しています。

## ✨ 機能

*   **フォルダ内画像の閲覧**: フォルダ内のすべての画像を素早く簡単に閲覧できます。
*   **画像詳細の表示**: 高解像度画像の表示や詳細な属性情報の確認が可能です。
*   **画像のコピー**: クリップボードへの画像のコピーをサポートしています。
*   **スクリーンショット**: 画面の内容をキャプチャするためのスクリーンショットツールを内蔵しています。

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
*   AVIF (dav1d.dll)

## 🛠️ 開発環境のセットアップ

このプロジェクトは、AVIF形式の画像をサポートするために `dav1d` ライブラリに依存しています。Windows環境でコンパイルする場合、C/C++の依存関係を管理するために `vcpkg` を使用することをお勧めします。

### 1. vcpkgとdav1dのインストール

Gitがインストールされていることを確認してください。

```powershell
# 1. vcpkgリポジトリをクローンします（C:\vcpkgへのインストールを推奨しますが、カスタムパスも可能です）
git clone https://github.com/microsoft/vcpkg.git C:\vcpkg

# 2. ディレクトリに移動し、ブートストラップスクリプトを実行します
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# 3. dav1dをインストールします（64ビットWindows版）
.\vcpkg install dav1d:x64-windows
```

### 2. pkg-configのインストール

Rustのビルドスクリプトがシステムライブラリを見つけられるように、`pkg-config` をインストールする必要があります。

Chocolateyを使用して `pkgconfiglite` をインストールすることをお勧めします：

> **ヒント**: Chocolateyがインストールされていない場合は、**管理者権限**でPowerShellを開き、以下のコマンドを実行してください：
> ```powershell
> Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
> ```

pkg-configをインストールします：
```powershell
choco install pkgconfiglite
```
*または、手動で `pkg-config.exe` をダウンロードし、システムのPATH環境変数に追加することもできます。*

### 3. 環境変数の設定

ビルドツールがライブラリファイルを見つけられるように、以下の環境変数を設定する必要があります：

*   **`VCPKG_ROOT`**: vcpkgのインストールディレクトリを指します（例：`C:\vcpkg`）。
*   **`PKG_CONFIG_PATH`**: vcpkg内のpkgconfigディレクトリを指します。
    *   一般的なパス：`C:\vcpkg\installed\x64-windows\lib\pkgconfig`

### 4. プロジェクトのビルド

環境設定が完了したら、プロジェクトをクリーンにして再ビルドします：

```shell
cargo clean
cargo build
```

## 🤝 謝辞

本プロジェクトの開発において、多大なる支援をいただいた **Gemini AI** に感謝いたします。

## 📄 ライセンス

このプロジェクトは、[MITライセンス](LICENSE)の下で公開されています。