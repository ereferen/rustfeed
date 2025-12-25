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
    /// rustfeed list
    /// ```
    List,

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
    /// rustfeed articles           # 全記事
    /// rustfeed articles --unread  # 未読のみ
    /// rustfeed articles -l 10     # 10件まで
    /// ```
    Articles {
        /// 未読記事のみを表示するフラグ
        #[arg(short, long)]
        unread: bool,

        /// 表示する記事数の上限（デフォルト: 20）
        #[arg(short, long, default_value = "20")]
        limit: usize,
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

        Commands::List => {
            commands::list_feeds(&db)?;
        }

        Commands::Fetch => {
            // .await: 非同期処理の完了を待つ
            commands::fetch_feeds(&db).await?;
        }

        Commands::Articles { unread, limit } => {
            commands::show_articles(&db, unread, limit)?;
        }

        Commands::Read { id } => {
            commands::mark_as_read(&db, id)?;
        }
    }

    // 成功を示す Ok(()) を返す
    // () は「ユニット型」で、意味のある値がないことを表す
    Ok(())
}
