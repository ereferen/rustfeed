//! # CLIコマンド実装モジュール
//!
//! このモジュールは、rustfeed の各サブコマンドの実装を提供します。
//!
//! ## 提供する関数
//!
//! | 関数 | 対応コマンド | 説明 |
//! |------|-------------|------|
//! | [`add_feed`] | `rustfeed add` | フィード追加 |
//! | [`remove_feed`] | `rustfeed remove` | フィード削除 |
//! | [`list_feeds`] | `rustfeed list` | フィード一覧 |
//! | [`fetch_feeds`] | `rustfeed fetch` | 記事取得 |
//! | [`show_articles`] | `rustfeed articles` | 記事一覧 |
//! | [`mark_as_read`] | `rustfeed read` | 既読マーク |
//!
//! ## 設計方針
//!
//! - 各関数は `Database` への参照を受け取る（依存性注入）
//! - 標準出力にカラー付きのメッセージを表示
//! - エラーは `Result` で返し、呼び出し元に処理を委ねる

use anyhow::{Context, Result};
use colored::Colorize;

use crate::db::Database;
use crate::feed;
use crate::models::Article;

// =============================================================================
// フィード管理コマンド
// =============================================================================

/// 新しいRSSフィードを追加する
///
/// # 処理の流れ
///
/// 1. URLからフィードを取得・パース
/// 2. カスタム名があれば適用
/// 3. データベースに保存
/// 4. 成功メッセージを表示
///
/// # 引数
///
/// * `db` - データベース接続への参照
/// * `url` - 追加するフィードのURL
/// * `name` - カスタム名（省略可能）
///
/// # `Option<&str>` について
///
/// `Option<&str>` は「文字列スライスへの参照があるかもしれない」を表します。
/// - `Some("name")` - 名前が指定された
/// - `None` - 名前は指定されていない
///
/// `&str` は `String` への参照で、所有権を持ちません。
/// これにより、呼び出し元の文字列を借用するだけで済みます。
///
/// # 例
///
/// ```rust,no_run
/// # use rustfeed::db::Database;
/// # use rustfeed::commands::add_feed;
/// # async fn example() -> anyhow::Result<()> {
/// # let db = Database::new()?;
/// // カスタム名なし
/// add_feed(&db, "https://example.com/feed.xml", None).await?;
///
/// // カスタム名あり
/// add_feed(&db, "https://example.com/feed.xml", Some("My Feed")).await?;
/// # Ok(())
/// # }
/// ```
pub async fn add_feed(db: &Database, url: &str, name: Option<&str>) -> Result<()> {
    // 取得中メッセージを表示
    // `Colorize` トレイトにより、文字列に `.blue()` などのメソッドが追加される
    println!("{} {}", "Fetching feed:".blue(), url);

    // フィードを取得してパース
    // `_articles` のアンダースコアプレフィックスは「未使用変数」を示す
    // コンパイラの警告を抑制するためのRustの慣習
    let (mut feed_info, _articles) = feed::fetch_feed(url)
        .await
        .with_context(|| format!("Failed to fetch feed from {}", url))?;

    // カスタム名が指定されていれば上書き
    // `if let Some(...)` は Option から値を取り出すイディオム
    if let Some(custom_name) = name {
        // `to_string()` で &str から String を作成
        feed_info.title = custom_name.to_string();
    }

    // データベースに保存
    let id = db.add_feed(&feed_info)?;

    // 成功メッセージを表示
    // `format!()` は文字列をフォーマットする（printlnの返り値版）
    // `.bold()`, `.green()` は colored クレートの機能
    println!(
        "{} {} (ID: {})",
        "Added feed:".green(),
        feed_info.title.bold(),
        id
    );

    Ok(())
}

/// RSSフィードを削除する
///
/// # 引数
///
/// * `db` - データベース接続への参照
/// * `id` - 削除するフィードのID
///
/// # 注意
///
/// フィードを削除すると、関連する全ての記事も削除されます
/// （ON DELETE CASCADE）。
pub fn remove_feed(db: &Database, id: i64) -> Result<()> {
    // `remove_feed` は削除成功時に true を返す
    if db.remove_feed(id)? {
        println!("{} {}", "Removed feed with ID:".green(), id);
    } else {
        // `.yellow()` で警告色
        println!("{} {}", "Feed not found with ID:".yellow(), id);
    }
    Ok(())
}

