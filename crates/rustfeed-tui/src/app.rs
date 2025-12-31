//! # アプリケーション状態管理
//!
//! TUIアプリケーションの状態とメインループを管理します。

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, Terminal};
use rustfeed_core::{config::AppConfig, db::Database, feed, Article, Feed};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::ui;

/// フィード更新の結果を表すメッセージ
pub enum FetchMessage {
    /// 更新開始（フィード名）
    Started(String),
    /// 1つのフィード更新完了（フィード名, 新規記事数, エラーメッセージ）
    FeedDone(String, usize, Option<String>),
    /// 全フィード更新完了（合計新規記事数）
    AllDone(usize),
}

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
    /// フィード更新中フラグ
    pub is_fetching: bool,
    /// フィード更新メッセージ受信用チャンネル
    pub fetch_rx: Option<mpsc::Receiver<FetchMessage>>,
    /// 現在更新中のフィード名
    pub fetching_feed: Option<String>,
    /// 更新進捗（完了数/全体数）
    pub fetch_progress: (usize, usize),
    /// 検索モードが有効かどうか
    pub search_mode: bool,
    /// 検索クエリ（入力中）
    pub search_query: String,
    /// 検索が有効かどうか（検索結果を表示中）
    pub search_active: bool,
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
            is_fetching: false,
            fetch_rx: None,
            fetching_feed: None,
            fetch_progress: (0, 0),
            search_mode: false,
            search_query: String::new(),
            search_active: false,
        })
    }

    /// メインループを実行
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // 画面を描画（リスト高さの更新のため可変参照）
            terminal.draw(|frame| ui::render(frame, self))?;

            // フェッチメッセージを処理（非ブロッキング）
            self.process_fetch_messages()?;

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
        // 検索モード時は専用のキー処理
        if self.search_mode {
            return self.handle_search_key(key);
        }

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
                // 検索をクリア
                if self.search_active {
                    self.clear_search()?;
                }
                self.status_message = Some("Feeds".to_string());
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                if self.focus == Focus::Feeds && !self.feeds.is_empty() {
                    self.focus = Focus::Articles;
                    // 検索をクリア
                    if self.search_active {
                        self.clear_search()?;
                    } else {
                        self.load_articles_for_selected_feed()?;
                    }
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
                self.start_fetch();
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

            // 検索モード開始
            KeyCode::Char('/') => {
                self.start_search();
            }

            // 検索クリア
            KeyCode::Esc => {
                if self.search_active {
                    self.clear_search()?;
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


    /// フィード更新を開始（バックグラウンドで実行）
    fn start_fetch(&mut self) {
        if self.is_fetching {
            self.status_message = Some("Already fetching...".to_string());
            return;
        }

        let feeds = self.feeds.clone();
        if feeds.is_empty() {
            self.status_message = Some("No feeds to fetch".to_string());
            return;
        }

        // チャンネルを作成
        let (tx, rx) = mpsc::channel::<FetchMessage>(32);
        self.fetch_rx = Some(rx);
        self.is_fetching = true;
        self.fetch_progress = (0, feeds.len());
        self.status_message = Some("Starting fetch...".to_string());

        // バックグラウンドタスクを起動
        tokio::spawn(async move {
            // バックグラウンドタスク用のDB接続を作成
            let db = match Database::new() {
                Ok(d) => d,
                Err(_) => return,
            };

            let mut total_new = 0;

            for stored_feed in feeds {
                let feed_name = stored_feed.display_name().to_string();
                
                // 更新開始を通知
                let _ = tx.send(FetchMessage::Started(feed_name.clone())).await;

                // フィードを取得
                match feed::fetch_feed(&stored_feed.url).await {
                    Ok((_feed_info, articles)) => {
                        let mut new_count = 0;

                        for mut article in articles {
                            article.feed_id = stored_feed.id;
                            if let Ok(Some(_)) = db.add_article(&article) {
                                new_count += 1;
                            }
                        }

                        total_new += new_count;
                        let _ = tx.send(FetchMessage::FeedDone(feed_name, new_count, None)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(FetchMessage::FeedDone(
                            feed_name,
                            0,
                            Some(e.to_string()),
                        )).await;
                    }
                }
            }

            // 全完了を通知
            let _ = tx.send(FetchMessage::AllDone(total_new)).await;
        });
    }

    /// フェッチメッセージを処理（非ブロッキング）
    fn process_fetch_messages(&mut self) -> Result<()> {
        // fetch_rxがNoneならすぐに戻る
        let rx = match &mut self.fetch_rx {
            Some(rx) => rx,
            None => return Ok(()),
        };

        // メッセージを収集（借用を解放するため）
        let mut messages = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }

        // 収集したメッセージを処理
        for msg in messages {
            match msg {
                FetchMessage::Started(name) => {
                    self.fetching_feed = Some(name.clone());
                    self.status_message = Some(format!("Fetching {}...", name));
                }
                FetchMessage::FeedDone(name, new_count, error) => {
                    self.fetch_progress.0 += 1;
                    if let Some(err) = error {
                        self.status_message = Some(format!("{}: Error - {}", name, err));
                    } else {
                        self.status_message = Some(format!(
                            "{}: {} new ({}/{})",
                            name,
                            new_count,
                            self.fetch_progress.0,
                            self.fetch_progress.1
                        ));
                    }
                }
                FetchMessage::AllDone(total_new) => {
                    self.is_fetching = false;
                    self.fetching_feed = None;
                    self.fetch_rx = None;
                    self.status_message = Some(format!(
                        "Fetch complete! {} new articles",
                        total_new
                    ));
                    // フィードと記事を再読み込み
                    self.feeds = self.db.get_feeds(None)?;
                    self.load_articles_for_selected_feed()?;
                }
            }
        }
        Ok(())
    }


    /// 検索モードを開始
    fn start_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.focus = Focus::Articles;
    }

    /// 検索モード時のキー処理
    fn handle_search_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            // 検索実行
            KeyCode::Enter => {
                if !self.search_query.is_empty() {
                    self.execute_search()?;
                }
                self.search_mode = false;
            }

            // 検索キャンセル
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
            }

            // 文字入力
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }

            // バックスペース
            KeyCode::Backspace => {
                self.search_query.pop();
            }

            _ => {}
        }
        Ok(())
    }

    /// 検索を実行
    fn execute_search(&mut self) -> Result<()> {
        // 全フィードから検索（フィルタとして検索クエリを使用）
        self.articles = self.db.get_articles(
            false,  // unread_only
            100,    // limit（検索結果は多めに）
            Some(&self.search_query),  // filter
            None,   // feed_id（全フィード対象）
        )?;

        self.search_active = true;
        self.selected_article = 0;
        self.status_message = Some(format!(
            "Search: '{}' ({} results)",
            self.search_query,
            self.articles.len()
        ));

        Ok(())
    }

    /// 検索をクリア
    fn clear_search(&mut self) -> Result<()> {
        self.search_active = false;
        self.search_query.clear();
        self.load_articles_for_selected_feed()?;
        self.status_message = Some("Search cleared".to_string());
        Ok(())
    }
}
