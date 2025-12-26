//! # CLIコマンド実装モジュール
//!
//! このモジュールは、rustfeed CLI の各サブコマンドの実装を提供します。

use anyhow::{Context, Result};
use colored::Colorize;
use rustfeed_core::{db::Database, feed, Article};

// =============================================================================
// フィード管理コマンド
// =============================================================================

/// 新しいRSSフィードを追加する
pub async fn add_feed(db: &Database, url: &str, name: Option<&str>) -> Result<()> {
    println!("{} {}", "Fetching feed:".blue(), url);

    let (mut feed_info, _articles) = feed::fetch_feed(url)
        .await
        .with_context(|| format!("Failed to fetch feed from {}", url))?;

    if let Some(custom_name) = name {
        feed_info.title = custom_name.to_string();
    }

    let id = db.add_feed(&feed_info)?;

    println!(
        "{} {} (ID: {})",
        "Added feed:".green(),
        feed_info.title.bold(),
        id
    );

    Ok(())
}

/// RSSフィードを削除する
pub fn remove_feed(db: &Database, id: i64) -> Result<()> {
    if db.remove_feed(id)? {
        println!("{} {}", "Removed feed with ID:".green(), id);
    } else {
        println!("{} {}", "Feed not found with ID:".yellow(), id);
    }
    Ok(())
}

/// 登録済みの全フィードを一覧表示する
pub fn list_feeds(db: &Database, category: Option<&str>) -> Result<()> {
    let feeds = db.get_feeds(category)?;

    if feeds.is_empty() {
        println!("{}", "No feeds registered yet.".yellow());
        println!("Use 'rustfeed add <url>' to add a feed.");
        return Ok(());
    }

    println!("{}", "Registered Feeds:".bold().underline());
    println!();

    for feed in feeds {
        let display_name = feed.custom_name.as_deref().unwrap_or(&feed.title);

        let mut info_parts = vec![
            format!("[{}]", feed.id).cyan().to_string(),
            display_name.bold().to_string(),
        ];

        if let Some(category) = &feed.category {
            info_parts.push(format!("[{}]", category).green().to_string());
        }

        if feed.priority != 0 {
            info_parts.push(
                format!("(priority: {})", feed.priority)
                    .magenta()
                    .to_string(),
            );
        }

        info_parts.push(format!("({})", feed.url).dimmed().to_string());

        println!("  {}", info_parts.join(" "));

        if let Some(desc) = &feed.description {
            let short_desc: String = desc.chars().take(80).collect();
            println!("      {}", short_desc.dimmed());
        }
    }

    Ok(())
}

/// フィードの名前を変更する
pub fn rename_feed(db: &Database, feed_id: i64, name: &str) -> Result<()> {
    let custom_name = if name.is_empty() { None } else { Some(name) };

    db.rename_feed(feed_id, custom_name)?;

    if custom_name.is_some() {
        println!(
            "{} {} {}",
            "Feed".green(),
            feed_id,
            format!("renamed to '{}'", name).green().bold()
        );
    } else {
        println!(
            "{} {} {}",
            "Feed".green(),
            feed_id,
            "custom name cleared (using original title)".green().bold()
        );
    }

    Ok(())
}

/// フィードのURLを更新する
pub fn update_feed_url(db: &Database, feed_id: i64, new_url: &str) -> Result<()> {
    db.update_feed_url(feed_id, new_url)?;

    println!(
        "{} {} {} {}",
        "Feed".green(),
        feed_id,
        "URL updated to".green().bold(),
        new_url.cyan()
    );

    Ok(())
}

/// フィードのカテゴリを設定する
pub fn set_feed_category(db: &Database, feed_id: i64, category: &str) -> Result<()> {
    let cat = if category.is_empty() {
        None
    } else {
        Some(category)
    };

    db.set_feed_category(feed_id, cat)?;

    if cat.is_some() {
        println!(
            "{} {} {} {}",
            "Feed".green(),
            feed_id,
            "category set to".green().bold(),
            format!("[{}]", category).green()
        );
    } else {
        println!(
            "{} {} {}",
            "Feed".green(),
            feed_id,
            "category cleared".green().bold()
        );
    }

    Ok(())
}

