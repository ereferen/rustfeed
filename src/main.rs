mod commands;
mod db;
mod feed;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustfeed")]
#[command(author = "ereferen")]
#[command(version = "0.1.0")]
#[command(about = "A CLI RSS reader written in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new RSS feed
    Add {
        /// The URL of the RSS feed
        url: String,
        /// Optional name for the feed
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Remove an RSS feed
    Remove {
        /// The ID of the feed to remove
        id: i64,
    },
    /// List all registered feeds
    List,
    /// Fetch new articles from all feeds
    Fetch,
    /// Show articles
    Articles {
        /// Show only unread articles
        #[arg(short, long)]
        unread: bool,
        /// Limit the number of articles
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Mark an article as read
    Read {
        /// The ID of the article
        id: i64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize database
    let db = db::Database::new()?;
    db.init()?;

    match cli.command {
        Commands::Add { url, name } => {
            commands::add_feed(&db, &url, name.as_deref()).await?;
        }
        Commands::Remove { id } => {
            commands::remove_feed(&db, id)?;
        }
        Commands::List => {
            commands::list_feeds(&db)?;
        }
        Commands::Fetch => {
            commands::fetch_feeds(&db).await?;
        }
        Commands::Articles { unread, limit } => {
            commands::show_articles(&db, unread, limit)?;
        }
        Commands::Read { id } => {
            commands::mark_as_read(&db, id)?;
        }
    }

    Ok(())
}
