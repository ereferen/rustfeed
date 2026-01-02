# Tauri学習ガイド - rustfeed GUIで学ぶTauri 2.0

このディレクトリは、rustfeed GUIアプリケーションの実装を通じて、Tauri 2.0を使ったデスクトップアプリ開発を学ぶための資料集です。

## 対象読者

- Rustの基礎を理解している方（`docs/learning/`を参照）
- React/TypeScriptの経験がある方
- Electronなど他のデスクトップアプリフレームワークの経験がある方（比較として）

## Tauriとは

Tauriは、Web技術（HTML/CSS/JavaScript）でUIを構築し、Rustでバックエンドロジックを実装するデスクトップアプリケーションフレームワークです。

### Electronとの比較

| 特徴 | Tauri | Electron |
|------|-------|----------|
| バイナリサイズ | ~3MB | ~150MB |
| メモリ使用量 | 少ない | 多い |
| バックエンド言語 | Rust | JavaScript (Node.js) |
| ブラウザエンジン | OS標準のWebView | Chromium同梱 |
| セキュリティ | 強固（Rust + 細かい権限制御） | 注意が必要 |
| モバイル対応 | Tauri 2.0でサポート | なし |

### なぜTauriを選ぶのか

1. **軽量**: Chromiumを同梱しないため、バイナリが小さい
2. **高性能**: Rustバックエンドによる高速な処理
3. **セキュア**: Rustの安全性 + 細かい権限制御（Capabilities）
4. **クロスプラットフォーム**: Windows, macOS, Linux, Android, iOS

## 学習トピック

| # | トピック | 内容 |
|---|----------|------|
| 1 | [アーキテクチャ概要](./01-architecture.md) | Tauriの構造とフロントエンド・バックエンドの関係 |
| 2 | [Commands (IPC)](./02-commands.md) | RustとJavaScript間の通信 |
| 3 | [状態管理](./03-state-management.md) | Tauri側の状態管理とフロントエンドとの連携 |
| 4 | [イベントシステム](./04-events.md) | バックエンドからフロントエンドへの通知 |
| 5 | [セキュリティとCapabilities](./05-security.md) | Tauri 2.0のセキュリティモデル |
| 6 | [ビルドとデプロイ](./06-build-deploy.md) | 各プラットフォーム向けのビルド |

## rustfeed GUI プロジェクト構造

```
rustfeed/
├── crates/
│   ├── rustfeed-core/        # 共有ライブラリ（既存）
│   ├── rustfeed-cli/         # CLI（既存）
│   ├── rustfeed-tui/         # TUI（既存）
│   └── rustfeed-tauri/       # Tauri バックエンド
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs        # Tauri Commands
│       │   └── main.rs       # エントリポイント
│       ├── tauri.conf.json   # Tauri設定
│       └── capabilities/     # セキュリティ権限
└── apps/
    └── rustfeed-gui/         # React フロントエンド
        ├── package.json
        ├── vite.config.ts
        ├── src/
        │   ├── main.tsx
        │   ├── App.tsx
        │   └── components/
        └── src-tauri/        # → crates/rustfeed-tauri へのシンボリックリンク
```

## 開発環境のセットアップ

### 必要なツール

```bash
# Rust（最新の安定版）
rustup update stable

# Tauri CLI
cargo install tauri-cli

# Node.js (18以上推奨)
node --version  # v18.x.x以上

# 各OS固有の依存関係（後述）
```

### OS固有の依存関係

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### macOS
```bash
xcode-select --install
```

#### Windows
- Microsoft Visual Studio C++ Build Tools
- WebView2（Windows 10/11には通常プリインストール）

## 学習の進め方

1. まず[01-architecture.md](./01-architecture.md)でTauriの全体像を把握
2. [02-commands.md](./02-commands.md)でRust-JS間通信の基本を学ぶ
3. rustfeed-tauriのコードを読みながら実践的な実装を理解
4. 各セクションの練習問題に挑戦

## クイックスタート

```bash
# 開発サーバー起動
cd apps/rustfeed-gui
npm install
npm run tauri dev

# ビルド
npm run tauri build
```

## 参考リソース

- [Tauri公式ドキュメント](https://tauri.app/v2/guides/)
- [Tauri 2.0リリースノート](https://tauri.app/blog/tauri-2-0/)
- [Tauriサンプルアプリ](https://github.com/tauri-apps/tauri/tree/dev/examples)

---

それでは、[01-architecture.md](./01-architecture.md)から始めましょう！
