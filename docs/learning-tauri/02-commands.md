# 02. Tauri Commands (IPC)

このセクションでは、フロントエンド（JavaScript/TypeScript）からバックエンド（Rust）の関数を呼び出す「Commands」について学びます。

## Commandsとは

Tauri Commandsは、フロントエンドからバックエンドのRust関数を呼び出すための仕組みです。RPCやRemote Procedure Callに似ています。

```
フロントエンド                    バックエンド
     │                              │
     │  invoke('get_feeds')         │
     │ ─────────────────────────────►│
     │                              │ Rust関数実行
     │  Result<Vec<Feed>>           │
     │ ◄─────────────────────────────│
     │                              │
```

## 基本的なCommand

### Rust側の定義

```rust
// crates/rustfeed-tauri/src/lib.rs

/// #[tauri::command] 属性でCommandとして公開
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// main.rsでCommandを登録
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### TypeScript側からの呼び出し

```typescript
// apps/rustfeed-gui/src/App.tsx
import { invoke } from '@tauri-apps/api/core';

async function sayHello() {
  // invoke<戻り値の型>('コマンド名', { 引数 })
  const message = await invoke<string>('greet', { name: 'World' });
  console.log(message); // "Hello, World!"
}
```

## rustfeed-tauriでの実装例

### フィード操作Commands

```rust
// crates/rustfeed-tauri/src/lib.rs
use rustfeed_core::{db::Database, models::{Feed, Article}};
use tauri::State;
use std::sync::Mutex;

// 状態としてDatabaseを保持
pub struct AppState {
    pub db: Mutex<Database>,
}

/// 全フィードを取得
#[tauri::command]
async fn get_feeds(state: State<'_, AppState>) -> Result<Vec<Feed>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_feeds().map_err(|e| e.to_string())
}

/// フィードを追加
#[tauri::command]
async fn add_feed(url: String, state: State<'_, AppState>) -> Result<Feed, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // フィードを取得してパース
    let feed_data = rustfeed_core::feed::fetch_feed(&url)
        .await
        .map_err(|e| e.to_string())?;

    // データベースに保存
    db.add_feed(&url, &feed_data.title)
        .map_err(|e| e.to_string())
}

/// フィードを削除
#[tauri::command]
async fn delete_feed(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_feed(id).map_err(|e| e.to_string())
}
```

### 記事操作Commands

```rust
/// 記事一覧を取得（フィルタ付き）
#[tauri::command]
async fn get_articles(
    feed_id: Option<i64>,
    unread_only: bool,
    favorites_only: bool,
    search: Option<String>,
    limit: i64,
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // フィルタ条件に応じて記事を取得
    db.get_articles(feed_id, unread_only, favorites_only, search.as_deref(), limit)
        .map_err(|e| e.to_string())
}

/// 既読/未読を切り替え
#[tauri::command]
async fn toggle_read(id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_article_read(id).map_err(|e| e.to_string())
}

