//! # rustfeed - CLI RSS Reader
//!
//! `rustfeed` は Rust で書かれたコマンドライン RSS リーダーです。
//!
//! ## 主な機能
//!
//! - RSS/Atom フィードの登録・管理
//! - 記事の取得・一覧表示
//! - 既読/未読の管理
//!
//! ## 使用例
//!
//! ```bash
//! # フィードを追加
//! rustfeed add https://blog.rust-lang.org/feed.xml
//!
//! # 登録済みフィード一覧
//! rustfeed list
//!
//! # 記事を取得
//! rustfeed fetch
//!
//! # 未読記事を表示
//! rustfeed articles --unread
//! ```

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use rustfeed_core::{config::AppConfig, db::Database};

// =============================================================================
// CLI構造体の定義
// =============================================================================

/// CLIのルート構造体
#[derive(Parser)]
#[command(name = "rustfeed")]
#[command(author = "ereferen")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A CLI RSS reader written in Rust", long_about = None)]
struct Cli {
    /// サブコマンドを格納するフィールド
    #[command(subcommand)]
    command: Commands,
}

/// 利用可能なサブコマンドの列挙型
#[derive(Subcommand)]
enum Commands {
    /// 新しい RSS フィードを追加する
    Add {
        /// RSS フィードの URL
        url: String,

        /// フィードのカスタム名（省略可能）
        #[arg(short, long)]
        name: Option<String>,
    },

    /// RSS フィードを削除する
    Remove {
        /// 削除するフィードの ID
        id: i64,
    },

    /// 登録済みの全フィードを一覧表示する
    List {
        /// カテゴリでフィルタリング（オプション）
        #[arg(long)]
        category: Option<String>,
    },

    /// 全フィードから新しい記事を取得する
    Fetch,

    /// 記事を一覧表示する
    Articles {
        /// 未読記事のみを表示するフラグ
        #[arg(short, long)]
        unread: bool,

        /// 表示する記事数の上限（指定しない場合は設定ファイルのデフォルト値）
        #[arg(short, long)]
        limit: Option<usize>,

        /// キーワードフィルタ（カンマ区切りで複数指定可能、OR条件）
        #[arg(short, long)]
        filter: Option<String>,

        /// 特定のフィードIDの記事のみを表示
        #[arg(long)]
        feed: Option<i64>,

        /// 指定日時以降の記事のみを表示（YYYY-MM-DD形式）
        #[arg(long)]
        after: Option<String>,

        /// 指定日時以前の記事のみを表示（YYYY-MM-DD形式）
        #[arg(long)]
        before: Option<String>,

        /// 過去N日間の記事のみを表示
        #[arg(long, conflicts_with_all = ["after", "before"])]
        last_days: Option<u32>,

        /// 過去N週間の記事のみを表示
        #[arg(long, conflicts_with_all = ["after", "before", "last_days"])]
        last_weeks: Option<u32>,
    },

    /// 記事を全文検索する
    Search {
        /// 検索クエリ
        query: String,

        /// 未読記事のみを検索
        #[arg(short, long)]
        unread: bool,

        /// 検索結果の上限（デフォルト: 20）
        #[arg(short, long)]
        limit: Option<usize>,

        /// 特定のフィードIDのみ検索
        #[arg(long)]
        feed: Option<i64>,

        /// 指定日時以降の記事のみ検索
        #[arg(long)]
        after: Option<String>,

        /// 指定日時以前の記事のみ検索
        #[arg(long)]
        before: Option<String>,
    },

    /// 記事を既読としてマークする
    Read {
        /// 既読にする記事の ID
        id: i64,
    },

    /// 記事をお気に入りに追加する
    Favorite {
        /// お気に入りにする記事の ID
        id: i64,
    },

    /// 記事をお気に入りから削除する
    Unfavorite {
        /// お気に入りから削除する記事の ID
        id: i64,
    },

