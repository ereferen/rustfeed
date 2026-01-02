# 06. ビルドとデプロイ

このセクションでは、Tauriアプリケーションのビルドと各プラットフォームへのデプロイについて学びます。

## 開発ビルド

### 開発サーバーの起動

```bash
cd apps/rustfeed-gui
npm run tauri dev
```

このコマンドは以下を実行します：
1. Vite開発サーバーを起動（HMR対応）
2. Rustバックエンドをコンパイル
3. Tauriウィンドウを起動

### 開発時の便利な設定

```json
// tauri.conf.json
{
  "build": {
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "rustfeed (dev)",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

## 本番ビルド

### 基本的なビルド

```bash
cd apps/rustfeed-gui
npm run tauri build
```

### プラットフォーム固有のビルド

```bash
# Windows (MSI インストーラー)
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS (DMG + App Bundle)
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin  # Apple Silicon

# Linux (AppImage, deb, rpm)
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

### ビルド出力

```
target/release/bundle/
├── macos/
│   ├── rustfeed.app/
│   └── rustfeed.dmg
├── msi/
│   └── rustfeed_0.1.0_x64.msi
├── nsis/
│   └── rustfeed_0.1.0_x64-setup.exe
├── deb/
│   └── rustfeed_0.1.0_amd64.deb
├── rpm/
│   └── rustfeed-0.1.0-1.x86_64.rpm
└── appimage/
    └── rustfeed_0.1.0_amd64.AppImage
```

## ビルド設定

### tauri.conf.json

```json
{
  "productName": "rustfeed",
  "version": "0.1.0",
  "identifier": "com.example.rustfeed",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    },
    "macOS": {
      "entitlements": null,
      "exceptionDomain": "",
      "frameworks": [],
      "providerShortName": null,
      "signingIdentity": null
    },
    "linux": {
      "deb": {
        "depends": []
      },
      "appimage": {
        "bundleMediaFramework": false
      }
    }
  }
}
```

### アイコンの生成

```bash
# Tauri CLIでアイコンを生成
cargo tauri icon path/to/source-icon.png

# 出力
# icons/
# ├── 32x32.png
# ├── 128x128.png
# ├── 128x128@2x.png
# ├── icon.icns (macOS)
# └── icon.ico (Windows)
```

## クロスコンパイル

### GitHub Actions での自動ビルド

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target aarch64-apple-darwin'
          - platform: 'macos-latest'
            args: '--target x86_64-apple-darwin'
          - platform: 'ubuntu-22.04'
            args: ''
          - platform: 'windows-latest'
            args: ''

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: lts/*

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Install dependencies (ubuntu only)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - name: Install frontend dependencies
        run: npm install
        working-directory: apps/rustfeed-gui

      - name: Build the app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'rustfeed v__VERSION__'
          releaseBody: 'See the assets to download and install this version.'
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
          projectPath: apps/rustfeed-gui
```

## 自動更新

### Tauri Updater プラグイン

```toml
# Cargo.toml
[dependencies]
tauri-plugin-updater = "2"
```

```rust
// main.rs
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .run(tauri::generate_context!())
        .unwrap();
}
```

```json
// capabilities/default.json
{
  "permissions": [
    "updater:default"
  ]
}
```

### 更新チェック（フロントエンド）

```typescript
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

async function checkForUpdates() {
  const update = await check();

  if (update) {
    console.log(`Update available: ${update.version}`);

    // ダウンロードとインストール
    await update.downloadAndInstall();

    // アプリを再起動
    await relaunch();
  }
}
```

### 更新サーバーの設定

```json
// tauri.conf.json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.example.com/rustfeed/{{target}}/{{arch}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

## デバッグビルド

### デバッグ情報付きリリースビルド

```bash
# Rustのデバッグシンボルを含める
CARGO_PROFILE_RELEASE_DEBUG=true npm run tauri build
```

### ログの有効化

```rust
// main.rs
fn main() {
    // 環境変数でログレベルを設定
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .unwrap();
}
```

## バイナリサイズの最適化

### Cargo.toml の設定

```toml
[profile.release]
lto = true           # Link-Time Optimization
opt-level = "s"      # サイズ最適化 ("z" はさらに小さく)
codegen-units = 1    # 単一ユニットでコンパイル
strip = true         # シンボルを削除
panic = "abort"      # パニック時にアボート
```

### 不要な依存の削除

```toml
[dependencies]
tauri = { version = "2", default-features = false, features = ["macos-private-api"] }
```

## プラットフォーム固有の注意点

### Windows

- **コード署名**: Microsoft Store への配布には必須
- **WebView2**: Windows 10/11にはプリインストール、古いOSでは同梱が必要

### macOS

- **公証 (Notarization)**: Gatekeeper対応に必要
- **コード署名**: 配布には Apple Developer アカウントが必要
- **Universal Binary**: Intel + Apple Silicon の両対応

```bash
# Universal Binary のビルド
npm run tauri build -- --target universal-apple-darwin
```

### Linux

- **依存関係**: WebKitGTK等が必要
- **AppImage**: 依存関係を同梱、どのディストロでも動作
- **Flatpak/Snap**: サンドボックス環境での配布

## 理解度確認

1. `npm run tauri build` は何を生成しますか？
2. クロスプラットフォームビルドを自動化する方法は？
3. バイナリサイズを最適化する方法を2つ挙げてください。

## まとめ

このガイドでは、Tauri 2.0 を使ったデスクトップアプリケーション開発の基礎を学びました：

1. **アーキテクチャ**: フロントエンド（React）とバックエンド（Rust）の分離
2. **Commands**: フロントエンドからRust関数を呼び出す
3. **状態管理**: Mutex/RwLockによるスレッドセーフな状態管理
4. **Events**: バックエンドからフロントエンドへの通知
5. **セキュリティ**: Capabilitiesによる細かい権限制御
6. **ビルド**: 各プラットフォーム向けのビルドとデプロイ

これらの知識を活かして、rustfeed GUIの実装を進めていきましょう！

## 参考リソース

- [Tauri 公式ドキュメント](https://tauri.app/v2/guides/)
- [Tauri GitHub リポジトリ](https://github.com/tauri-apps/tauri)
- [Tauri Discord コミュニティ](https://discord.com/invite/tauri)