/// フィードの優先順位を設定する
pub fn set_feed_priority(db: &Database, feed_id: i64, priority: i64) -> Result<()> {
    db.set_feed_priority(feed_id, priority)?;

    println!(
        "{} {} {} {}",
        "Feed".green(),
        feed_id,
        "priority set to".green().bold(),
        priority.to_string().magenta()
    );

    Ok(())
}

/// フィードの詳細情報を表示する
pub fn show_feed_info(db: &Database, feed_id: i64) -> Result<()> {
    let feed = db.get_feed(feed_id)?;

    if let Some(feed) = feed {
        println!("{}", "Feed Information:".bold().underline());
        println!();
        println!("  {}: {}", "ID".cyan(), feed.id);
        println!("  {}: {}", "Title".cyan(), feed.title.bold());

        if let Some(custom_name) = &feed.custom_name {
            println!("  {}: {}", "Custom Name".cyan(), custom_name.bold().green());
        }

        println!("  {}: {}", "URL".cyan(), feed.url);

        if let Some(description) = &feed.description {
            println!("  {}: {}", "Description".cyan(), description);
        }

        if let Some(category) = &feed.category {
            println!(
                "  {}: {}",
                "Category".cyan(),
                format!("[{}]", category).green()
            );
        }

        println!(
            "  {}: {}",
            "Priority".cyan(),
            feed.priority.to_string().magenta()
        );

        println!(
            "  {}: {}",
            "Created".cyan(),
            feed.created_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!(
            "  {}: {}",
            "Updated".cyan(),
            feed.updated_at.format("%Y-%m-%d %H:%M:%S")
        );

        // 記事数を取得して表示
        let (total, unread) = db.get_article_counts(feed_id)?;
        println!(
            "  {}: {} (unread: {})",
            "Articles".cyan(),
            total,
            unread.to_string().yellow()
        );
    } else {
        println!("{} {}", "Feed not found with ID:".yellow(), feed_id);
    }

    Ok(())
}

// =============================================================================
// 記事関連コマンド
// =============================================================================

/// 全フィードから新しい記事を取得する
pub async fn fetch_feeds(db: &Database) -> Result<()> {
    let feeds = db.get_feeds(None)?;

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

                println!("{} ({} new)", "OK".green(), new_count.to_string().cyan());

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

/// 記事を一覧表示する
#[allow(clippy::too_many_arguments)]
pub fn show_articles(
    db: &Database,
    unread_only: bool,
    limit: usize,
    filter: Option<&str>,
    feed_id: Option<i64>,
    disabled_feeds: &[i64],
    after: Option<&str>,
    before: Option<&str>,
) -> Result<()> {
    let mut articles = db.get_articles(unread_only, limit, filter, feed_id)?;

    // disabled_feedsでフィルタリング
    if feed_id.is_none() && !disabled_feeds.is_empty() {
        articles.retain(|article| !disabled_feeds.contains(&article.feed_id));
    }

    // 日付範囲でフィルタリング
    if let Some(after_date) = after {
        articles.retain(|article| {
            article
                .published_at
                .map(|dt| dt.format("%Y-%m-%d").to_string().as_str() >= after_date)
                .unwrap_or(false)
        });
    }

    if let Some(before_date) = before {
        articles.retain(|article| {
            article
                .published_at
                .map(|dt| dt.format("%Y-%m-%d").to_string().as_str() <= before_date)
                .unwrap_or(false)
        });
    }

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

/// 記事を既読としてマークする
pub fn mark_as_read(db: &Database, id: i64) -> Result<()> {
    if db.mark_as_read(id)? {
        println!("{} {}", "Marked as read:".green(), id);
    } else {
        println!("{} {}", "Article not found with ID:".yellow(), id);
    }
    Ok(())
}

/// 記事をお気に入りに追加する
pub fn add_favorite(db: &Database, id: i64) -> Result<()> {
    if db.add_favorite(id)? {
        println!("{} {}", "Added to favorites:".green(), id);
    } else {
        println!("{} {}", "Article not found with ID:".yellow(), id);
    }
    Ok(())
}

/// 記事をお気に入りから削除する
pub fn remove_favorite(db: &Database, id: i64) -> Result<()> {
    if db.remove_favorite(id)? {
        println!("{} {}", "Removed from favorites:".green(), id);
    } else {
        println!("{} {}", "Article not found with ID:".yellow(), id);
    }
    Ok(())
}

/// お気に入り記事を一覧表示する
pub fn show_favorites(db: &Database, limit: usize) -> Result<()> {
    let articles = db.get_favorite_articles(limit)?;

    if articles.is_empty() {
        println!("{}", "No favorite articles.".yellow());
        println!("Use 'rustfeed favorite <id>' to add articles to favorites.");
        return Ok(());
    }

    println!("{}", "Favorite Articles:".bold().underline());
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

/// 記事をエクスポートする
pub fn export_articles(
    db: &Database,
    format: &str,
    favorites: bool,
    unread: bool,
    limit: Option<usize>,
) -> Result<()> {
    let articles = if favorites {
        let limit_val = limit.unwrap_or(usize::MAX);
        db.get_favorite_articles(limit_val)?
    } else {
        let limit_val = limit.unwrap_or(usize::MAX);
        db.get_articles(unread, limit_val, None, None)?
    };

    if articles.is_empty() {
        eprintln!("{}", "No articles to export.".yellow());
        return Ok(());
    }

    match format.to_lowercase().as_str() {
        "json" => export_as_json(&articles)?,
        "markdown" | "md" => export_as_markdown(&articles)?,
        _ => {
            anyhow::bail!(
                "Unsupported format: '{}'. Use 'json' or 'markdown'.",
                format
            );
        }
    }

    Ok(())
}

fn export_as_json(articles: &[Article]) -> Result<()> {
    let json =
        serde_json::to_string_pretty(articles).context("Failed to serialize articles to JSON")?;

    println!("{}", json);

    Ok(())
}

fn export_as_markdown(articles: &[Article]) -> Result<()> {
    println!("# Exported Articles\n");
    println!("Total: {} articles\n", articles.len());
    println!("---\n");

    for (index, article) in articles.iter().enumerate() {
        println!("## {}. {}\n", index + 1, article.title);

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

        if let Some(ref content) = article.content {
            println!("### Content\n");
            println!("{}\n", content);
        }

        if index < articles.len() - 1 {
            println!("---\n");
        }
    }

    Ok(())
}

// =============================================================================
// 既読管理コマンド
// =============================================================================

/// 記事を一括で既読にする
pub fn mark_all_read(db: &Database, feed_id: Option<i64>, before_date: Option<&str>) -> Result<()> {
    let count = db.mark_all_read_with_filter(feed_id, before_date)?;

    if count == 0 {
        println!("{}", "No articles matched the criteria.".yellow());
    } else {
        println!(
            "{} {}",
            format!("Marked {} article(s) as read.", count).green(),
            "✓".green().bold()
        );
    }

    Ok(())
}

/// 記事を未読に戻す
pub fn mark_unread(db: &Database, id: Option<i64>, feed_id: Option<i64>, all: bool) -> Result<()> {
    let options_count = [id.is_some(), feed_id.is_some(), all]
        .iter()
        .filter(|&&x| x)
        .count();

    if options_count == 0 {
        anyhow::bail!("Please specify either an article ID, --feed, or --all");
    }

    if options_count > 1 {
        anyhow::bail!("Please specify only one of: article ID, --feed, or --all");
    }

    let count = if let Some(article_id) = id {
        let success = db.mark_as_unread(article_id)?;
        if !success {
            anyhow::bail!("Article not found with ID: {}", article_id);
        }
        1
    } else if let Some(feed) = feed_id {
        db.mark_all_unread_by_feed(feed)?
    } else {
        db.mark_all_unread()?
    };

    if count == 0 {
        println!("{}", "No articles to mark as unread.".yellow());
    } else {
        println!(
            "{} {}",
            format!("Marked {} article(s) as unread.", count).green(),
            "✓".green().bold()
        );
    }

    Ok(())
}

/// 記事の既読/未読状態を反転する
pub fn toggle_read(db: &Database, id: i64) -> Result<()> {
    let success = db.toggle_read_status(id)?;

    if !success {
        anyhow::bail!("Article not found with ID: {}", id);
    }

    println!(
        "{} {}",
        format!("Toggled read status for article {}.", id).green(),
        "✓".green().bold()
    );

    Ok(())
}
