//! # RSS/Atom フィード取得モジュール
//!
//! このモジュールは、URLからRSS/Atomフィードを取得し、
//! パースしてアプリケーションのデータモデルに変換する機能を提供します。
//!
//! ## 対応フォーマット
//!
//! `feed-rs` クレートにより、以下のフォーマットに対応しています:
//!
//! - RSS 0.9, 1.0, 2.0
//! - Atom 0.3, 1.0
//! - JSON Feed
//!
//! ## 使用例
//!
//! ```rust,no_run
//! use rustfeed_core::feed::fetch_feed;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (feed, articles) = fetch_feed("https://blog.rust-lang.org/feed.xml")
//!         .await
//!         .expect("Failed to fetch feed");
//!
//!     println!("Feed: {}", feed.title);
//!     println!("Articles: {}", articles.len());
//! }
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use feed_rs::parser;

use crate::db::Database;
use crate::models::{Article, Feed};

// =============================================================================
// 公開関数
// =============================================================================

/// URLからRSS/Atomフィードを取得してパースする
///
/// # 引数
///
/// * `url` - フィードのURL（`&str` = 文字列スライス = 文字列への参照）
///
/// # 戻り値
///
/// `(Feed, Vec<Article>)` のタプル:
/// - `Feed`: フィードのメタ情報
/// - `Vec<Article>`: 記事のリスト（`feed_id` は 0 で初期化）
///
/// # 非同期関数について
///
/// `async fn` はRustの非同期関数を定義します。
/// この関数を呼び出すと `Future` が返され、`.await` で実行を待機します。
///
/// 非同期処理のメリット:
/// - I/O待ち（ネットワーク、ファイル）中に他の処理を実行可能
/// - 複数のフィードを並行して取得可能（将来の改善点）
///
/// # エラー
///
/// - ネットワークエラー（接続失敗、タイムアウト）
/// - パースエラー（無効なXML、未対応フォーマット）
///
/// # 例
///
/// ```rust,no_run
/// # use rustfeed_core::feed::fetch_feed;
/// # async fn example() -> anyhow::Result<()> {
/// let (feed, articles) = fetch_feed("https://example.com/feed.xml").await?;
///
/// // フィード情報を表示
/// println!("Title: {}", feed.title);
///
/// // 記事をイテレート
/// for article in articles {
///     println!("- {}", article.title);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch_feed(url: &str) -> Result<(Feed, Vec<Article>)> {
    // -------------------------------------------------------------------------
    // Step 1: HTTPリクエストでフィードを取得
    // -------------------------------------------------------------------------

    // `reqwest::get()` は非同期HTTPクライアント
    // `.await` でレスポンスが返るまで待機（この間、他のタスクが実行可能）
    let response = reqwest::get(url)
        .await
        // `with_context()` でエラーに追加情報を付与
        // クロージャ `|| format!(...)` はエラー時のみ評価される（遅延評価）
        .with_context(|| format!("Failed to fetch feed from {}", url))?;

    // レスポンスボディをバイト列として取得
    // HTTP通信が完了してからボディを読み取る
    let bytes = response
        .bytes()
        .await
        .with_context(|| "Failed to read response body")?;

    // -------------------------------------------------------------------------
    // Step 2: フィードをパース
    // -------------------------------------------------------------------------

    // `feed_rs::parser::parse()` はバイト列からフィードをパース
    // `&bytes[..]` は `Bytes` 型を `&[u8]`（バイトスライス）に変換
    // これは Deref トレイトによる自動変換の一例
    let parsed = parser::parse(&bytes[..]).with_context(|| "Failed to parse feed")?;

    // -------------------------------------------------------------------------
    // Step 3: Feed 構造体を作成
    // -------------------------------------------------------------------------

    // タイトルを取得（存在しない場合はデフォルト値）
    // `map()` は Option/Result の中身を変換する
    // `unwrap_or_else()` は None/Err の場合にクロージャを実行
    let title = parsed
        .title
        .map(|t| t.content) // Some(Text) -> Some(String)
        .unwrap_or_else(|| "Untitled Feed".to_string()); // None -> String

    // 説明を取得（オプショナル）
    let description = parsed.description.map(|d| d.content);

    // Feed 構造体を作成
    let feed = Feed::new(url.to_string(), title, description);

    // -------------------------------------------------------------------------
    // Step 4: Article 構造体のリストを作成
    // -------------------------------------------------------------------------

    // イテレータチェーン:
    // 1. `into_iter()` - ベクタをイテレータに変換（所有権を移動）
    // 2. `map()` - 各要素を変換
    // 3. `collect()` - イテレータを新しいベクタに収集
    //
    // この方式はforループより宣言的で、Rustでは一般的なパターンです。
    let articles: Vec<Article> = parsed
        .entries
        .into_iter() // Vec<Entry> をイテレータに変換
        .map(|entry| {
            // --- エントリのタイトルを取得 ---
            let title = entry
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "Untitled".to_string());

            // --- エントリのURLを取得（最初のリンク）---
            // `first()` は Option<&T> を返す
            // `map()` で所有権のある String にクローン
            let url = entry.links.first().map(|l| l.href.clone());

            // --- コンテンツを取得（summary または content から）---
            // `or_else()` は None の場合に別の Option を試す
            // これは「フォールバック」パターンの一例
            let content = entry
                .summary
                .map(|s| s.content) // まず summary を試す
                .or_else(|| {
                    // summary が None なら content を試す
                    entry.content.and_then(|c| c.body)
                });

            // --- 公開日時を取得（published または updated から）---
            // `or()` は Option 同士のフォールバック
            let published_at: Option<DateTime<Utc>> = entry
                .published
                .or(entry.updated) // published がなければ updated を使用
                .map(|dt| dt.with_timezone(&Utc)); // タイムゾーンをUTCに統一

            // Article 構造体を作成
            // feed_id は 0（呼び出し元で正しいIDを設定する）
            Article::new(0, title, url, content, published_at)
        })
        .collect(); // イテレータを Vec<Article> に収集

    // タプルで返す
    // Rust ではタプルを使って複数の値を返すことができる
    Ok((feed, articles))
}

