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
//!
//! ## アーキテクチャ
//!
//! このアプリケーションは以下のモジュールで構成されています:
//!
//! - [`commands`] - CLIコマンドの実装
//! - [`db`] - SQLiteデータベース操作
//! - [`feed`] - RSS/Atomフィードの取得・パース
//! - [`models`] - データモデル（Feed, Article）

// =============================================================================
// モジュール宣言
// =============================================================================
// Rustでは `mod` キーワードでサブモジュールを宣言します。
// これにより、同名のファイル（例: commands/mod.rs）が読み込まれます。

mod commands;
mod config;
mod db;
mod feed;
mod models;

// =============================================================================
// 外部クレートのインポート
// =============================================================================
// `use` キーワードで外部クレートの型や関数をスコープに持ち込みます。

use anyhow::Result; // エラーハンドリング用。Result<T> = Result<T, anyhow::Error>
use clap::{Parser, Subcommand}; // CLIパーサー。derive マクロで自動生成

// =============================================================================
// CLI構造体の定義
// =============================================================================

/// CLIのルート構造体
///
/// `#[derive(Parser)]` マクロにより、clap が自動的に
/// コマンドライン引数のパース処理を生成します。
///
/// # Derive マクロについて
///
/// Rust の derive マクロは、構造体や列挙型に対して
/// トレイト（インターフェース）の実装を自動生成する機能です。
/// `Parser` トレイトを derive することで、この構造体を
/// コマンドライン引数から自動的に構築できるようになります。
#[derive(Parser)]
#[command(name = "rustfeed")] // プログラム名
#[command(author = "ereferen")] // 作者
#[command(version = "0.1.0")] // バージョン
#[command(about = "A CLI RSS reader written in Rust", long_about = None)]
struct Cli {
    /// サブコマンドを格納するフィールド
    ///
    /// `#[command(subcommand)]` 属性により、このフィールドが
    /// サブコマンド（add, list, fetch など）を受け取ることを示します。
    #[command(subcommand)]
    command: Commands,
}

/// 利用可能なサブコマンドの列挙型
///
/// Rust の `enum`（列挙型）は、複数のバリアント（選択肢）を持つことができ、
/// 各バリアントは異なるデータを持つことができます。
/// これは「代数的データ型」と呼ばれ、Rust の強力な機能の一つです。
///
/// # パターンマッチング
///
/// `match` 式を使って、どのバリアントかを判定し、
/// 適切な処理を行うことができます（main関数内を参照）。
#[derive(Subcommand)]
enum Commands {
    /// 新しい RSS フィードを追加する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed add https://example.com/feed.xml
    /// rustfeed add https://example.com/feed.xml --name "My Feed"
    /// ```
    Add {
        /// RSS フィードの URL
        url: String,

        /// フィードのカスタム名（省略可能）
        ///
        /// `#[arg(short, long)]` により、`-n` または `--name` で指定できます。
        /// `Option<String>` は、値が存在しない可能性を型で表現しています。
        #[arg(short, long)]
        name: Option<String>,
    },

    /// RSS フィードを削除する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed remove 1
    /// ```
    Remove {
        /// 削除するフィードの ID
        id: i64,
    },

    /// 登録済みの全フィードを一覧表示する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed list                        # 全フィード
    /// rustfeed list --category "Tech"     # Techカテゴリのみ
    /// ```
    List {
        /// カテゴリでフィルタリング（オプション）
        #[arg(long)]
        category: Option<String>,
    },

    /// 全フィードから新しい記事を取得する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed fetch
    /// ```
    Fetch,

