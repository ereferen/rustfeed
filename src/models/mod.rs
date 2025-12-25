//! # データモデル
//!
//! このモジュールは、アプリケーション全体で使用されるデータ構造を定義します。
//!
//! ## 主な型
//!
//! - [`Feed`] - RSSフィードのメタデータ
//! - [`Article`] - 個別の記事データ
//!
//! ## Serdeについて
//!
//! `serde` は Rust の代表的なシリアライズ/デシリアライズフレームワークです。
//! `#[derive(Serialize, Deserialize)]` を付けることで、構造体を
//! JSON, TOML, YAML などの形式に変換できるようになります。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// =============================================================================
// Feed 構造体
// =============================================================================

/// RSSフィードの情報を表す構造体
///
/// # フィールド
///
/// | フィールド | 型 | 説明 |
/// |------------|-----|------|
/// | `id` | `i64` | データベースで自動採番されるID |
/// | `url` | `String` | フィードのURL |
/// | `title` | `String` | フィードのタイトル |
/// | `description` | `Option<String>` | フィードの説明（任意） |
/// | `created_at` | `DateTime<Utc>` | 作成日時 |
/// | `updated_at` | `DateTime<Utc>` | 更新日時 |
///
/// # Derive マクロの説明
///
/// - `Debug`: `{:?}` フォーマットでのデバッグ出力を可能にする
/// - `Clone`: `.clone()` メソッドで値のコピーを作成可能にする
/// - `Serialize`: JSON等への変換を可能にする（serde）
/// - `Deserialize`: JSON等からの読み込みを可能にする（serde）
///
/// # 例
///
/// ```rust
/// use rustfeed::models::Feed;
///
/// let feed = Feed::new(
///     "https://example.com/feed.xml".to_string(),
///     "Example Feed".to_string(),
///     Some("An example RSS feed".to_string()),
/// );
/// println!("{:?}", feed);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    /// データベースで自動採番されるユニークID
    ///
    /// 新規作成時は0を設定し、DBへの挿入後に実際のIDが設定されます。
    pub id: i64,

    /// RSSフィードのURL
    ///
    /// 例: "https://blog.rust-lang.org/feed.xml"
    pub url: String,

    /// フィードのタイトル
    ///
    /// RSSフィードから取得されるか、ユーザーが指定できます。
    pub title: String,

    /// フィードの説明文（オプショナル）
    ///
    /// `Option<T>` は Rust の「存在するかもしれない値」を表す型です。
    /// - `Some(value)`: 値が存在する
    /// - `None`: 値が存在しない
    ///
    /// これにより、nullポインタ例外を型システムで防ぐことができます。
    pub description: Option<String>,

    /// 登録日時（UTC）
    ///
    /// `chrono::DateTime<Utc>` は時刻を扱う型で、タイムゾーンを型で表現します。
    pub created_at: DateTime<Utc>,

    /// 最終更新日時（UTC）
    pub updated_at: DateTime<Utc>,
}

impl Feed {
    /// 新しい Feed インスタンスを作成する
    ///
    /// # 引数
    ///
    /// * `url` - RSSフィードのURL
    /// * `title` - フィードのタイトル
    /// * `description` - フィードの説明（任意）
    ///
    /// # 戻り値
    ///
    /// 新しい `Feed` インスタンス。`id` は 0 に初期化され、
    /// `created_at` と `updated_at` は現在時刻に設定されます。
    ///
    /// # 所有権について
    ///
    /// 引数の `String` は「所有権を受け取る」形式です。
    /// 呼び出し元は `.to_string()` や `String::from()` で String を作成するか、
    /// 既存の String の所有権を渡す必要があります。
    ///
    /// # 例
    ///
    /// ```rust
    /// let feed = Feed::new(
    ///     "https://example.com/feed".to_string(),
    ///     "My Feed".to_string(),
    ///     None, // 説明なし
    /// );
    /// ```
    pub fn new(url: String, title: String, description: Option<String>) -> Self {
        // 現在時刻を取得
        let now = Utc::now();

        // Self は impl ブロック内で現在の型（Feed）を指すエイリアス
        Self {
            id: 0, // データベース挿入後に実際のIDが設定される
            url,
            title,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

// =============================================================================
// Article 構造体
// =============================================================================

/// RSS記事を表す構造体
///
/// # フィールド
///
/// | フィールド | 型 | 説明 |
/// |------------|-----|------|
/// | `id` | `i64` | データベースで自動採番されるID |
/// | `feed_id` | `i64` | 所属するフィードのID（外部キー） |
/// | `title` | `String` | 記事のタイトル |
/// | `url` | `Option<String>` | 記事のURL（任意） |
/// | `content` | `Option<String>` | 記事の本文/要約（任意） |
/// | `published_at` | `Option<DateTime<Utc>>` | 公開日時（任意） |
/// | `is_read` | `bool` | 既読フラグ |
/// | `created_at` | `DateTime<Utc>` | 取得日時 |
///
/// # 例
///
/// ```rust
/// let article = Article::new(
///     1, // feed_id
///     "New Rust Release".to_string(),
///     Some("https://blog.rust-lang.org/...".to_string()),
///     Some("Rust 1.x has been released!".to_string()),
///     Some(Utc::now()),
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    /// データベースで自動採番されるユニークID
    pub id: i64,

    /// この記事が属するフィードのID（外部キー）
    ///
    /// リレーショナルデータベースの概念で、feeds テーブルの id を参照します。
    pub feed_id: i64,

    /// 記事のタイトル
    pub title: String,

    /// 記事へのURL（オプショナル）
    ///
    /// RSS/Atomフィードによっては URL が含まれない場合があるため Option です。
    pub url: Option<String>,

    /// 記事の内容または要約（オプショナル）
    ///
    /// フィードによっては要約のみ、全文のみ、または両方ある場合があります。
    pub content: Option<String>,

    /// 記事の公開日時（オプショナル）
    ///
    /// フィードによっては公開日時が含まれない場合があります。
    /// `Option<DateTime<Utc>>` は「存在するかもしれない日時」を表します。
    pub published_at: Option<DateTime<Utc>>,

    /// 既読フラグ
    ///
    /// `bool` は真偽値型で、`true` または `false` のみを取ります。
    pub is_read: bool,

    /// この記事をデータベースに保存した日時
    pub created_at: DateTime<Utc>,
}

impl Article {
    /// 新しい Article インスタンスを作成する
    ///
    /// # 引数
    ///
    /// * `feed_id` - 所属するフィードのID
    /// * `title` - 記事のタイトル
    /// * `url` - 記事のURL（任意）
    /// * `content` - 記事の内容/要約（任意）
    /// * `published_at` - 公開日時（任意）
    ///
    /// # 戻り値
    ///
    /// 新しい `Article` インスタンス。`is_read` は `false` に初期化されます。
    ///
    /// # 例
    ///
    /// ```rust
    /// let article = Article::new(
    ///     1,
    ///     "Title".to_string(),
    ///     Some("https://example.com/article".to_string()),
    ///     None,
    ///     None,
    /// );
    /// assert!(!article.is_read); // 新規記事は未読
    /// ```
    pub fn new(
        feed_id: i64,
        title: String,
        url: Option<String>,
        content: Option<String>,
        published_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: 0, // データベース挿入後に実際のIDが設定される
            feed_id,
            title,
            url,
            content,
            published_at,
            is_read: false, // 新規記事は未読状態で作成
            created_at: Utc::now(),
        }
    }
}
