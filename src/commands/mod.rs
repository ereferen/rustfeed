use anyhow::{Context, Result};
use colored::Colorize;

use crate::db::Database;
use crate::feed;

/// Add a new RSS feed
pub async fn add_feed(db: &Database, url: &str, name: Option<&str>) -> Result<()> {
    println!("{} {}", "Fetching feed:".blue(), url);

    // Fetch and parse the feed
    let (mut feed_info, _articles) = feed::fetch_feed(url)
        .await
        .with_context(|| format!("Failed to fetch feed from {}", url))?;

    // Use custom name if provided
    if let Some(custom_name) = name {
        feed_info.title = custom_name.to_string();
    }

    // Save to database
    let id = db.add_feed(&feed_info)?;

    println!(
        "{} {} (ID: {})",
        "Added feed:".green(),
        feed_info.title.bold(),
        id
    );

    Ok(())
}

/// Remove an RSS feed
pub fn remove_feed(db: &Database, id: i64) -> Result<()> {
    if db.remove_feed(id)? {
        println!("{} {}", "Removed feed with ID:".green(), id);
    } else {
        println!("{} {}", "Feed not found with ID:".yellow(), id);
    }
    Ok(())
}

/// List all registered feeds
pub fn list_feeds(db: &Database) -> Result<()> {
    let feeds = db.get_feeds()?;

    if feeds.is_empty() {
        println!("{}", "No feeds registered yet.".yellow());
        println!("Use 'rustfeed add <url>' to add a feed.");
        return Ok(());
    }

    println!("{}", "Registered Feeds:".bold().underline());
    println!();

    for feed in feeds {
        println!(
            "  {} {} {}",
            format!("[{}]", feed.id).cyan(),
            feed.title.bold(),
            format!("({})", feed.url).dimmed()
        );
        if let Some(desc) = &feed.description {
            let short_desc: String = desc.chars().take(80).collect();
            println!("      {}", short_desc.dimmed());
        }
    }

    Ok(())
}

/// Fetch new articles from all feeds
pub async fn fetch_feeds(db: &Database) -> Result<()> {
    let feeds = db.get_feeds()?;

    if feeds.is_empty() {
        println!("{}", "No feeds registered yet.".yellow());
        return Ok(());
    }

    println!("{}", "Fetching articles from all feeds...".blue());
    println!();

    let mut total_new = 0;

    for stored_feed in feeds {
        print!("  {} {}... ", "Fetching".dimmed(), stored_feed.title);

        match feed::fetch_feed(&stored_feed.url).await {
            Ok((_feed_info, articles)) => {
                let mut new_count = 0;
                for mut article in articles {
                    article.feed_id = stored_feed.id;
                    if db.add_article(&article)?.is_some() {
                        new_count += 1;
                    }
                }
                println!(
                    "{} ({} new)",
                    "OK".green(),
                    new_count.to_string().cyan()
                );
                total_new += new_count;
            }
            Err(e) => {
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

/// Show articles
pub fn show_articles(db: &Database, unread_only: bool, limit: usize) -> Result<()> {
    let articles = db.get_articles(unread_only, limit)?;

    if articles.is_empty() {
        if unread_only {
            println!("{}", "No unread articles.".green());
        } else {
            println!("{}", "No articles found.".yellow());
            println!("Use 'rustfeed fetch' to get articles from your feeds.");
        }
        return Ok(());
    }

    let header = if unread_only {
        "Unread Articles:"
    } else {
        "Articles:"
    };
    println!("{}", header.bold().underline());
    println!();

    for article in articles {
        let read_marker = if article.is_read {
            "[x]".dimmed()
        } else {
            "[*]".cyan()
        };

        let date = article
            .published_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "----------".to_string());

        println!(
            "  {} {} {} {}",
            read_marker,
            format!("[{}]", article.id).dimmed(),
            date.dimmed(),
            article.title.bold()
        );

        if let Some(url) = &article.url {
            println!("      {}", url.dimmed());
        }
    }

    Ok(())
}

/// Mark an article as read
pub fn mark_as_read(db: &Database, id: i64) -> Result<()> {
    if db.mark_as_read(id)? {
        println!("{} {}", "Marked as read:".green(), id);
    } else {
        println!("{} {}", "Article not found with ID:".yellow(), id);
    }
    Ok(())
}