    /// お気に入り記事を一覧表示する
    Favorites {
        /// 表示する記事数の上限（デフォルト: 20）
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// 記事をエクスポートする
    Export {
        /// エクスポート形式（json または markdown）
        #[arg(short, long, default_value = "json")]
        format: String,

        /// お気に入り記事のみエクスポートするフラグ
        #[arg(long)]
        favorites: bool,

        /// 未読記事のみエクスポートするフラグ
        #[arg(long)]
        unread: bool,

        /// エクスポートする記事数の上限（オプション）
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// 記事を一括で既読にする
    MarkAllRead {
        /// 特定のフィードIDの記事のみを対象
        #[arg(long)]
        feed: Option<i64>,

        /// 指定日時以前の記事を既読にする（YYYY-MM-DD形式）
        #[arg(long)]
        before: Option<String>,
    },

    /// 記事を未読に戻す
    MarkUnread {
        /// 未読にする記事のID（--feedまたは--allと排他）
        id: Option<i64>,

        /// 特定のフィードIDの全記事を未読にする
        #[arg(long)]
        feed: Option<i64>,

        /// 全記事を未読にする
        #[arg(long)]
        all: bool,
    },

    /// 記事の既読/未読状態を反転する
    ToggleRead {
        /// 反転する記事のID
        id: i64,
    },

    /// フィードの名前を変更する
    Rename {
        /// 変更するフィードのID
        id: i64,

        /// 新しい名前（空文字列の場合はカスタム名をクリア）
        name: String,
    },

    /// フィードのURLを更新する
    UpdateUrl {
        /// 更新するフィードのID
        id: i64,

        /// 新しいURL
        url: String,
    },

    /// フィードのカテゴリを設定する
    SetCategory {
        /// 設定するフィードのID
        id: i64,

        /// カテゴリ名（空文字列の場合はカテゴリをクリア）
        category: String,
    },

    /// フィードの優先順位を設定する
    SetPriority {
        /// 設定するフィードのID
        id: i64,

        /// 優先順位（高いほど優先、デフォルト0）
        priority: i64,
    },

    /// フィードの詳細情報を表示する
    Info {
        /// 表示するフィードのID
        id: i64,
    },
}

// =============================================================================
// メイン関数
// =============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // データベース接続を初期化
    let db = Database::new()?;
    db.init()?;

    // 設定ファイルを読み込む
    let config = AppConfig::load()?;

    // パターンマッチングでサブコマンドを処理
    match cli.command {
        Commands::Add { url, name } => {
            commands::add_feed(&db, &url, name.as_deref()).await?;
        }

        Commands::Remove { id } => {
            commands::remove_feed(&db, id)?;
        }

        Commands::List { category } => {
            commands::list_feeds(&db, category.as_deref())?;
        }

        Commands::Fetch => {
            commands::fetch_feeds(&db).await?;
        }

        Commands::Articles {
            unread,
            limit,
            filter,
            feed,
            after,
            before,
            last_days,
            last_weeks,
        } => {
            use chrono::{Duration, Utc};

            let limit_val = limit.unwrap_or(config.general.default_limit);

            let final_feed = if feed.is_some() {
                feed
            } else if !config.general.disabled_feeds.is_empty() {
                feed
            } else {
                feed
            };

            // 日付範囲を計算
            let (after_date, before_date) = if let Some(days) = last_days {
                let after = Utc::now() - Duration::days(days as i64);
                (Some(after.format("%Y-%m-%d").to_string()), None)
            } else if let Some(weeks) = last_weeks {
                let after = Utc::now() - Duration::weeks(weeks as i64);
                (Some(after.format("%Y-%m-%d").to_string()), None)
            } else {
                (after, before)
            };

            commands::show_articles(
                &db,
                unread,
                limit_val,
                filter.as_deref(),
                final_feed,
                &config.general.disabled_feeds,
                after_date.as_deref(),
                before_date.as_deref(),
            )?;
        }

        Commands::Search {
            query,
            unread,
            limit,
            feed,
            after,
            before,
        } => {
            let limit_val = limit.unwrap_or(20);

            commands::show_articles(
                &db,
                unread,
                limit_val,
                Some(&query),
                feed,
                &[],
                after.as_deref(),
                before.as_deref(),
            )?;
        }

        Commands::Read { id } => {
            commands::mark_as_read(&db, id)?;
        }

        Commands::Favorite { id } => {
            commands::add_favorite(&db, id)?;
        }

        Commands::Unfavorite { id } => {
            commands::remove_favorite(&db, id)?;
        }

        Commands::Favorites { limit } => {
            commands::show_favorites(&db, limit)?;
        }

        Commands::Export {
            format,
            favorites,
            unread,
            limit,
        } => {
            commands::export_articles(&db, &format, favorites, unread, limit)?;
        }

        Commands::MarkAllRead { feed, before } => {
            commands::mark_all_read(&db, feed, before.as_deref())?;
        }

        Commands::MarkUnread { id, feed, all } => {
            commands::mark_unread(&db, id, feed, all)?;
        }

        Commands::ToggleRead { id } => {
            commands::toggle_read(&db, id)?;
        }

        Commands::Rename { id, name } => {
            commands::rename_feed(&db, id, &name)?;
        }

        Commands::UpdateUrl { id, url } => {
            commands::update_feed_url(&db, id, &url)?;
        }

        Commands::SetCategory { id, category } => {
            commands::set_feed_category(&db, id, &category)?;
        }

        Commands::SetPriority { id, priority } => {
            commands::set_feed_priority(&db, id, priority)?;
        }

        Commands::Info { id } => {
            commands::show_feed_info(&db, id)?;
        }
    }

    Ok(())
}