/// 登録済みの全フィードを一覧表示する
///
/// # 出力フォーマット
///
/// ```text
/// Registered Feeds:
///
///   [1] Feed Title (https://example.com/feed.xml)
///       Description text...
/// ```
///
/// # 空の場合
///
/// フィードが登録されていない場合は、使い方のヒントを表示します。
pub fn list_feeds(db: &Database) -> Result<()> {
    // データベースから全フィードを取得
    let feeds = db.get_feeds()?;

    // フィードが空の場合
    if feeds.is_empty() {
        println!("{}", "No feeds registered yet.".yellow());
        println!("Use 'rustfeed add <url>' to add a feed.");
        return Ok(()); // 早期リターン
    }

    // ヘッダー表示
    // `.underline()` で下線付き
    println!("{}", "Registered Feeds:".bold().underline());
    println!(); // 空行

    // 各フィードを表示
    // `for ... in` はイテレータをループ処理
    for feed in feeds {
        // フォーマット文字列で整形
        println!(
            "  {} {} {}",
            format!("[{}]", feed.id).cyan(),    // ID（シアン色）
            feed.title.bold(),                  // タイトル（太字）
            format!("({})", feed.url).dimmed()  // URL（薄い色）
        );

        // 説明があれば表示（最初の80文字まで）
        if let Some(desc) = &feed.description {
            // `chars()` でUnicode文字単位でイテレート
            // `take(80)` で最初の80文字を取得
            // `collect()` で String に収集
            let short_desc: String = desc.chars().take(80).collect();
            println!("      {}", short_desc.dimmed());
        }
    }

    Ok(())
}

// =============================================================================
// 記事関連コマンド
// =============================================================================

/// 全フィードから新しい記事を取得する
///
/// # 処理の流れ
///
/// 1. 登録済みの全フィードを取得
/// 2. 各フィードについて:
///    - RSSを取得・パース
///    - 新規記事をデータベースに保存
///    - 結果を表示
/// 3. 取得した記事数のサマリーを表示
///
/// # エラーハンドリング
///
/// 個別のフィード取得エラーは表示のみで、他のフィードの処理は続行します。
/// これにより、1つのフィードが壊れていても全体が失敗しません。
pub async fn fetch_feeds(db: &Database) -> Result<()> {
    let feeds = db.get_feeds()?;

    if feeds.is_empty() {
        println!("{}", "No feeds registered yet.".yellow());
        return Ok(());
    }

    println!("{}", "Fetching articles from all feeds...".blue());
    println!();

    // 新規取得記事数のカウンター
    // `mut` で可変変数として宣言
    let mut total_new = 0;

    // 各フィードを処理
    for stored_feed in feeds {
        // プログレス表示（改行なし）
        // `print!` は改行しない版の `println!`
        print!("  {} {}... ", "Fetching".dimmed(), stored_feed.title);

        // フィード取得を試みる
        // `match` でResult を処理
        match feed::fetch_feed(&stored_feed.url).await {
            Ok((_feed_info, articles)) => {
                // 成功: 記事をデータベースに保存
                let mut new_count = 0;

                for mut article in articles {
                    // feed_id を設定
                    // `mut article` なので変更可能
                    article.feed_id = stored_feed.id;

                    // 新規記事（重複でない）の場合はカウント
                    // `is_some()` は Option が Some かどうかを判定
                    if db.add_article(&article)?.is_some() {
                        new_count += 1;
                    }
                }

                // 結果表示
                println!("{} ({} new)", "OK".green(), new_count.to_string().cyan());

                total_new += new_count;
            }
            Err(e) => {
                // エラー: メッセージを表示して続行
                // エラーを握りつぶさず、表示はする
                println!("{} ({})", "ERROR".red(), e);
            }
        }
    }

    println!();
    println!(
        "{} {} new articles fetched.",
        "Done!".green().bold(),
        total_new.to_string().cyan()
    );

    Ok(())
}

