# 01. Tauriアーキテクチャ概要

このセクションでは、Tauri 2.0のアーキテクチャと、フロントエンド・バックエンドがどのように連携するかを学びます。

## Tauriの基本構造

```
┌─────────────────────────────────────────────────────┐
│                    Tauri アプリ                      │
│  ┌───────────────────┐  ┌────────────────────────┐  │
│  │   フロントエンド    │  │     バックエンド        │  │
│  │  (WebView/HTML)   │  │       (Rust)           │  │
│  │                   │  │                        │  │
│  │  ┌─────────────┐  │  │  ┌──────────────────┐  │  │
│  │  │   React     │  │  │  │  Tauri Commands  │  │  │
│  │  │  TypeScript │◄─┼──┼─►│                  │  │  │
│  │  │             │  │  │  │  rustfeed-core   │  │  │
│  │  └─────────────┘  │  │  └──────────────────┘  │  │
│  │                   │  │                        │  │
│  └───────────────────┘  └────────────────────────┘  │
│           ▲                        ▲                │
│           │        IPC            │                │
│           └────────────────────────┘                │
└─────────────────────────────────────────────────────┘
```

## フロントエンドとバックエンド

### フロントエンド（WebView）

- **技術スタック**: React + TypeScript + Vite
- **役割**: UI/UXの提供
- **実行環境**: OSネイティブのWebView（Chromiumではない）
  - Windows: WebView2 (Edge/Chromiumベース)
  - macOS: WKWebView (Safari/WebKitベース)
  - Linux: WebKitGTK

```typescript
// apps/rustfeed-gui/src/App.tsx
import { invoke } from '@tauri-apps/api/core';

function App() {
  const [feeds, setFeeds] = useState<Feed[]>([]);

  useEffect(() => {
    // Rustバックエンドの関数を呼び出す
    invoke<Feed[]>('get_feeds').then(setFeeds);
  }, []);

  return <FeedList feeds={feeds} />;
}
```

### バックエンド（Rust）

- **技術スタック**: Rust + Tauri + rustfeed-core
- **役割**: ビジネスロジック、データベース操作、ファイルI/O
- **実行環境**: ネイティブバイナリ

```rust
// crates/rustfeed-tauri/src/lib.rs
use rustfeed_core::{db::Database, models::Feed};

#[tauri::command]
async fn get_feeds() -> Result<Vec<Feed>, String> {
    let db = Database::open()?;
    db.get_all_feeds()
        .map_err(|e| e.to_string())
}
```

## IPC（プロセス間通信）

Tauriでは、フロントエンドとバックエンド間の通信にIPCを使用します。

### Commands（フロントエンド → バックエンド）

フロントエンドからRust関数を呼び出す仕組みです。

```typescript
// フロントエンド: JavaScript/TypeScript
import { invoke } from '@tauri-apps/api/core';

// invoke<戻り値の型>('コマンド名', { 引数 })
const feeds = await invoke<Feed[]>('get_feeds');
const feed = await invoke<Feed>('add_feed', { url: 'https://example.com/feed.xml' });
```

```rust
// バックエンド: Rust
#[tauri::command]
async fn get_feeds() -> Result<Vec<Feed>, String> {
    // ...
}

#[tauri::command]
async fn add_feed(url: String) -> Result<Feed, String> {
    // ...
}
```

### Events（バックエンド → フロントエンド）

バックエンドからフロントエンドに通知を送る仕組みです。

```rust
// バックエンド: イベントを発火
app.emit("feed-updated", FeedUpdatePayload { feed_id: 1 })?;
```

```typescript
// フロントエンド: イベントをリッスン
import { listen } from '@tauri-apps/api/event';

await listen('feed-updated', (event) => {
  console.log('Feed updated:', event.payload);
  refreshFeeds();
});
```

## Tauri 2.0の特徴

### 1. Capabilitiesベースのセキュリティ

Tauri 2.0では、アプリが使用できる機能を明示的に宣言します。

```json
// capabilities/default.json
{
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "http:default"
  ]
}
```

### 2. プラグインシステム

機能がプラグインとして分離され、必要なものだけを追加できます。

```toml
# Cargo.toml
[dependencies]
tauri-plugin-shell = "2"
tauri-plugin-http = "2"
```

### 3. モバイルサポート

Tauri 2.0ではiOS/Androidもサポート（同じコードベースで）。

## ディレクトリ構成の詳細

### crates/rustfeed-tauri/

```
rustfeed-tauri/
├── Cargo.toml           # 依存関係
├── src/
│   ├── lib.rs           # Tauri Commands定義
│   └── main.rs          # アプリケーションエントリポイント
├── tauri.conf.json      # Tauri設定（ウィンドウ、ビルド等）
├── capabilities/        # セキュリティ権限
│   └── default.json
└── icons/               # アプリアイコン
```

### apps/rustfeed-gui/

```
rustfeed-gui/
├── package.json         # npm依存関係
├── vite.config.ts       # Vite設定
├── tsconfig.json        # TypeScript設定
├── tailwind.config.js   # Tailwind CSS設定
├── index.html           # エントリHTML
├── src/
│   ├── main.tsx         # Reactエントリポイント
│   ├── App.tsx          # メインコンポーネント
│   ├── components/      # UIコンポーネント
│   ├── hooks/           # カスタムフック
│   ├── types/           # TypeScript型定義
│   └── styles/          # CSS/Tailwind
└── src-tauri/           # → crates/rustfeed-tauriへのリンク
```

## Electronとの比較（詳細）

### バイナリサイズ

```
Electron: ~150MB（Chromiumを同梱）
Tauri:    ~3-10MB（OS標準WebViewを使用）
```

### アーキテクチャ

```
Electron:
  ┌──────────────┐  ┌──────────────┐
  │ Main Process │  │Renderer Proc │
  │  (Node.js)   │◄─►│  (Chromium)  │
  └──────────────┘  └──────────────┘

Tauri:
  ┌──────────────┐  ┌──────────────┐
  │ Rust Backend │  │   WebView    │
  │  (Native)    │◄─►│ (OS Native) │
  └──────────────┘  └──────────────┘
```

### セキュリティモデル

| 観点 | Electron | Tauri |
|------|----------|-------|
| サンドボックス | オプション | デフォルト有効 |
| 権限制御 | 粗い | 細かい（Capabilities） |
| Node.js API | 全アクセス可能 | なし（Rust経由のみ） |
| ファイルアクセス | 制限なし | 明示的な許可が必要 |

## 理解度確認

1. Tauriアプリでフロントエンドからバックエンドの関数を呼び出す仕組みは何ですか？
2. Tauri 2.0のCapabilitiesは何のために使用されますか？
3. TauriがElectronより軽量な理由を説明してください。

## 次のステップ

[02-commands.md](./02-commands.md)で、Tauri Commandsの詳細な実装方法を学びましょう。
