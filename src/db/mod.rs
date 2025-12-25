use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::models::{Article, Feed};

/// Database handler for rustfeed
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;

        Ok(Self { conn })
    }

    /// Get the database file path
    fn get_db_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".rustfeed").join("rustfeed.db"))
    }

    /// Initialize database tables
    pub fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

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

        // Create index for faster queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_is_read ON articles(is_read)",
            [],
        )?;

        Ok(())
    }

    /// Add a new feed
    pub fn add_feed(&self, feed: &Feed) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feeds (url, title, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                feed.url,
                feed.title,
                feed.description,
                feed.created_at.to_rfc3339(),
                feed.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Remove a feed by ID
    pub fn remove_feed(&self, id: i64) -> Result<bool> {
        let affected = self.conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// Get all feeds
    pub fn get_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, description, created_at, updated_at FROM feeds ORDER BY id",
        )?;

        let feeds = stmt
            .query_map([], |row| {
                Ok(Feed {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    created_at: parse_datetime(row.get::<_, String>(4)?),
                    updated_at: parse_datetime(row.get::<_, String>(5)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(feeds)
    }

    /// Get a feed by ID
    pub fn get_feed(&self, id: i64) -> Result<Option<Feed>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, description, created_at, updated_at FROM feeds WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

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

    /// Add a new article (or ignore if already exists)
    pub fn add_article(&self, article: &Article) -> Result<Option<i64>> {
        let result = self.conn.execute(
            "INSERT OR IGNORE INTO articles (feed_id, title, url, content, published_at, is_read, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                article.feed_id,
                article.title,
                article.url,
                article.content,
                article.published_at.map(|dt| dt.to_rfc3339()),
                article.is_read as i32,
                article.created_at.to_rfc3339(),
            ],
        )?;

        if result > 0 {
            Ok(Some(self.conn.last_insert_rowid()))
        } else {
            Ok(None) // Article already existed
        }
    }

    /// Get articles with optional filters
    pub fn get_articles(&self, unread_only: bool, limit: usize) -> Result<Vec<Article>> {
        let sql = if unread_only {
            "SELECT id, feed_id, title, url, content, published_at, is_read, created_at
             FROM articles WHERE is_read = 0
             ORDER BY published_at DESC, created_at DESC LIMIT ?1"
        } else {
            "SELECT id, feed_id, title, url, content, published_at, is_read, created_at
             FROM articles
             ORDER BY published_at DESC, created_at DESC LIMIT ?1"
        };

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
                    created_at: parse_datetime(row.get::<_, String>(7)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Mark an article as read
    pub fn mark_as_read(&self, id: i64) -> Result<bool> {
        let affected = self
            .conn
            .execute("UPDATE articles SET is_read = 1 WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }
}

/// Parse RFC3339 datetime string
fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