/// 記事を一覧表示する
///
/// # 引数
///
/// * `db` - データベース接続
/// * `unread_only` - true なら未読記事のみ表示
/// * `limit` - 表示する最大件数
/// * `filter` - キーワードフィルタ（カンマ区切りで複数指定可能）
/// * `feed_id` - 特定のフィードIDでフィルタ（None の場合は全フィード）
/// * `disabled_feeds` - 無効化するフィードIDのリスト
///
/// # 出力フォーマット
///
/// ```text
/// Articles:
///
///   [*] [1] 2024-01-01 Article Title
///       https://example.com/article
///   [x] [2] 2024-01-02 Read Article
///       https://example.com/read
/// ```
///
/// - `[*]` = 未読（シアン色）
/// - `[x]` = 既読（薄い色）
pub fn show_articles(
    db: &Database,
    unread_only: bool,
    limit: usize,
    filter: Option<&str>,
    feed_id: Option<i64>,
    disabled_feeds: &[i64],
) -> Result<()> {
    // 記事を取得
    let mut articles = db.get_articles(unread_only, limit, filter, feed_id)?;

    // disabled_feedsでフィルタリング（feed_idが指定されていない場合のみ）
    if feed_id.is_none() && !disabled_feeds.is_empty() {
        articles.retain(|article| !disabled_feeds.contains(&article.feed_id));
    }

    // 空の場合の処理
    if articles.is_empty() {
        if unread_only {
            // 未読がない = 良い状態
            println!("{}", "No unread articles.".green());
        } else {
            println!("{}", "No articles found.".yellow());
            println!("Use 'rustfeed fetch' to get articles from your feeds.");
        }
        return Ok(());
    }

    // ヘッダー（フィルタ状態に応じて変更）
    let header = if unread_only {
        "Unread Articles:"
    } else {
        "Articles:"
    };
    println!("{}", header.bold().underline());
    println!();

    // 各記事を表示
    for article in articles {
        // 既読/未読マーカー
        // 条件式で異なる型を返す場合、両方が同じ型である必要がある
        let read_marker = if article.is_read {
            "[x]".dimmed() // ColoredString 型
        } else {
            "[*]".cyan() // ColoredString 型
        };

        // 日付フォーマット
        // `map()` で Option の中身を変換
        // `unwrap_or_else()` で None 時のデフォルト値
        let date = article
            .published_at
            .map(|dt| dt.format("%Y-%m-%d").to_string()) // 日付をフォーマット
            .unwrap_or_else(|| "----------".to_string()); // 日付なしの場合

        // 記事情報を表示
        println!(
            "  {} {} {} {}",
            read_marker,
            format!("[{}]", article.id).dimmed(),
            date.dimmed(),
            article.title.bold()
        );

        // URLがあれば表示
        if let Some(url) = &article.url {
            // `&article.url` は参照を借用
            // 所有権を移動せずに内容を参照できる
            println!("      {}", url.dimmed());
        }
    }

    Ok(())
}

/// 記事を既読としてマークする
///
/// # 引数
///
/// * `db` - データベース接続
/// * `id` - 既読にする記事のID
///
/// # 結果
///
/// - 記事が存在すれば既読にマーク
/// - 存在しなければ警告を表示
pub fn mark_as_read(db: &Database, id: i64) -> Result<()> {
    if db.mark_as_read(id)? {
        println!("{} {}", "Marked as read:".green(), id);
    } else {
        println!("{} {}", "Article not found with ID:".yellow(), id);
    }
    Ok(())
}

/// 記事をお気に入りに追加する
///
/// # 引数
///
/// * `db` - データベース接続
/// * `id` - お気に入りにする記事のID
///
/// # 結果
///
/// - 記事が存在すればお気に入りに追加
/// - 存在しなければ警告を表示
pub fn add_favorite(db: &Database, id: i64) -> Result<()> {
    if db.add_favorite(id)? {
        println!("{} {}", "Added to favorites:".green(), id);
    } else {
        println!("{} {}", "Article not found with ID:".yellow(), id);
    }
    Ok(())
}

/// 記事をお気に入りから削除する
///
/// # 引数
///
/// * `db` - データベース接続
/// * `id` - お気に入りから削除する記事のID
///
/// # 結果
///
/// - 記事が存在すればお気に入りから削除
/// - 存在しなければ警告を表示
pub fn remove_favorite(db: &Database, id: i64) -> Result<()> {
    if db.remove_favorite(id)? {
        println!("{} {}", "Removed from favorites:".green(), id);
    } else {
        println!("{} {}", "Article not found with ID:".yellow(), id);
    }
    Ok(())
}

/// お気に入り記事を一覧表示する
///
/// # 引数
///
/// * `db` - データベース接続
/// * `limit` - 表示する最大件数
///
/// # 出力フォーマット
///
/// ```text
/// Favorite Articles:
///
///   [*] [1] 2024-01-01 Article Title
///       https://example.com/article
/// ```
///
/// - `[*]` = 未読（シアン色）
/// - `[x]` = 既読（薄い色）
pub fn show_favorites(db: &Database, limit: usize) -> Result<()> {
    // お気に入り記事を取得
    let articles = db.get_favorite_articles(limit)?;

    // 空の場合の処理
    if articles.is_empty() {
        println!("{}", "No favorite articles.".yellow());
        println!("Use 'rustfeed favorite <id>' to add articles to favorites.");
        return Ok(());
    }

    // ヘッダー
    println!("{}", "Favorite Articles:".bold().underline());
    println!();

    // 各記事を表示
    for article in articles {
        // 既読/未読マーカー
        let read_marker = if article.is_read {
            "[x]".dimmed()
        } else {
            "[*]".cyan()
        };

        // 日付フォーマット
        let date = article
            .published_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "----------".to_string());

        // 記事情報を表示
        println!(
            "  {} {} {} {}",
            read_marker,
            format!("[{}]", article.id).dimmed(),
            date.dimmed(),
            article.title.bold()
        );

        // URLがあれば表示
        if let Some(url) = &article.url {
            println!("      {}", url.dimmed());
        }
    }

    Ok(())
}