/// パースしたフィードの記事をデータベースに保存する
///
/// # 引数
///
/// * `db` - データベース接続への参照
/// * `feed_id` - 記事を関連付けるフィードのID
/// * `feed_data` - `fetch_feed`から返されたタプル (Feed, Vec<Article>)
///
/// # 戻り値
///
/// 新規追加された記事の数
///
/// # 使用例
///
/// ```rust,no_run
/// use rustfeed_core::{db::Database, feed::{fetch_feed, save_articles}};
///
/// async fn update_feed(db: &Database, feed_id: i64, url: &str) -> anyhow::Result<usize> {
///     let feed_data = fetch_feed(url).await?;
///     let count = save_articles(db, feed_id, &feed_data)?;
///     Ok(count)
/// }
/// ```
pub fn save_articles(
    db: &Database,
    feed_id: i64,
    feed_data: &(Feed, Vec<Article>),
) -> Result<usize> {
    let (_, articles) = feed_data;
    let mut count = 0;

    for article in articles {
        // feed_id を設定して記事を作成
        let article_with_feed_id = Article {
            feed_id,
            ..article.clone()
        };

        // 記事をデータベースに追加（重複は無視される）
        if let Some(_id) = db.add_article(&article_with_feed_id)? {
            count += 1;
        }
    }

    Ok(count)
}

// =============================================================================
// テスト
// =============================================================================

/// テストモジュール
///
/// `#[cfg(test)]` 属性により、このモジュールはテスト時のみコンパイルされます。
/// `cargo test` で実行できます。
#[cfg(test)]
mod tests {
    // 親モジュールの全てをインポート
    use super::*;

    /// Rustブログのフィードを取得するテスト
    ///
    /// `#[tokio::test]` は非同期テストを定義するマクロです。
    /// 通常の `#[test]` の代わりに使用し、async 関数をテストできます。
    ///
    /// # テストの実行
    ///
    /// ```bash
    /// cargo test test_fetch_feed
    /// ```
    #[tokio::test]
    async fn test_fetch_feed() {
        // 実際のRSSフィードを取得してテスト
        // これは「統合テスト」に近い（外部サービスに依存）
        let result = fetch_feed("https://blog.rust-lang.org/feed.xml").await;

        // assert! マクロでテスト条件を検証
        // 第2引数はテスト失敗時のメッセージ
        assert!(result.is_ok(), "Should successfully fetch Rust blog feed");

        // パターンマッチでテスト続行
        if let Ok((feed, articles)) = result {
            // フィードにタイトルがあることを確認
            assert!(!feed.title.is_empty(), "Feed should have a title");

            // 記事が存在することを確認
            assert!(!articles.is_empty(), "Feed should have articles");
        }
    }
}
