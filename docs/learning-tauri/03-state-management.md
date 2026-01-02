# 03. 状態管理

このセクションでは、Tauriバックエンドでの状態管理と、フロントエンドとの連携について学びます。

## Tauriの状態管理

Tauriでは、アプリケーション全体で共有する状態を`manage()`メソッドで登録します。

```rust
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
    pub config: Mutex<AppConfig>,
}

fn main() {
    let state = AppState {
        db: Mutex::new(Database::open().unwrap()),
        config: Mutex::new(AppConfig::load().unwrap()),
    };

    tauri::Builder::default()
        .manage(state)  // 状態を登録
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .unwrap();
}
```

## State<'_, T>の使い方

Commandで状態にアクセスするには`State<'_, T>`を引数に追加します。

```rust
use tauri::State;

#[tauri::command]
async fn get_feeds(state: State<'_, AppState>) -> Result<Vec<Feed>, String> {
    // Mutexをロックしてデータベースにアクセス
    let db = state.db.lock().map_err(|_| "Lock failed")?;
    db.get_all_feeds().map_err(|e| e.to_string())
}
```

### なぜMutexが必要か

Tauriは複数のスレッドからCommandを呼び出す可能性があるため、共有状態には排他制御が必要です。

```rust
// ✗ コンパイルエラー: Database is not Sync
pub struct AppState {
    pub db: Database,  // スレッドセーフでない
}

// ✓ 正しい実装
pub struct AppState {
    pub db: Mutex<Database>,  // Mutexで保護
}
```

## 複数の状態

複数の状態を管理する場合は、それぞれ別に登録できます。

```rust
pub struct DatabaseState {
    pub db: Mutex<Database>,
}

pub struct ConfigState {
    pub config: Mutex<AppConfig>,
}

fn main() {
    tauri::Builder::default()
        .manage(DatabaseState { db: Mutex::new(Database::open().unwrap()) })
        .manage(ConfigState { config: Mutex::new(AppConfig::load().unwrap()) })
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .unwrap();
}

// Commandで個別にアクセス
#[tauri::command]
async fn get_feeds(db: State<'_, DatabaseState>) -> Result<Vec<Feed>, String> {
    let db = db.db.lock().map_err(|_| "Lock failed")?;
    db.get_all_feeds().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config(config: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let config = config.config.lock().map_err(|_| "Lock failed")?;
    Ok(config.clone())
}
```

## 非同期ロック（tokio::sync::Mutex）

長時間ロックが必要な場合は、`tokio::sync::Mutex`を使うとブロッキングを避けられます。

```rust
use tokio::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
}

#[tauri::command]
async fn fetch_all_feeds(state: State<'_, AppState>) -> Result<FetchResult, String> {
    // .await でロックを取得（他のタスクをブロックしない）
    let db = state.db.lock().await;

    // 時間のかかる処理
    let result = perform_fetch_all(&db).await?;
    Ok(result)
}
```

## rustfeed-tauriでの実践

### 状態の定義

```rust
// crates/rustfeed-tauri/src/lib.rs
use rustfeed_core::db::Database;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
}

impl AppState {
    pub fn new() -> Result<Self, anyhow::Error> {
        let db = Database::open()?;
        Ok(Self { db: Mutex::new(db) })
    }
}
```

### 初期化

```rust
// crates/rustfeed-tauri/src/main.rs
use crate::AppState;

fn main() {
    let state = AppState::new().expect("Failed to initialize app state");

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_feeds,
            add_feed,
            delete_feed,
            get_articles,
            toggle_read,
            toggle_favorite,
            fetch_feed,
            fetch_all_feeds,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## フロントエンドの状態管理

フロントエンド側では、Reactの状態管理（useState, useReducer, Zustand等）を使用します。

### useState + カスタムフック

```typescript
// apps/rustfeed-gui/src/hooks/useFeeds.ts
import { useState, useEffect, useCallback } from 'react';
import { getFeeds, addFeed as addFeedApi, deleteFeed as deleteFeedApi } from '../api/commands';
import type { Feed } from '../types';

export function useFeeds() {
  const [feeds, setFeeds] = useState<Feed[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await getFeeds();
      setFeeds(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const addFeed = useCallback(async (url: string) => {
    const feed = await addFeedApi(url);
    setFeeds(prev => [...prev, feed]);
    return feed;
  }, []);

  const deleteFeed = useCallback(async (id: number) => {
    await deleteFeedApi(id);
    setFeeds(prev => prev.filter(f => f.id !== id));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { feeds, loading, error, refresh, addFeed, deleteFeed };
}
```

### コンポーネントでの使用

```tsx
// apps/rustfeed-gui/src/components/FeedList/FeedList.tsx
import { useFeeds } from '../../hooks/useFeeds';

export function FeedList() {
  const { feeds, loading, error, addFeed, deleteFeed } = useFeeds();

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div>
      {feeds.map(feed => (
        <FeedItem
          key={feed.id}
          feed={feed}
          onDelete={() => deleteFeed(feed.id)}
        />
      ))}
    </div>
  );
}
```

### Zustandを使う場合

より複雑な状態管理にはZustandが便利です。

```typescript
// apps/rustfeed-gui/src/stores/feedStore.ts
import { create } from 'zustand';
import { getFeeds, addFeed, deleteFeed } from '../api/commands';
import type { Feed } from '../types';

interface FeedState {
  feeds: Feed[];
  selectedFeedId: number | null;
  loading: boolean;
  error: string | null;

  fetchFeeds: () => Promise<void>;
  addFeed: (url: string) => Promise<Feed>;
  deleteFeed: (id: number) => Promise<void>;
  selectFeed: (id: number | null) => void;
}

export const useFeedStore = create<FeedState>((set) => ({
  feeds: [],
  selectedFeedId: null,
  loading: false,
  error: null,

  fetchFeeds: async () => {
    set({ loading: true, error: null });
    try {
      const feeds = await getFeeds();
      set({ feeds, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addFeed: async (url: string) => {
    const feed = await addFeed(url);
    set((state) => ({ feeds: [...state.feeds, feed] }));
    return feed;
  },

  deleteFeed: async (id: number) => {
    await deleteFeed(id);
    set((state) => ({
      feeds: state.feeds.filter(f => f.id !== id),
      selectedFeedId: state.selectedFeedId === id ? null : state.selectedFeedId,
    }));
  },

  selectFeed: (id) => set({ selectedFeedId: id }),
}));
```

## Tauri vs Electron の状態管理比較

| 観点 | Tauri | Electron |
|------|-------|----------|
| バックエンド状態 | Rust (Mutex/RwLock) | Node.js (グローバル変数) |
| スレッドセーフ | 明示的なロック必要 | シングルスレッドなので不要 |
| フロントエンド | React等（自由に選択） | React等（自由に選択） |
| IPC | `invoke()`/`emit()` | `ipcRenderer`/`ipcMain` |

## 理解度確認

1. なぜTauriの状態にはMutexが必要ですか？
2. `State<'_, T>`はどのように使いますか？
3. フロントエンドとバックエンドの状態を同期するベストプラクティスは？

## 次のステップ

[04-events.md](./04-events.md)で、バックエンドからフロントエンドへの通知（Events）について学びましょう。
