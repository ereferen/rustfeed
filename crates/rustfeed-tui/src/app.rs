//! # アプリケーション状態管理
//!
//! TUIアプリケーションの状態とメインループを管理します。

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, Terminal};
use rustfeed_core::{config::AppConfig, db::Database, Article, Feed};
use std::time::Duration;

use crate::ui;

/// アプリケーションのフォーカス状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// フィード一覧にフォーカス
    Feeds,
    /// 記事一覧にフォーカス
    Articles,
}

/// アプリケーション状態
pub struct App {
    /// データベース接続
    pub db: Database,
    /// アプリケーション設定
    pub config: AppConfig,
    /// 終了フラグ
    pub should_quit: bool,
    /// 現在のフォーカス
    pub focus: Focus,
    /// フィード一覧
    pub feeds: Vec<Feed>,
    /// 選択中のフィードインデックス
    pub selected_feed: usize,
    /// 記事一覧
    pub articles: Vec<Article>,
    /// 選択中の記事インデックス
    pub selected_article: usize,
    /// ステータスメッセージ
    pub status_message: Option<String>,
}

impl App {
    /// 新しいアプリケーション状態を作成
    pub fn new(db: Database, config: AppConfig) -> Result<Self> {
        let feeds = db.get_feeds(None)?;
        let articles = if !feeds.is_empty() {
            db.get_articles(false, 50, None, Some(feeds[0].id))?
        } else {
            Vec::new()
        };

        Ok(Self {
            db,
            config,
            should_quit: false,
            focus: Focus::Feeds,
            feeds,
            selected_feed: 0,
            articles,
            selected_article: 0,
            status_message: None,
        })
    }

    /// メインループを実行
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // 画面を描画
            terminal.draw(|frame| ui::render(frame, self))?;

            // イベントをポーリング（100msタイムアウト）
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // KeyEventKind::Press のみ処理（リピートを防ぐ）
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code).await?;
                    }
                }
            }

            // 終了フラグが立っていたらループを抜ける
            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// キー入力を処理
    async fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            // 終了
            KeyCode::Char('q') => {
                self.should_quit = true;
            }

            // 上下移動
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
            }

            // 左右でフォーカス切り替え
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = Focus::Feeds;
                self.status_message = Some("Feeds".to_string());
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                if self.focus == Focus::Feeds && !self.feeds.is_empty() {
                    self.focus = Focus::Articles;
                    self.load_articles_for_selected_feed()?;
                    self.status_message = Some("Articles".to_string());
                }
            }

            // Tab でフォーカス切り替え
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Feeds => Focus::Articles,
                    Focus::Articles => Focus::Feeds,
                };
            }

            // 既読/未読トグル
            KeyCode::Char('r') => {
                if self.focus == Focus::Articles && !self.articles.is_empty() {
                    self.toggle_read()?;
                }
            }

            // お気に入りトグル
            KeyCode::Char('f') => {
                if self.focus == Focus::Articles && !self.articles.is_empty() {
                    self.toggle_favorite()?;
                }
            }

            // フィード更新
            KeyCode::Char('R') => {
                self.status_message = Some("Refreshing feeds...".to_string());
                // TODO: 非同期でフィード更新を実装
            }

            _ => {}
        }

        Ok(())
    }

    /// 上に移動
    fn move_up(&mut self) {
        match self.focus {
            Focus::Feeds => {
                if self.selected_feed > 0 {
                    self.selected_feed -= 1;
                    let _ = self.load_articles_for_selected_feed();
                }
            }
            Focus::Articles => {
                if self.selected_article > 0 {
                    self.selected_article -= 1;
                }
            }
        }
    }

    /// 下に移動
    fn move_down(&mut self) {
        match self.focus {
            Focus::Feeds => {
                if self.selected_feed < self.feeds.len().saturating_sub(1) {
                    self.selected_feed += 1;
                    let _ = self.load_articles_for_selected_feed();
                }
            }
            Focus::Articles => {
                if self.selected_article < self.articles.len().saturating_sub(1) {
                    self.selected_article += 1;
                }
            }
        }
    }

    /// 選択中のフィードの記事を読み込む
    fn load_articles_for_selected_feed(&mut self) -> Result<()> {
        if let Some(feed) = self.feeds.get(self.selected_feed) {
            self.articles = self.db.get_articles(false, 50, None, Some(feed.id))?;
            self.selected_article = 0;
        }
        Ok(())
    }

    /// 既読/未読をトグル
    fn toggle_read(&mut self) -> Result<()> {
        if let Some(article) = self.articles.get(self.selected_article) {
            self.db.toggle_read_status(article.id)?;
            self.load_articles_for_selected_feed()?;
            self.status_message = Some("Toggled read status".to_string());
        }
        Ok(())
    }

    /// お気に入りをトグル
    fn toggle_favorite(&mut self) -> Result<()> {
        if let Some(article) = self.articles.get(self.selected_article) {
            if article.is_favorite {
                self.db.remove_favorite(article.id)?;
                self.status_message = Some("Removed from favorites".to_string());
            } else {
                self.db.add_favorite(article.id)?;
                self.status_message = Some("Added to favorites".to_string());
            }
            self.load_articles_for_selected_feed()?;
        }
        Ok(())
    }
}
