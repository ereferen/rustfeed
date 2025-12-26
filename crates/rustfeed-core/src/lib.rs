//! # rustfeed-core
//!
//! rustfeed アプリケーションの共有ライブラリです。
//!
//! このクレートは、CLI と TUI の両方から使用される
//! コア機能を提供します:
//!
//! - **models**: データモデル（Feed, Article）
//! - **db**: データベース操作
//! - **feed**: RSS/Atom フィード取得・パース
//! - **config**: 設定ファイル管理
//!
//! ## 使用例
//!
//! ```rust,no_run
//! use rustfeed_core::{db::Database, feed::fetch_feed, config::AppConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 設定を読み込む
//!     let config = AppConfig::load()?;
//!
//!     // データベースを初期化
//!     let db = Database::new()?;
//!     db.init()?;
//!
//!     // フィードを取得
//!     let (feed, articles) = fetch_feed("https://example.com/feed.xml").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod db;
pub mod feed;
pub mod models;

// 便利な再エクスポート
pub use config::AppConfig;
pub use db::Database;
pub use models::{Article, Feed};
