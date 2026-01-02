# 04. イベントシステム

このセクションでは、バックエンドからフロントエンドへ通知を送る「Events」について学びます。

## Eventsとは

Commandsがフロントエンド→バックエンドの通信なのに対し、Eventsはバックエンド→フロントエンドの通信です。

```
Commands (同期的):
  フロントエンド ──invoke()──► バックエンド ──Result──► フロントエンド

Events (非同期的):
  バックエンド ──emit()──► フロントエンド (いつでも)
```

## 基本的な使い方

### バックエンド（Rust）

```rust
use tauri::{AppHandle, Emitter};
use serde::Serialize;

#[derive(Clone, Serialize)]
struct FeedUpdatedPayload {
    feed_id: i64,
    new_article_count: usize,
}

#[tauri::command]
async fn fetch_feed(
    id: i64,
    app: AppHandle,  // AppHandleでイベントを発火
    state: State<'_, AppState>,
) -> Result<Vec<Article>, String> {
    let articles = do_fetch(id).await?;
    let count = articles.len();

    // イベントを発火
    app.emit("feed-updated", FeedUpdatedPayload {
        feed_id: id,
        new_article_count: count,
    }).map_err(|e| e.to_string())?;

    Ok(articles)
}
```

### フロントエンド（TypeScript）

```typescript
import { listen } from '@tauri-apps/api/event';

interface FeedUpdatedPayload {
  feed_id: number;
  new_article_count: number;
}

// イベントをリッスン
const unlisten = await listen<FeedUpdatedPayload>('feed-updated', (event) => {
  console.log(`Feed ${event.payload.feed_id} updated with ${event.payload.new_article_count} new articles`);
  // UIを更新
  refreshArticles();
});

// リスナーの解除（コンポーネントのクリーンアップ時）
unlisten();
```

## Reactでの使用

### useEffectでのリスナー登録

```tsx
// apps/rustfeed-gui/src/hooks/useFeedEvents.ts
import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface FeedUpdatedPayload {
  feed_id: number;
  new_article_count: number;
}

export function useFeedEvents(onUpdate: (payload: FeedUpdatedPayload) => void) {
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    // リスナーを登録
    listen<FeedUpdatedPayload>('feed-updated', (event) => {
      onUpdate(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });

    // クリーンアップ
    return () => {
      unlisten?.();
    };
  }, [onUpdate]);
}
```

### コンポーネントでの使用

```tsx
// apps/rustfeed-gui/src/components/ArticleList.tsx
import { useCallback } from 'react';
import { useFeedEvents } from '../hooks/useFeedEvents';
import { useArticles } from '../hooks/useArticles';

export function ArticleList({ feedId }: { feedId: number }) {
  const { articles, refresh } = useArticles(feedId);

  // フィード更新イベントを受け取ったらリフレッシュ
  const handleFeedUpdated = useCallback((payload) => {
    if (payload.feed_id === feedId) {
      refresh();
    }
  }, [feedId, refresh]);

  useFeedEvents(handleFeedUpdated);

  return (
    <ul>
      {articles.map(article => (
        <ArticleItem key={article.id} article={article} />
      ))}
    </ul>
  );
}
```

## rustfeed-tauriでの実践例

### 進捗イベント

フィード更新の進捗をリアルタイムで通知する例：

