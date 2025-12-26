//! # データベース操作モジュール
//!
//! SQLiteデータベースとの連携を担当するモジュールです。
//!
//! ## 概要
//!
//! このモジュールは [`Database`] 構造体を提供し、
//! フィードと記事のCRUD操作（作成・読み取り・更新・削除）を行います。
//!
//! ## データベース構成
//!
//! - **保存場所**: `~/.rustfeed/rustfeed.db`
//! - **テーブル**:
//!   - `feeds`: RSSフィード情報
//!   - `articles`: 記事情報（feedsへの外部キーを持つ）
//!
//! ## 使用例
//!
//! ```rust,no_run
//! use rustfeed::db::Database;
//!
//! let db = Database::new()?;
//! db.init()?;
//!
//! // フィード一覧を取得
//! let feeds = db.get_feeds()?;
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::models::{Article, Feed};

// =============================================================================
// Database 構造体
// =============================================================================

/// SQLiteデータベースへの接続を管理する構造体
///
/// # 構造体の設計
///
/// Rust では構造体のフィールドに対してアクセス修飾子を設定できます。
/// `conn` フィールドは `pub` がないため、このモジュール内でのみアクセス可能です。
/// これにより、外部からデータベース接続を直接操作されることを防ぎます。
///
/// # ライフタイムについて
///
/// この構造体は `Connection` を所有しています。
/// 構造体がドロップ（破棄）されるとき、Connection も自動的に閉じられます。
/// これが Rust の RAII（Resource Acquisition Is Initialization）パターンです。
pub struct Database {
    /// SQLiteデータベース接続
    ///
    /// `rusqlite::Connection` はスレッドセーフではないため、
    /// 複数スレッドで使用する場合は `Mutex` で保護する必要があります。
    conn: Connection,
}

impl Database {
    /// 新しいデータベース接続を作成する
    ///
    /// データベースファイルが存在しない場合は自動的に作成されます。
    /// 親ディレクトリ（`~/.rustfeed/`）も必要に応じて作成されます。
    ///
    /// # 戻り値
    ///
    /// - `Ok(Database)`: 接続成功
    /// - `Err(...)`: ファイル作成やDB接続に失敗
    ///
    /// # エラーハンドリング
    ///
    /// `with_context()` は `anyhow` クレートの機能で、
    /// エラーにコンテキスト情報（何をしようとしていたか）を追加します。
    /// これにより、デバッグ時に原因を特定しやすくなります。
    pub fn new() -> Result<Self> {
        // データベースファイルのパスを取得
        let db_path = Self::get_db_path()?;

        // 親ディレクトリを作成（存在しない場合）
        // `if let Some(...)` は Option から値を取り出すイディオム
        if let Some(parent) = db_path.parent() {
            // `create_dir_all` は `mkdir -p` と同等で、再帰的にディレクトリを作成
            std::fs::create_dir_all(parent)?;
        }

        // データベース接続を開く
        // `Connection::open` はファイルが存在しなければ新規作成する
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;

        Ok(Self { conn })
    }

    /// データベースファイルのパスを取得する（プライベート関数）
    ///
    /// # 戻り値
    ///
    /// `~/.rustfeed/rustfeed.db` へのパス
    ///
    /// # パスの構築
    ///
    /// `PathBuf` は所有権を持つパス型で、`join` で安全にパスを連結できます。
    /// これにより、OS間のパス区切り文字の違い（`/` vs `\`）を自動処理します。
    fn get_db_path() -> Result<PathBuf> {
        // `dirs::home_dir()` はホームディレクトリを取得（Noneの可能性あり）
        let home = dirs::home_dir().context("Could not find home directory")?;

        // パスの連結: home/.rustfeed/rustfeed.db
        Ok(home.join(".rustfeed").join("rustfeed.db"))
    }

