//! rustfeed-tauri: Tauri Backend for rustfeed GUI
//!
//! このクレートは、rustfeed-coreの機能をTauriアプリケーションとして公開します。
//! フロントエンド（React）からTauri Commandsを呼び出すことで、
//! フィードと記事の管理が可能になります。

use rustfeed_core::db::Database;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// アプリケーション状態
///
/// データベース接続をMutexで保護し、複数スレッドから安全にアクセスできるようにします。
pub struct AppState {
    pub db: Mutex<Database>,
}

impl AppState {
    /// 新しいAppStateを作成
    ///
    /// データベースを開き、状態として保持します。
    pub fn new() -> anyhow::Result<Self> {
        let db = Database::open()?;
        Ok(Self { db: Mutex::new(db) })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new().expect("Failed to initialize app state")
    }
}

/// フィード更新結果
#[derive(Clone, Serialize, Deserialize)]
pub struct FetchResult {
    pub total_feeds: usize,
    pub new_articles: usize,
    pub errors: Vec<String>,
}