```rust
// crates/rustfeed-tauri/src/lib.rs
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct FetchProgressPayload {
    pub current: usize,
    pub total: usize,
    pub feed_title: String,
    pub status: String,  // "fetching", "done", "error"
}

#[derive(Clone, Serialize)]
pub struct FetchCompletePayload {
    pub total_feeds: usize,
    pub new_articles: usize,
    pub errors: Vec<String>,
}

#[tauri::command]
async fn fetch_all_feeds(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<FetchCompletePayload, String> {
    let db = state.db.lock().map_err(|_| "Lock failed")?;
    let feeds = db.get_all_feeds().map_err(|e| e.to_string())?;
    drop(db);  // ロックを早めに解放

    let total = feeds.len();
    let mut new_articles = 0;
    let mut errors = Vec::new();

    for (i, feed) in feeds.iter().enumerate() {
        // 進捗を通知
        app.emit("fetch-progress", FetchProgressPayload {
            current: i + 1,
            total,
            feed_title: feed.title.clone(),
            status: "fetching".to_string(),
        }).ok();

        // フィードを取得
        match fetch_single_feed(feed.id, &state).await {
            Ok(count) => {
                new_articles += count;
                app.emit("fetch-progress", FetchProgressPayload {
                    current: i + 1,
                    total,
                    feed_title: feed.title.clone(),
                    status: "done".to_string(),
                }).ok();
            }
            Err(e) => {
                errors.push(format!("{}: {}", feed.title, e));
                app.emit("fetch-progress", FetchProgressPayload {
                    current: i + 1,
                    total,
                    feed_title: feed.title.clone(),
                    status: "error".to_string(),
                }).ok();
            }
        }
    }

    // 完了を通知
    let result = FetchCompletePayload {
        total_feeds: total,
        new_articles,
        errors: errors.clone(),
    };
    app.emit("fetch-complete", result.clone()).ok();

    Ok(result)
}
```

### フロントエンドでの進捗表示

```tsx
// apps/rustfeed-gui/src/components/FetchProgress.tsx
import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface FetchProgress {
  current: number;
  total: number;
  feed_title: string;
  status: 'fetching' | 'done' | 'error';
}

export function FetchProgress() {
  const [progress, setProgress] = useState<FetchProgress | null>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenComplete: (() => void) | undefined;

    listen<FetchProgress>('fetch-progress', (event) => {
      setProgress(event.payload);
      setVisible(true);
    }).then(fn => { unlistenProgress = fn; });

    listen('fetch-complete', () => {
      setTimeout(() => setVisible(false), 2000);
    }).then(fn => { unlistenComplete = fn; });

    return () => {
      unlistenProgress?.();
      unlistenComplete?.();
    };
  }, []);

  if (!visible || !progress) return null;

  const percentage = Math.round((progress.current / progress.total) * 100);

  return (
    <div className="fixed bottom-4 right-4 bg-white shadow-lg rounded-lg p-4 w-80">
      <div className="flex justify-between mb-2">
        <span className="text-sm font-medium">Fetching feeds...</span>
        <span className="text-sm text-gray-500">{progress.current}/{progress.total}</span>
      </div>
      <div className="w-full bg-gray-200 rounded-full h-2">
        <div
          className="bg-blue-600 h-2 rounded-full transition-all"
          style={{ width: `${percentage}%` }}
        />
      </div>
      <div className="mt-2 text-sm text-gray-600 truncate">
        {progress.feed_title}
        {progress.status === 'error' && <span className="text-red-500 ml-2">Error</span>}
      </div>
    </div>
  );
}
```

## ウィンドウ固有のイベント

特定のウィンドウにのみイベントを送ることもできます。

```rust
use tauri::Manager;

// 全ウィンドウに送信
app.emit("global-event", payload)?;

// 特定のウィンドウに送信
if let Some(window) = app.get_webview_window("main") {
    window.emit("window-event", payload)?;
}
```

## フロントエンドからのイベント発火

フロントエンドからもイベントを発火できます（他のウィンドウやバックエンドへ通知）。

```typescript
import { emit } from '@tauri-apps/api/event';

// イベントを発火
await emit('user-action', { action: 'refresh-requested' });
```

```rust
// バックエンドでリッスン（アプリ初期化時）
app.listen("user-action", |event| {
    println!("User action: {:?}", event.payload());
});
```

## Commands vs Events

| 観点 | Commands | Events |
|------|----------|--------|
| 方向 | フロントエンド → バックエンド | バックエンド → フロントエンド |
| 戻り値 | あり (Result) | なし |
| タイミング | 呼び出し時 | いつでも |
| 用途 | データ取得、操作実行 | 進捗通知、状態変更通知 |

## 理解度確認

1. CommandsとEventsの違いは何ですか？
2. `app.emit()`はどのような場面で使用しますか？
3. Reactコンポーネントでイベントリスナーを使う際の注意点は？

## 次のステップ

[05-security.md](./05-security.md)で、Tauri 2.0のセキュリティモデル（Capabilities）について学びましょう。
