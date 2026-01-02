//! rustfeed GUI - Tauriアプリケーションのエントリポイント
//!
//! このバイナリは、rustfeed GUIのTauriバックエンドを起動します。
//! すべてのTauri Commandsはここで定義されています。

// Windows で release ビルド時にコンソールウィンドウを非表示
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use rustfeed_core::models::{Article, Feed};
use rustfeed_tauri::{AppState, FetchResult};
use tauri::State;

// =============================================================================
// Tauri Commands
// =============================================================================

/// 全フィードを取得
#[tauri::command]
async fn get_feeds(state: State<'_, AppState>) -> Result<Vec<Feed>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_feeds(None).map_err(|e| e.to_string())
}

/// フィードを追加
#[tauri::command]
async fn add_feed(url: String, state: State<'_, AppState>) -> Result<Feed, String> {
    // フィードを取得してパース
    let (feed_info, _articles) = rustfeed_core::feed::fetch_feed(&url)
        .await
        .map_err(|e| format!("フィードの取得に失敗: {}", e))?;

    let db = state.db.lock().map_err(|e| e.to_string())?;

    // データベースに追加
    db.add_feed_simple(&url, &feed_info.title)
        .map_err(|e| e.to_string())
}

/// フィードを削除
#[tauri::command]
async fn delete_feed(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.remove_feed(id).map_err(|e| e.to_string())?;
    Ok(())
}

/// フィードをリネーム
#[tauri::command]
async fn rename_feed(id: i64, title: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.rename_feed(id, Some(&title)).map_err(|e| e.to_string())
}

/// 記事一覧を取得
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

    // フィルタ条件に基づいて記事を取得
    if favorites_only {
        db.get_favorite_articles(limit as usize)
            .map_err(|e| e.to_string())
    } else if let Some(query) = search {
        db.search_articles(&query, limit)
            .map_err(|e| e.to_string())
    } else {
        db.get_articles(unread_only, limit as usize, None, feed_id)
            .map_err(|e| e.to_string())
    }
}

/// 記事を既読にする
#[tauri::command]
async fn mark_as_read(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.mark_as_read(id).map_err(|e| e.to_string())?;
    Ok(())
}

/// 記事を未読にする
#[tauri::command]
async fn mark_as_unread(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.mark_as_unread(id).map_err(|e| e.to_string())?;
    Ok(())
}

/// お気に入りを切り替え
#[tauri::command]
async fn toggle_favorite(id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_favorite(id).map_err(|e| e.to_string())
}

/// 記事のコンテンツを取得
#[tauri::command]
async fn get_article_content(id: i64, state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let article = db
        .get_article(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "記事が見つかりません".to_string())?;
    Ok(article.content.unwrap_or_default())
}

/// 単一フィードを更新
#[tauri::command]
async fn fetch_feed(id: i64, state: State<'_, AppState>) -> Result<usize, String> {
    // フィード情報を取得（スコープでロックを自動解放）
    let feed_url = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let feed = db
            .get_feed(id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "フィードが見つかりません".to_string())?;
        feed.url
    };

    // フィードを取得（ロック解放後にawait）
    let feed_data = rustfeed_core::feed::fetch_feed(&feed_url)
        .await
        .map_err(|e| format!("フィードの取得に失敗: {}", e))?;

    // 記事をデータベースに保存
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let count = rustfeed_core::feed::save_articles(&db, id, &feed_data)
        .map_err(|e| format!("記事の保存に失敗: {}", e))?;

    Ok(count)
}

/// 全フィードを更新
#[tauri::command]
async fn fetch_all_feeds(state: State<'_, AppState>) -> Result<FetchResult, String> {
    // フィード情報を取得（スコープでロックを自動解放）
    let feeds = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_feeds(None).map_err(|e| e.to_string())?
    };

    let total_feeds = feeds.len();
    let mut new_articles = 0;
    let mut errors = Vec::new();

    for feed in feeds {
        // フィードを取得（ロック解放後にawait）
        match rustfeed_core::feed::fetch_feed(&feed.url).await {
            Ok(feed_data) => {
                // 記事を保存（新しいスコープでロック）
                let save_result = {
                    let db = state.db.lock().map_err(|e| e.to_string())?;
                    rustfeed_core::feed::save_articles(&db, feed.id, &feed_data)
                };
                match save_result {
                    Ok(count) => new_articles += count,
                    Err(e) => errors.push(format!("{}: {}", feed.title, e)),
                }
            }
            Err(e) => errors.push(format!("{}: {}", feed.title, e)),
        }
    }

    Ok(FetchResult {
        total_feeds,
        new_articles,
        errors,
    })
}

/// アプリバージョンを取得
#[tauri::command]
fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    let state = AppState::new().expect("Failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_feeds,
            add_feed,
            delete_feed,
            rename_feed,
            get_articles,
            mark_as_read,
            mark_as_unread,
            toggle_favorite,
            get_article_content,
            fetch_feed,
            fetch_all_feeds,
            get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
