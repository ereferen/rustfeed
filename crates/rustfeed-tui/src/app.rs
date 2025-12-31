//! # アプリケーション状態管理
//!
//! TUIアプリケーションの状態とメインループを管理します。

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
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
    /// フィードリストの表示可能行数（スクロール計算用）
    pub feeds_list_height: u16,
    /// 記事リストの表示可能行数（スクロール計算用）
    pub articles_list_height: u16,
    /// プレビューモードが有効かどうか
    pub show_preview: bool,
    /// プレビューのスクロール位置
    pub preview_scroll: usize,
    /// プレビュー用のテキスト（HTMLから変換済み）
    pub preview_content: Vec<String>,
    /// プレビュー画面の表示可能行数
    pub preview_height: u16,
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
            feeds_list_height: 10,    // 初期値、UIで更新される
            articles_list_height: 10, // 初期値、UIで更新される
            show_preview: false,
            preview_scroll: 0,
            preview_content: Vec::new(),
            preview_height: 10,       // 初期値、UIで更新される
        })
    }

    /// メインループを実行
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // 画面を描画（リスト高さの更新のため可変参照）
            terminal.draw(|frame| ui::render(frame, self))?;

            // イベントをポーリング（100msタイムアウト）
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // KeyEventKind::Press のみ処理（リピートを防ぐ）
                    if key.kind == KeyEventKind::Press {
                        // Ctrl+L で画面をリフレッシュ
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('l')
                        {
                            terminal.clear()?;
                            self.status_message = Some("Screen refreshed".to_string());
                            continue;
                        }
                        self.handle_key(key.code, key.modifiers).await?;
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
    async fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        // プレビューモード時は専用のキー処理
        if self.show_preview {
            return self.handle_preview_key(key, modifiers);
        }

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

            // ページ単位移動
            KeyCode::PageUp => {
                self.page_up();
            }
            KeyCode::PageDown => {
                self.page_down();
            }

            // 半ページ移動 (Ctrl+u/d)
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.half_page_up();
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.half_page_down();
            }

            // 先頭/末尾へジャンプ
            KeyCode::Char('g') => {
                self.jump_to_top();
            }
            KeyCode::Char('G') => {
                self.jump_to_bottom();
            }
            KeyCode::Home => {
                self.jump_to_top();
            }
            KeyCode::End => {
                self.jump_to_bottom();
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

            // 記事をブラウザで開く
            KeyCode::Char('o') => {
                if self.focus == Focus::Articles && !self.articles.is_empty() {
                    self.open_article_in_browser()?;
                }
            }

            // プレビュー表示
            KeyCode::Char('p') => {
                if self.focus == Focus::Articles && !self.articles.is_empty() {
                    self.open_preview();
                }
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

    /// ページアップ（リスト高さ分上に移動）
    fn page_up(&mut self) {
        match self.focus {
            Focus::Feeds => {
                let page_size = self.feeds_list_height.saturating_sub(2) as usize;
                self.selected_feed = self.selected_feed.saturating_sub(page_size);
                let _ = self.load_articles_for_selected_feed();
            }
            Focus::Articles => {
                let page_size = self.articles_list_height.saturating_sub(2) as usize;
                self.selected_article = self.selected_article.saturating_sub(page_size);
            }
        }
    }

    /// ページダウン（リスト高さ分下に移動）
    fn page_down(&mut self) {
        match self.focus {
            Focus::Feeds => {
                let page_size = self.feeds_list_height.saturating_sub(2) as usize;
                let max_index = self.feeds.len().saturating_sub(1);
                self.selected_feed = (self.selected_feed + page_size).min(max_index);
                let _ = self.load_articles_for_selected_feed();
            }
            Focus::Articles => {
                let page_size = self.articles_list_height.saturating_sub(2) as usize;
                let max_index = self.articles.len().saturating_sub(1);
                self.selected_article = (self.selected_article + page_size).min(max_index);
            }
        }
    }

    /// 半ページアップ
    fn half_page_up(&mut self) {
        match self.focus {
            Focus::Feeds => {
                let half_page = (self.feeds_list_height.saturating_sub(2) / 2) as usize;
                self.selected_feed = self.selected_feed.saturating_sub(half_page.max(1));
                let _ = self.load_articles_for_selected_feed();
            }
            Focus::Articles => {
                let half_page = (self.articles_list_height.saturating_sub(2) / 2) as usize;
                self.selected_article = self.selected_article.saturating_sub(half_page.max(1));
            }
        }
    }

    /// 半ページダウン
    fn half_page_down(&mut self) {
        match self.focus {
            Focus::Feeds => {
                let half_page = (self.feeds_list_height.saturating_sub(2) / 2) as usize;
                let max_index = self.feeds.len().saturating_sub(1);
                self.selected_feed = (self.selected_feed + half_page.max(1)).min(max_index);
                let _ = self.load_articles_for_selected_feed();
            }
            Focus::Articles => {
                let half_page = (self.articles_list_height.saturating_sub(2) / 2) as usize;
                let max_index = self.articles.len().saturating_sub(1);
                self.selected_article = (self.selected_article + half_page.max(1)).min(max_index);
            }
        }
    }

    /// 先頭へジャンプ
    fn jump_to_top(&mut self) {
        match self.focus {
            Focus::Feeds => {
                if self.selected_feed != 0 {
                    self.selected_feed = 0;
                    let _ = self.load_articles_for_selected_feed();
                }
            }
            Focus::Articles => {
                self.selected_article = 0;
            }
        }
    }

    /// 末尾へジャンプ
    fn jump_to_bottom(&mut self) {
        match self.focus {
            Focus::Feeds => {
                let max_index = self.feeds.len().saturating_sub(1);
                if self.selected_feed != max_index {
                    self.selected_feed = max_index;
                    let _ = self.load_articles_for_selected_feed();
                }
            }
            Focus::Articles => {
                self.selected_article = self.articles.len().saturating_sub(1);
            }
        }
    }

    /// 選択中のフィードの記事を読み込む
    fn load_articles_for_selected_feed(&mut self) -> Result<()> {
        if let Some(feed) = self.feeds.get(self.selected_feed) {
            let previous_selected = self.selected_article;
            self.articles = self.db.get_articles(false, 50, None, Some(feed.id))?;
            // 選択位置を保持（記事数が減った場合は調整）
            if self.articles.is_empty() {
                self.selected_article = 0;
            } else {
                self.selected_article = previous_selected.min(self.articles.len() - 1);
            }
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

    /// 記事をブラウザで開く
    fn open_article_in_browser(&mut self) -> Result<()> {
        if let Some(article) = self.articles.get(self.selected_article) {
            if let Some(url) = &article.url {
                match Self::open_url(url) {
                    Ok(_) => {
                        self.status_message = Some("Opened in browser".to_string());
                        // 記事を既読にする
                        if !article.is_read {
                            self.db.mark_as_read(article.id)?;
                            self.load_articles_for_selected_feed()?;
                        }
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to open: {}", e));
                    }
                }
            } else {
                self.status_message = Some("Article has no URL".to_string());
            }
        }
        Ok(())
    }

    /// URLをブラウザで開く（WSL対応）
    fn open_url(url: &str) -> std::io::Result<()> {
        // WSL環境かどうかを検出
        if Self::is_wsl() {
            // WSLの場合はcmd.exeを使用してWindowsブラウザで開く
            // stdout/stderrを抑制して画面の乱れを防ぐ
            std::process::Command::new("cmd.exe")
                .args(["/C", "start", "", url])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;
            Ok(())
        } else {
            // 通常環境ではopen crateを使用
            open::that(url)
        }
    }

    /// WSL環境かどうかを検出
    fn is_wsl() -> bool {
        // /proc/versionにMicrosoftまたはWSLが含まれているか確認
        std::fs::read_to_string("/proc/version")
            .map(|v| v.to_lowercase().contains("microsoft") || v.to_lowercase().contains("wsl"))
            .unwrap_or(false)
    }


    /// プレビューを開く
    fn open_preview(&mut self) {
        if let Some(article) = self.articles.get(self.selected_article) {
            // HTMLコンテンツをテキストに変換
            let content = article.content.as_deref().unwrap_or("(No content available)");
            let text = html2text::from_read(content.as_bytes(), 80);
            
            // 行ごとに分割して保存
            self.preview_content = text.lines().map(|s| s.to_string()).collect();
            self.preview_scroll = 0;
            self.show_preview = true;
        }
    }

    /// プレビューを閉じる
    fn close_preview(&mut self) {
        self.show_preview = false;
        self.preview_content.clear();
        self.preview_scroll = 0;
    }

    /// プレビューモード時のキー処理
    fn handle_preview_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match key {
            // プレビューを閉じる
            KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('q') => {
                self.close_preview();
            }

            // スクロール
            KeyCode::Up | KeyCode::Char('k') => {
                self.preview_scroll_up(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.preview_scroll_down(1);
            }

            // ページ単位スクロール
            KeyCode::PageUp => {
                let page_size = self.preview_height.saturating_sub(4) as usize;
                self.preview_scroll_up(page_size);
            }
            KeyCode::PageDown => {
                let page_size = self.preview_height.saturating_sub(4) as usize;
                self.preview_scroll_down(page_size);
            }

            // 半ページスクロール
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                let half_page = (self.preview_height.saturating_sub(4) / 2) as usize;
                self.preview_scroll_up(half_page.max(1));
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                let half_page = (self.preview_height.saturating_sub(4) / 2) as usize;
                self.preview_scroll_down(half_page.max(1));
            }

            // 先頭/末尾へ
            KeyCode::Char('g') | KeyCode::Home => {
                self.preview_scroll = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                let visible_height = self.preview_height.saturating_sub(4) as usize;
                self.preview_scroll = self.preview_content.len().saturating_sub(visible_height);
            }

            // ブラウザで開く
            KeyCode::Char('o') => {
                self.open_article_in_browser()?;
            }

            _ => {}
        }
        Ok(())
    }

    /// プレビューを上にスクロール
    fn preview_scroll_up(&mut self, lines: usize) {
        self.preview_scroll = self.preview_scroll.saturating_sub(lines);
    }

    /// プレビューを下にスクロール
    fn preview_scroll_down(&mut self, lines: usize) {
        let visible_height = self.preview_height.saturating_sub(4) as usize;
        let max_scroll = self.preview_content.len().saturating_sub(visible_height);
        self.preview_scroll = (self.preview_scroll + lines).min(max_scroll);
    }
}
