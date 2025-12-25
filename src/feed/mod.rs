use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use feed_rs::parser;

use crate::models::{Article, Feed};

/// Fetch and parse an RSS/Atom feed from URL
pub async fn fetch_feed(url: &str) -> Result<(Feed, Vec<Article>)> {
    // Fetch the feed content
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to fetch feed from {}", url))?;

    let bytes = response
        .bytes()
        .await
        .with_context(|| "Failed to read response body")?;

    // Parse the feed
    let parsed = parser::parse(&bytes[..]).with_context(|| "Failed to parse feed")?;

    // Create Feed struct
    let title = parsed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let description = parsed.description.map(|d| d.content);

    let feed = Feed::new(url.to_string(), title, description);

    // Create Article structs
    let articles: Vec<Article> = parsed
        .entries
        .into_iter()
        .map(|entry| {
            let title = entry
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "Untitled".to_string());

            let url = entry.links.first().map(|l| l.href.clone());

            let content = entry
                .summary
                .map(|s| s.content)
                .or_else(|| entry.content.and_then(|c| c.body));

            let published_at: Option<DateTime<Utc>> = entry
                .published
                .or(entry.updated)
                .map(|dt| dt.with_timezone(&Utc));

            Article::new(0, title, url, content, published_at) // feed_id will be set later
        })
        .collect();

    Ok((feed, articles))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_feed() {
        // Test with a known RSS feed
        let result = fetch_feed("https://blog.rust-lang.org/feed.xml").await;
        assert!(result.is_ok(), "Should successfully fetch Rust blog feed");

        if let Ok((feed, articles)) = result {
            assert!(!feed.title.is_empty(), "Feed should have a title");
            assert!(!articles.is_empty(), "Feed should have articles");
        }
    }
}
