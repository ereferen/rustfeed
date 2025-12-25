use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// RSS Feed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Article from an RSS feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: i64,
    pub feed_id: i64,
    pub title: String,
    pub url: Option<String>,
    pub content: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

impl Feed {
    pub fn new(url: String, title: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            url,
            title,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Article {
    pub fn new(
        feed_id: i64,
        title: String,
        url: Option<String>,
        content: Option<String>,
        published_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: 0, // Will be set by database
            feed_id,
            title,
            url,
            content,
            published_at,
            is_read: false,
            created_at: Utc::now(),
        }
    }
}