    /// データベーステーブルを初期化する
    ///
    /// `CREATE TABLE IF NOT EXISTS` を使用しているため、
    /// テーブルが既に存在する場合は何もしません（べき等性）。
    ///
    /// # テーブル構造
    ///
    /// ## feeds テーブル
    /// | カラム | 型 | 説明 |
    /// |--------|-----|------|
    /// | id | INTEGER | 主キー（自動採番） |
    /// | url | TEXT | フィードURL（ユニーク） |
    /// | title | TEXT | タイトル |
    /// | description | TEXT | 説明（NULL可） |
    /// | created_at | TEXT | 作成日時（RFC3339） |
    /// | updated_at | TEXT | 更新日時（RFC3339） |
    ///
    /// ## articles テーブル
    /// | カラム | 型 | 説明 |
    /// |--------|-----|------|
    /// | id | INTEGER | 主キー（自動採番） |
    /// | feed_id | INTEGER | 外部キー（feeds.id） |
    /// | title | TEXT | タイトル |
    /// | url | TEXT | 記事URL（NULL可） |
    /// | content | TEXT | 本文（NULL可） |
    /// | published_at | TEXT | 公開日時（NULL可） |
    /// | is_read | INTEGER | 既読フラグ（0/1） |
    /// | created_at | TEXT | 取得日時 |
    ///
    /// # SQLについて
    ///
    /// - `PRIMARY KEY AUTOINCREMENT`: 自動的に一意のIDを生成
    /// - `NOT NULL`: NULL値を禁止
    /// - `UNIQUE`: 重複を禁止
    /// - `FOREIGN KEY`: 他テーブルへの参照制約
    /// - `ON DELETE CASCADE`: 親レコード削除時に子も削除
    pub fn init(&self) -> Result<()> {
        // feeds テーブルの作成
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [], // パラメータなし
        )?;