    /// 記事を一覧表示する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed articles                    # 全記事
    /// rustfeed articles --unread           # 未読のみ
    /// rustfeed articles -l 10              # 10件まで
    /// rustfeed articles --filter "rust"    # キーワードでフィルタ
    /// rustfeed articles --feed 2           # フィードID 2の記事のみ
    /// rustfeed articles --after "2025-01-01"  # 2025年1月1日以降
    /// rustfeed articles --before "2025-12-31" # 2025年12月31日以前
    /// rustfeed articles --last-days 7      # 過去7日間
    /// rustfeed articles --last-weeks 2     # 過去2週間
    /// rustfeed articles --filter "rust,cargo" --unread --feed 2 --last-days 7  # 複合フィルタ
    /// ```
    Articles {
        /// 未読記事のみを表示するフラグ
        #[arg(short, long)]
        unread: bool,

        /// 表示する記事数の上限（指定しない場合は設定ファイルのデフォルト値）
        #[arg(short, long)]
        limit: Option<usize>,

        /// キーワードフィルタ（カンマ区切りで複数指定可能、OR条件）
        ///
        /// タイトルまたは本文に指定したキーワードを含む記事のみを表示します。
        /// 複数のキーワードをカンマで区切って指定すると、いずれかを含む記事を表示します。
        #[arg(short, long)]
        filter: Option<String>,

        /// 特定のフィードIDの記事のみを表示
        ///
        /// 指定したフィードIDの記事のみを表示します。
        /// フィードIDは `rustfeed list` コマンドで確認できます。
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
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed search "rust async"                 # 基本検索
    /// rustfeed search "rust" --unread -l 10        # 未読のみ10件
    /// rustfeed search "rust" --after "2025-01-01"  # 日付フィルタ併用
    /// ```
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
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed read 1
    /// ```
    Read {
        /// 既読にする記事の ID
        id: i64,
    },

    /// 記事をお気に入りに追加する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed favorite 1
    /// ```
    Favorite {
        /// お気に入りにする記事の ID
        id: i64,
    },

    /// 記事をお気に入りから削除する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed unfavorite 1
    /// ```
    Unfavorite {
        /// お気に入りから削除する記事の ID
        id: i64,
    },

    /// お気に入り記事を一覧表示する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed favorites
    /// rustfeed favorites -l 10  # 10件まで
    /// ```
    Favorites {
        /// 表示する記事数の上限（デフォルト: 20）
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// 記事をエクスポートする
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed export                        # JSON形式でエクスポート
    /// rustfeed export --format markdown      # Markdown形式でエクスポート
    /// rustfeed export --favorites            # お気に入りのみエクスポート
    /// rustfeed export --unread -l 50         # 未読50件をエクスポート
    /// rustfeed export -f json --favorites > articles.json
    /// ```
    Export {
        /// エクスポート形式（json または markdown）
        ///
        /// `json`: JSON形式で出力（機械可読、他のツールとの連携に適している）
        /// `markdown`: Markdown形式で出力（人間可読、ドキュメントとして利用可能）
        #[arg(short, long, default_value = "json")]
        format: String,

        /// お気に入り記事のみエクスポートするフラグ
        #[arg(long)]
        favorites: bool,

        /// 未読記事のみエクスポートするフラグ
        #[arg(long)]
        unread: bool,

        /// エクスポートする記事数の上限（オプション）
        ///
        /// 指定しない場合は全記事をエクスポートします。
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// 記事を一括で既読にする
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed mark-all-read                # 全記事を既読にする
    /// rustfeed mark-all-read --feed 2       # フィード2の記事を既読にする
    /// rustfeed mark-all-read --before "2025-01-01"  # 指定日付以前を既読にする
    /// ```
    MarkAllRead {
        /// 特定のフィードIDの記事のみを対象
        #[arg(long)]
        feed: Option<i64>,

        /// 指定日時以前の記事を既読にする（YYYY-MM-DD形式）
        #[arg(long)]
        before: Option<String>,
    },

    /// 記事を未読に戻す
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed mark-unread 123              # 記事ID 123を未読にする
    /// rustfeed mark-unread --feed 2         # フィード2の全記事を未読にする
    /// rustfeed mark-unread --all            # 全記事を未読にする
    /// ```
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
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed toggle-read 123              # 記事ID 123の状態を反転
    /// ```
    ToggleRead {
        /// 反転する記事のID
        id: i64,
    },

    /// フィードの名前を変更する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed rename 1 "My Tech Blog"       # フィード1の名前を変更
    /// rustfeed rename 1 ""                   # カスタム名をクリア（元のタイトルに戻す）
    /// ```
    Rename {
        /// 変更するフィードのID
        id: i64,

        /// 新しい名前（空文字列の場合はカスタム名をクリア）
        name: String,
    },

    /// フィードのURLを更新する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed update-url 1 https://example.com/new-feed.xml
    /// ```
    UpdateUrl {
        /// 更新するフィードのID
        id: i64,

        /// 新しいURL
        url: String,
    },

    /// フィードのカテゴリを設定する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed set-category 1 "Tech"         # フィード1にTechカテゴリを設定
    /// rustfeed set-category 1 ""             # カテゴリをクリア
    /// ```
    SetCategory {
        /// 設定するフィードのID
        id: i64,

        /// カテゴリ名（空文字列の場合はカテゴリをクリア）
        category: String,
    },

    /// フィードの優先順位を設定する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed set-priority 1 10             # フィード1の優先順位を10に設定
    /// rustfeed set-priority 2 0              # フィード2の優先順位をデフォルトに戻す
    /// ```
    SetPriority {
        /// 設定するフィードのID
        id: i64,

        /// 優先順位（高いほど優先、デフォルト0）
        priority: i64,
    },

    /// フィードの詳細情報を表示する
    ///
    /// # 使用例
    /// ```bash
    /// rustfeed info 1                        # フィード1の詳細を表示
    /// ```
    Info {
        /// 表示するフィードのID
        id: i64,
    },
}

// =============================================================================
// メイン関数
// =============================================================================

/// アプリケーションのエントリーポイント
///
/// # 非同期処理について
///
/// `#[tokio::main]` マクロは、この関数を非同期ランタイム上で実行します。
/// `async fn` は非同期関数を定義し、内部で `.await` を使って
/// 非同期処理の完了を待つことができます。
///
/// # エラーハンドリング
///
/// `Result<()>` を返すことで、`?` 演算子を使ったエラー伝播が可能になります。
/// エラーが発生した場合、自動的にエラーメッセージが表示されます。
#[tokio::main]
async fn main() -> Result<()> {
    // コマンドライン引数をパースして Cli 構造体を構築
    // parse() は clap の Parser トレイトで定義されたメソッド
    let cli = Cli::parse();

    // データベース接続を初期化
    // `?` 演算子: Result が Err の場合、即座に関数から return する
    let db = db::Database::new()?;

    // データベーステーブルを初期化（存在しない場合は作成）
    db.init()?;

    // 設定ファイルを読み込む
    // ファイルが存在しない場合はデフォルト値を使用
    let config = config::AppConfig::load()?;

    // パターンマッチングでサブコマンドを処理
    // Rust の match は網羅的（exhaustive）: 全てのケースを処理する必要がある
    match cli.command {
        // Add コマンド: 構造体の分解束縛（destructuring）を使用
        Commands::Add { url, name } => {
            // name.as_deref(): Option<String> を Option<&str> に変換
            // これにより、所有権を移動せずに参照を渡せる
            commands::add_feed(&db, &url, name.as_deref()).await?;
        }

        Commands::Remove { id } => {
            commands::remove_feed(&db, id)?;
        }

        Commands::List { category } => {
            commands::list_feeds(&db, category.as_deref())?;
        }

        Commands::Fetch => {
            // .await: 非同期処理の完了を待つ
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

            // limitが指定されていない場合、設定ファイルのデフォルト値を使用
            let limit_val = limit.unwrap_or(config.general.default_limit);

            // disabled_feedsを適用
            // ユーザーが特定のフィードを指定している場合は、disabled_feedsを無視
            let final_feed = if feed.is_some() {
                feed
            } else if !config.general.disabled_feeds.is_empty() {
                // disabled_feedsが設定されている場合、それらを除外するために
                // show_articles関数にdisabled_feedsを渡す
                // ただし、現在の実装ではfeedは単一のi64なので、
                // disabled_feedsを適用するにはshow_articles関数を変更する必要がある
                // 今はシンプルに、feedがNoneの場合のみdisabled_feedsを考慮する
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
                &[], // searchコマンドではdisabled_feedsを無視
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

    // 成功を示す Ok(()) を返す
    // () は「ユニット型」で、意味のある値がないことを表す
    Ok(())
}
