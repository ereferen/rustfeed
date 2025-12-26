//! # rustfeed-tui - TUI RSS Reader
//!
//! `rustfeed-tui` は Rust で書かれたターミナルUI RSS リーダーです。
//!
//! ## 主な機能
//!
//! - インタラクティブなフィード・記事閲覧
//! - キーボードナビゲーション
//! - 2ペインレイアウト（フィード一覧 + 記事一覧）
//!
//! ## 使用方法
//!
//! ```bash
//! rustfeed-tui
//! ```
//!
//! ## キーバインド
//!
//! - `j`/`k` または `↓`/`↑`: 上下移動
//! - `Enter`: 選択
//! - `r`: 既読/未読切り替え
//! - `f`: お気に入り切り替え
//! - `R`: フィード更新
//! - `q`: 終了

mod app;
mod event;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use rustfeed_core::{config::AppConfig, db::Database};
use std::io;

/// アプリケーションのエントリーポイント
#[tokio::main]
async fn main() -> Result<()> {
    // データベースを初期化
    let db = Database::new()?;
    db.init()?;

    // 設定を読み込み
    let config = AppConfig::load()?;

    // アプリケーション状態を初期化
    let mut app = App::new(db, config)?;

    // ターミナルをセットアップ
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // アプリケーションを実行
    let result = app.run(&mut terminal).await;

    // ターミナルをリストア
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // エラーがあれば表示
    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }

    result
}