        // articles テーブルの作成
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS articles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                url TEXT,
                content TEXT,
                published_at TEXT,
                is_read INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE,
                UNIQUE(feed_id, url)
            )",
            [],
        )?;

        // インデックスの作成（クエリ高速化のため）
        // インデックスは検索を高速化するが、挿入/更新時のオーバーヘッドがある
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_is_read ON articles(is_read)",
            [],
        )?;

        // マイグレーション: is_favorite カラムの追加
        // 既存のテーブルにカラムが存在しない場合のみ追加
        // SQLiteでは IF NOT EXISTS が使えないため、エラーを無視する
        let _ = self.conn.execute(
            "ALTER TABLE articles ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0",
            [],
        );

        // is_favorite用のインデックスを追加
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_is_favorite ON articles(is_favorite)",
            [],
        )?;

        Ok(())
    }

    // =========================================================================
    // Feed 関連のCRUD操作
    // =========================================================================

    /// 新しいフィードをデータベースに追加する
    ///
    /// # 引数
    ///
    /// * `feed` - 追加するフィード（参照で受け取る）
    ///
    /// # 戻り値
    ///
    /// 挿入されたレコードのID
    ///
    /// # 参照について
    ///
    /// `&Feed` は「Feedへの参照」を意味します。
    /// 所有権を移動せずに値を借用（borrow）するため、
    /// 呼び出し元は `feed` を引き続き使用できます。
    ///
    /// # SQL インジェクション対策
    ///
    /// `params![]` マクロを使ったパラメータ化クエリにより、
    /// SQLインジェクション攻撃を防いでいます。
    /// 値はプレースホルダ（?1, ?2...）で指定し、実際の値は別途渡します。
    pub fn add_feed(&self, feed: &Feed) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feeds (url, title, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                feed.url,
                feed.title,
                feed.description,
                feed.created_at.to_rfc3339(), // RFC3339形式の文字列に変換
                feed.updated_at.to_rfc3339(),
            ],
        )?;

        // 最後に挿入されたレコードのIDを返す
        Ok(self.conn.last_insert_rowid())
    }

    /// フィードをIDで削除する
    ///
    /// # 引数
    ///
    /// * `id` - 削除するフィードのID
    ///
    /// # 戻り値
    ///
    /// - `Ok(true)`: 削除成功
    /// - `Ok(false)`: 該当するフィードが存在しなかった
    ///
    /// # 注意
    ///
    /// `ON DELETE CASCADE` により、関連する記事も自動削除されます。
    pub fn remove_feed(&self, id: i64) -> Result<bool> {
        let affected = self
            .conn
            .execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        // affected > 0 なら少なくとも1行削除された
        Ok(affected > 0)
    }

    /// 全てのフィードを取得する
    ///
    /// # 戻り値
    ///
    /// フィードのベクター（ID順）
    ///
    /// # イテレータとクロージャ
    ///
    /// `query_map` はSQLの結果セットをイテレータとして処理します。
    /// 引数のクロージャ `|row| { ... }` は各行に対して実行され、
    /// `Feed` 構造体に変換します。
    ///
    /// クロージャは `|| {}` で定義する無名関数で、
    /// 周囲の変数をキャプチャできます。
    pub fn get_feeds(&self) -> Result<Vec<Feed>> {
        // プリペアドステートメントを作成
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, description, created_at, updated_at FROM feeds ORDER BY id",
        )?;

        // クエリ実行と結果のマッピング
        let feeds = stmt
            .query_map([], |row| {
                // 各行を Feed 構造体に変換
                // row.get(index) でカラムの値を取得
                Ok(Feed {
                    id: row.get(0)?,          // 1列目: id
                    url: row.get(1)?,         // 2列目: url
                    title: row.get(2)?,       // 3列目: title
                    description: row.get(3)?, // 4列目: description (NULL可)
                    created_at: parse_datetime(row.get::<_, String>(4)?),
                    updated_at: parse_datetime(row.get::<_, String>(5)?),
                })
            })?
            // イテレータをVecに収集
            // `collect::<Result<Vec<_>, _>>()` はエラー処理付きの収集
            .collect::<Result<Vec<_>, _>>()?;

        Ok(feeds)
    }

    /// IDでフィードを取得する
    ///
    /// # 引数
    ///
    /// * `id` - 取得するフィードのID
    ///
    /// # 戻り値
    ///
    /// - `Ok(Some(feed))`: フィードが見つかった
    /// - `Ok(None)`: フィードが見つからなかった
    /// - `Err(...)`: データベースエラー
    ///
    /// # Option型について
    ///
    /// `Option<T>` は「値が存在するかもしれない」ことを型で表現します。
    /// これにより、nullチェックを忘れるバグを防ぎます。
    #[allow(dead_code)] // 現在未使用だが将来使用予定
    pub fn get_feed(&self, id: i64) -> Result<Option<Feed>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, description, created_at, updated_at FROM feeds WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        // 最初の行があれば取得
        if let Some(row) = rows.next()? {
            Ok(Some(Feed {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                created_at: parse_datetime(row.get::<_, String>(4)?),
                updated_at: parse_datetime(row.get::<_, String>(5)?),
            }))
        } else {
            Ok(None)
        }
    }

    // =========================================================================
    // Article 関連のCRUD操作
    // =========================================================================

    /// 新しい記事を追加する（既存の場合は無視）
    ///
    /// # 引数
    ///
    /// * `article` - 追加する記事
    ///
    /// # 戻り値
    ///
    /// - `Ok(Some(id))`: 新規挿入された場合、そのID
    /// - `Ok(None)`: 既に存在していた場合（重複URL）
    ///
    /// # INSERT OR IGNORE
    ///
    /// SQLite の `INSERT OR IGNORE` は、ユニーク制約違反時に
    /// エラーではなく単に無視します。これにより、
    /// 同じ記事を重複して登録することを防ぎます。
    pub fn add_article(&self, article: &Article) -> Result<Option<i64>> {
        let result = self.conn.execute(
            "INSERT OR IGNORE INTO articles (feed_id, title, url, content, published_at, is_read, is_favorite, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                article.feed_id,
                article.title,
                article.url,
                article.content,
                // Option<DateTime> を Option<String> に変換
                article.published_at.map(|dt| dt.to_rfc3339()),
                article.is_read as i32, // bool を整数に変換（SQLiteはboolがない）
                article.is_favorite as i32, // bool を整数に変換
                article.created_at.to_rfc3339(),
            ],
        )?;

        if result > 0 {
            Ok(Some(self.conn.last_insert_rowid()))
        } else {
            Ok(None) // 記事は既に存在していた
        }
    }

    /// 記事を取得する（フィルタ付き）
    ///
    /// # 引数
    ///
    /// * `unread_only` - true なら未読記事のみ取得
    /// * `limit` - 取得する最大件数
    /// * `filter` - キーワードフィルタ（カンマ区切りで複数指定可能、OR条件）
    /// * `feed_id` - 特定のフィードIDでフィルタ（None の場合は全フィード）
    ///
    /// # 戻り値
    ///
    /// 記事のベクター（公開日時の降順）
    ///
    /// # キーワードフィルタについて
    ///
    /// `filter` パラメータにキーワードを指定すると、タイトルまたは本文に
    /// そのキーワードを含む記事のみを取得します。
    /// 複数のキーワードをカンマで区切って指定すると、いずれかを含む記事を取得します（OR条件）。
    pub fn get_articles(
        &self,
        unread_only: bool,
        limit: usize,
        filter: Option<&str>,
        feed_id: Option<i64>,
    ) -> Result<Vec<Article>> {
        // ベースとなるSQLクエリ
        let mut sql = String::from(
            "SELECT id, feed_id, title, url, content, published_at, is_read, is_favorite, created_at
             FROM articles"
        );

        // WHERE句の条件を格納するベクター
        let mut conditions = Vec::new();

        // 未読フィルタ
        if unread_only {
            conditions.push("is_read = 0".to_string());
        }

        // フィードIDフィルタ
        if feed_id.is_some() {
            conditions.push("feed_id = ?".to_string());
        }

        // キーワードフィルタ
        // filter が Some の場合、キーワードで検索条件を追加
        if let Some(filter_str) = filter {
            // カンマで分割して各キーワードに対する条件を作成
            // 例: "rust,cargo" -> ["rust", "cargo"]
            let keywords: Vec<&str> = filter_str.split(',').map(|s| s.trim()).collect();

            if !keywords.is_empty() {
                // 各キーワードに対してOR条件を作成
                // (title LIKE '%keyword1%' OR content LIKE '%keyword1%') OR (title LIKE '%keyword2%' OR content LIKE '%keyword2%')
                let keyword_conditions: Vec<String> = keywords
                    .iter()
                    .map(|_| "(title LIKE ? OR content LIKE ?)".to_string())
                    .collect();

                conditions.push(format!("({})", keyword_conditions.join(" OR ")));
            }
        }

        // WHERE句を追加
        if !conditions.is_empty() {
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }

        // ORDER BY と LIMIT を追加
        sql.push_str(" ORDER BY published_at DESC, created_at DESC LIMIT ?");

        // パラメータを準備
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // フィードIDパラメータを追加
        if let Some(id) = feed_id {
            params.push(Box::new(id));
        }

        // キーワードフィルタのパラメータを追加
        if let Some(filter_str) = filter {
            let keywords: Vec<&str> = filter_str.split(',').map(|s| s.trim()).collect();
            for keyword in keywords {
                let pattern = format!("%{}%", keyword);
                // タイトル用とコンテンツ用で2回追加
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        // limit パラメータを追加
        params.push(Box::new(limit as i64));

        // プリペアドステートメントを準備
        let mut stmt = self.conn.prepare(&sql)?;

        // パラメータをバインドして実行
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let articles = stmt
            .query_map(&params_refs[..], |row| {
                Ok(Article {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    content: row.get(4)?,
                    published_at: row.get::<_, Option<String>>(5)?.map(parse_datetime),
                    is_read: row.get::<_, i32>(6)? != 0,
                    is_favorite: row.get::<_, i32>(7)? != 0,
                    created_at: parse_datetime(row.get::<_, String>(8)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// 記事を既読としてマークする
    ///
    /// # 引数
    ///
    /// * `id` - 既読にする記事のID
    ///
    /// # 戻り値
    ///
    /// - `Ok(true)`: 更新成功
    /// - `Ok(false)`: 該当する記事が存在しなかった
    pub fn mark_as_read(&self, id: i64) -> Result<bool> {
        let affected = self
            .conn
            .execute("UPDATE articles SET is_read = 1 WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// 記事を一括で既読にする（フィルタ付き）
    ///
    /// # 引数
    ///
    /// * `feed_id` - 特定のフィードのみ対象（None の場合は全フィード）
    /// * `before_date` - 指定日時以前の記事のみ対象（RFC3339形式、None の場合は全期間）
    ///
    /// # 戻り値
    ///
    /// 更新された記事の件数
    pub fn mark_all_read_with_filter(
        &self,
        feed_id: Option<i64>,
        before_date: Option<&str>,
    ) -> Result<usize> {
        let affected = match (feed_id, before_date) {
            (None, None) => {
                // 全記事を既読に
                self.conn.execute("UPDATE articles SET is_read = 1", [])?
            }
            (Some(id), None) => {
                // 特定フィードの記事を既読に
                self.conn.execute(
                    "UPDATE articles SET is_read = 1 WHERE feed_id = ?1",
                    params![id],
                )?
            }
            (None, Some(date)) => {
                // 指定日付以前の記事を既読に
                self.conn.execute(
                    "UPDATE articles SET is_read = 1 WHERE published_at < ?1",
                    params![date],
                )?
            }
            (Some(id), Some(date)) => {
                // 特定フィードかつ指定日付以前の記事を既読に
                self.conn.execute(
                    "UPDATE articles SET is_read = 1 WHERE feed_id = ?1 AND published_at < ?2",
                    params![id, date],
                )?
            }
        };

        Ok(affected)
    }

    /// 記事を未読に戻す
    ///
    /// # 引数
    ///
    /// * `id` - 未読にする記事のID
    ///
    /// # 戻り値
    ///
    /// - `Ok(true)`: 更新成功
    /// - `Ok(false)`: 該当する記事が存在しなかった
    pub fn mark_as_unread(&self, id: i64) -> Result<bool> {
        let affected = self
            .conn
            .execute("UPDATE articles SET is_read = 0 WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// フィード単位で記事を未読に戻す
    ///
    /// # 引数
    ///
    /// * `feed_id` - 対象フィードのID
    ///
    /// # 戻り値
    ///
    /// 更新された記事の件数
    pub fn mark_all_unread_by_feed(&self, feed_id: i64) -> Result<usize> {
        let affected = self.conn.execute(
            "UPDATE articles SET is_read = 0 WHERE feed_id = ?1",
            params![feed_id],
        )?;
        Ok(affected)
    }

    /// 全記事を未読に戻す
    ///
    /// # 戻り値
    ///
    /// 更新された記事の件数
    pub fn mark_all_unread(&self) -> Result<usize> {
        let affected = self.conn.execute("UPDATE articles SET is_read = 0", [])?;
        Ok(affected)
    }

    /// 記事の既読/未読状態を反転する
    ///
    /// # 引数
    ///
    /// * `id` - 対象記事のID
    ///
    /// # 戻り値
    ///
    /// - `Ok(true)`: 更新成功
    /// - `Ok(false)`: 該当する記事が存在しなかった
    pub fn toggle_read_status(&self, id: i64) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE articles SET is_read = CASE WHEN is_read = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            params![id],
        )?;
        Ok(affected > 0)
    }

    /// 記事をお気に入りに追加する
    ///
    /// # 引数
    ///
    /// * `id` - お気に入りにする記事のID
    ///
    /// # 戻り値
    ///
    /// - `Ok(true)`: 更新成功
    /// - `Ok(false)`: 該当する記事が存在しなかった
    pub fn add_favorite(&self, id: i64) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE articles SET is_favorite = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(affected > 0)
    }

    /// 記事をお気に入りから削除する
    ///
    /// # 引数
    ///
    /// * `id` - お気に入りから削除する記事のID
    ///
    /// # 戻り値
    ///
    /// - `Ok(true)`: 更新成功
    /// - `Ok(false)`: 該当する記事が存在しなかった
    pub fn remove_favorite(&self, id: i64) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE articles SET is_favorite = 0 WHERE id = ?1",
            params![id],
        )?;
        Ok(affected > 0)
    }

    /// お気に入り記事を取得する
    ///
    /// # 引数
    ///
    /// * `limit` - 取得する最大件数
    ///
    /// # 戻り値
    ///
    /// お気に入り記事のベクター（公開日時の降順）
    pub fn get_favorite_articles(&self, limit: usize) -> Result<Vec<Article>> {
        let sql = "SELECT id, feed_id, title, url, content, published_at, is_read, is_favorite, created_at
             FROM articles WHERE is_favorite = 1
             ORDER BY published_at DESC, created_at DESC LIMIT ?1";

        let mut stmt = self.conn.prepare(sql)?;

        let articles = stmt
            .query_map(params![limit as i64], |row| {
                Ok(Article {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    content: row.get(4)?,
                    published_at: row.get::<_, Option<String>>(5)?.map(parse_datetime),
                    is_read: row.get::<_, i32>(6)? != 0,
                    is_favorite: row.get::<_, i32>(7)? != 0,
                    created_at: parse_datetime(row.get::<_, String>(8)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }
}

// =============================================================================
// ヘルパー関数
// =============================================================================

/// RFC3339形式の文字列を DateTime<Utc> にパースする
///
/// # 引数
///
/// * `s` - RFC3339形式の日時文字列（例: "2024-01-01T12:00:00Z"）
///
/// # 戻り値
///
/// パースされた DateTime。パース失敗時は現在時刻を返す。
///
/// # エラー処理
///
/// `unwrap_or_else` はエラー時にクロージャを実行してデフォルト値を返します。
/// これにより、不正な日時形式でもプログラムがパニックしません。
fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc)) // タイムゾーンをUTCに変換
        .unwrap_or_else(|_| Utc::now()) // パース失敗時は現在時刻
}