/// 記事をエクスポートする
///
/// # 引数
///
/// * `db` - データベース接続
/// * `format` - エクスポート形式（"json" または "markdown"）
/// * `favorites` - お気に入りのみエクスポートする場合 true
/// * `unread` - 未読のみエクスポートする場合 true
/// * `limit` - エクスポートする記事数の上限（None の場合は全件）
///
/// # エクスポート形式
///
/// ## JSON
/// 記事の配列を JSON 形式で出力します。機械可読で他のツールとの連携に適しています。
///
/// ```json
/// [
///   {
///     "id": 1,
///     "feed_id": 1,
///     "title": "記事タイトル",
///     "url": "https://example.com/article",
///     ...
///   }
/// ]
/// ```
///
/// ## Markdown
/// 人間が読みやすい Markdown 形式で出力します。ドキュメントとして利用可能です。
///
/// ```markdown
/// # Exported Articles
///
/// ## 記事タイトル
/// **Published:** 2024-01-01  
/// **URL:** https://example.com/article  
/// **Read:** Yes
///
/// 記事の本文...
/// ```
///
/// # 使用例
///
/// ```bash
/// rustfeed export > articles.json                    # JSON形式で全記事
/// rustfeed export --format markdown > articles.md    # Markdown形式
/// rustfeed export --favorites > favorites.json       # お気に入りのみ
/// rustfeed export --unread -l 50 > unread.json       # 未読50件
/// ```
pub fn export_articles(
    db: &Database,
    format: &str,
    favorites: bool,
    unread: bool,
    limit: Option<usize>,
) -> Result<()> {
    // 記事を取得
    // お気に入りフラグに基づいて適切なメソッドを呼び出す
    let articles = if favorites {
        // お気に入り記事を取得
        let limit_val = limit.unwrap_or(usize::MAX);
        db.get_favorite_articles(limit_val)?
    } else {
        // 通常の記事を取得
        let limit_val = limit.unwrap_or(usize::MAX);
        db.get_articles(unread, limit_val, None, None)?
    };

    // 空の場合の処理
    if articles.is_empty() {
        eprintln!("{}", "No articles to export.".yellow());
        return Ok(());
    }

    // フォーマットに応じてエクスポート
    match format.to_lowercase().as_str() {
        "json" => export_as_json(&articles)?,
        "markdown" | "md" => export_as_markdown(&articles)?,
        _ => {
            // サポートされていないフォーマット
            anyhow::bail!(
                "Unsupported format: '{}'. Use 'json' or 'markdown'.",
                format
            );
        }
    }

    Ok(())
}

/// 記事を JSON 形式でエクスポートする
///
/// # 引数
///
/// * `articles` - エクスポートする記事のスライス
///
/// # 実装詳細
///
/// `serde_json::to_string_pretty()` を使用して、整形された JSON を出力します。
/// `pretty` を使うことで、人間が読みやすいインデントされた JSON になります。
fn export_as_json(articles: &[Article]) -> Result<()> {
    // serde_json でシリアライズ
    // to_string_pretty() は整形された（インデント付き）JSON を生成
    let json =
        serde_json::to_string_pretty(articles).context("Failed to serialize articles to JSON")?;

    // 標準出力に出力（リダイレクトでファイルに保存可能）
    println!("{}", json);

    Ok(())
}

/// 記事を Markdown 形式でエクスポートする
///
/// # 引数
///
/// * `articles` - エクスポートする記事のスライス
///
/// # 実装詳細
///
/// 各記事を見出し、メタデータ、本文の順に出力します。
/// Markdown 形式により、そのままドキュメントとして使用できます。
fn export_as_markdown(articles: &[Article]) -> Result<()> {
    // ヘッダー
    println!("# Exported Articles\n");
    println!("Total: {} articles\n", articles.len());
    println!("---\n");

    // 各記事を Markdown 形式で出力
    for (index, article) in articles.iter().enumerate() {
        // 記事番号と見出し
        println!("## {}. {}\n", index + 1, article.title);

        // メタデータ
        if let Some(ref url) = article.url {
            println!("**URL:** {}\n", url);
        }

        if let Some(published_at) = article.published_at {
            println!(
                "**Published:** {}\n",
                published_at.format("%Y-%m-%d %H:%M:%S")
            );
        }

        println!("**Read:** {}\n", if article.is_read { "Yes" } else { "No" });
        println!(
            "**Favorite:** {}\n",
            if article.is_favorite { "Yes" } else { "No" }
        );

        // 本文
        if let Some(ref content) = article.content {
            println!("### Content\n");
            println!("{}\n", content);
        }

        // 区切り線（最後の記事以外）
        if index < articles.len() - 1 {
            println!("---\n");
        }
    }

    Ok(())
}