/// お気に入りを切り替え
#[tauri::command]
async fn toggle_favorite(id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_article_favorite(id).map_err(|e| e.to_string())
}
```

### Commandsの登録

```rust
// crates/rustfeed-tauri/src/main.rs
fn main() {
    let db = Database::open().expect("Failed to open database");

    tauri::Builder::default()
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            get_feeds,
            add_feed,
            delete_feed,
            get_articles,
            toggle_read,
            toggle_favorite,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## TypeScript型定義

フロントエンドで型安全に使うための型定義を作成します。

```typescript
// apps/rustfeed-gui/src/types/index.ts

export interface Feed {
  id: number;
  url: string;
  title: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface Article {
  id: number;
  feed_id: number;
  title: string;
  url: string;
  content: string | null;
  published_at: string | null;
  is_read: boolean;
  is_favorite: boolean;
  created_at: string;
}

// Tauri Commands のラッパー関数
// apps/rustfeed-gui/src/api/commands.ts
import { invoke } from '@tauri-apps/api/core';
import type { Feed, Article } from '../types';

export async function getFeeds(): Promise<Feed[]> {
  return invoke<Feed[]>('get_feeds');
}

export async function addFeed(url: string): Promise<Feed> {
  return invoke<Feed>('add_feed', { url });
}

export async function deleteFeed(id: number): Promise<void> {
  return invoke('delete_feed', { id });
}

export async function getArticles(options: {
  feedId?: number;
  unreadOnly?: boolean;
  favoritesOnly?: boolean;
  search?: string;
  limit?: number;
}): Promise<Article[]> {
  return invoke<Article[]>('get_articles', {
    feed_id: options.feedId ?? null,
    unread_only: options.unreadOnly ?? false,
    favorites_only: options.favoritesOnly ?? false,
    search: options.search ?? null,
    limit: options.limit ?? 100,
  });
}

export async function toggleRead(id: number): Promise<boolean> {
  return invoke<boolean>('toggle_read', { id });
}

export async function toggleFavorite(id: number): Promise<boolean> {
  return invoke<boolean>('toggle_favorite', { id });
}
```

## エラーハンドリング

### Rust側

Commandsでは`Result<T, String>`を返すのが一般的です。

```rust
#[tauri::command]
async fn add_feed(url: String, state: State<'_, AppState>) -> Result<Feed, String> {
    // anyhow::Errorを文字列に変換
    let feed = fetch_and_parse(&url)
        .await
        .map_err(|e| format!("フィードの取得に失敗: {}", e))?;

    let db = state.db.lock()
        .map_err(|_| "データベースのロックに失敗".to_string())?;

    db.add_feed(&url, &feed.title)
        .map_err(|e| format!("保存に失敗: {}", e))
}
```

### TypeScript側

```typescript
import { getFeeds, addFeed } from '../api/commands';

async function handleAddFeed(url: string) {
  try {
    const feed = await addFeed(url);
    console.log('追加成功:', feed);
    return feed;
  } catch (error) {
    // errorはstring型（Rust側でStringに変換したもの）
    console.error('追加失敗:', error);
    throw error;
  }
}
```

## 非同期Command

重い処理は`async`で非同期にします。

```rust
// 同期Command（軽い処理向け）
#[tauri::command]
fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// 非同期Command（I/O処理向け）
#[tauri::command]
async fn fetch_feed(id: i64, state: State<'_, AppState>) -> Result<Vec<Article>, String> {
    // ネットワーク通信は非同期で
    let articles = rustfeed_core::feed::fetch_feed_articles(id)
        .await
        .map_err(|e| e.to_string())?;

    // データベース保存
    let db = state.db.lock().map_err(|_| "Lock failed")?;
    db.save_articles(&articles).map_err(|e| e.to_string())?;

    Ok(articles)
}
```

## 引数の命名規則

Rust側とTypeScript側で引数名を一致させる必要があります。

```rust
// Rust: snake_case
#[tauri::command]
async fn get_articles(feed_id: Option<i64>, unread_only: bool) -> Result<...>
```

```typescript
// TypeScript: Rustと同じsnake_caseを使用
await invoke('get_articles', {
  feed_id: 1,      // ✓ Rustと一致
  unread_only: true // ✓ Rustと一致
});

// camelCaseは使わない
await invoke('get_articles', {
  feedId: 1,       // ✗ 動作しない
  unreadOnly: true // ✗ 動作しない
});
```

## 理解度確認

1. `#[tauri::command]`属性の役割は何ですか？
2. フロントエンドからCommandを呼び出す際、引数名はどのようなケースで指定しますか？
3. 非同期Commandを使うべき場面はどのような時ですか？

## 練習問題

1. 記事のコンテンツを取得する`get_article_content(id: i64)`コマンドを実装してください
2. 対応するTypeScript型定義とラッパー関数を作成してください

## 次のステップ

[03-state-management.md](./03-state-management.md)で、Tauriの状態管理について学びましょう。
