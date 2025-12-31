//! # UIレンダリングモジュール
//!
//! TUIの描画処理を担当します。

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::app::{App, Focus};

/// メイン描画関数
pub fn render(frame: &mut Frame, app: &mut App) {
    // プレビューモードの場合はオーバーレイを描画
    if app.show_preview {
        render_preview(frame, app);
        return;
    }
    // レイアウトを作成（縦に3分割：ヘッダー、メイン、フッター）
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // ヘッダー
            Constraint::Min(0),    // メインエリア
            Constraint::Length(3), // フッター（ヘルプ）
        ])
        .split(frame.area());

    // ヘッダーを描画
    render_header(frame, chunks[0]);

    // メインエリアを左右に分割
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // フィード一覧
            Constraint::Percentage(70), // 記事一覧
        ])
        .split(chunks[1]);

    // リスト高さを更新（ボーダー分を引いた内部の高さ）
    app.feeds_list_height = main_chunks[0].height;
    app.articles_list_height = main_chunks[1].height;

    // フィード一覧を描画
    render_feeds(frame, app, main_chunks[0]);

    // 記事一覧を描画
    render_articles(frame, app, main_chunks[1]);

    // フッター（ヘルプ）を描画
    render_footer(frame, app, chunks[2]);
}

/// ヘッダーを描画
fn render_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new("rustfeed-tui - RSS Reader")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(header, area);
}

/// フィード一覧を描画
fn render_feeds(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .feeds
        .iter()
        .map(|feed| {
            let name = feed.display_name();
            ListItem::new(name.to_string())
        })
        .collect();

    let border_style = if app.focus == Focus::Feeds {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let feeds_list = List::new(items)
        .block(
            Block::default()
                .title(" Feeds ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_feed));

    frame.render_stateful_widget(feeds_list, area, &mut state);

    // スクロールバーを描画（アイテムが表示領域より多い場合のみ）
    let visible_height = area.height.saturating_sub(2) as usize; // ボーダー分を引く
    if app.feeds.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(app.feeds.len())
            .position(app.selected_feed)
            .viewport_content_length(visible_height);

        // スクロールバーの描画領域（ボーダーの内側）
        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y + 1,
            width: 1,
            height: area.height.saturating_sub(2),
        };

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

/// 記事一覧を描画
fn render_articles(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .articles
        .iter()
        .map(|article| {
            let read_marker = if article.is_read { "  " } else { "* " };
            let fav_marker = if article.is_favorite { "♥ " } else { "  " };
            let date = article
                .published_at
                .map(|dt| dt.format("%m/%d").to_string())
                .unwrap_or_else(|| "-----".to_string());

            let line = Line::from(vec![
                Span::styled(
                    read_marker,
                    Style::default().fg(if article.is_read {
                        Color::DarkGray
                    } else {
                        Color::Cyan
                    }),
                ),
                Span::styled(fav_marker, Style::default().fg(Color::Red)),
                Span::styled(format!("{} ", date), Style::default().fg(Color::DarkGray)),
                Span::raw(&article.title),
            ]);

            ListItem::new(line)
        })
        .collect();

    let border_style = if app.focus == Focus::Articles {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let title = if app.feeds.is_empty() {
        " Articles ".to_string()
    } else if let Some(feed) = app.feeds.get(app.selected_feed) {
        format!(" {} ", feed.display_name())
    } else {
        " Articles ".to_string()
    };

    let articles_list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_article));

    frame.render_stateful_widget(articles_list, area, &mut state);

    // スクロールバーを描画（アイテムが表示領域より多い場合のみ）
    let visible_height = area.height.saturating_sub(2) as usize; // ボーダー分を引く
    if app.articles.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(app.articles.len())
            .position(app.selected_article)
            .viewport_content_length(visible_height);

        // スクロールバーの描画領域（ボーダーの内側）
        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y + 1,
            width: 1,
            height: area.height.saturating_sub(2),
        };

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

/// フッター（ヘルプ）を描画
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = "q:Quit j/k:Move g/G:Top/End ^u/d:Half Tab:Switch r:Read f:Fav o:Open p:Preview";

    let status = if let Some(msg) = &app.status_message {
        format!(" | {}", msg)
    } else {
        String::new()
    };

    let footer = Paragraph::new(format!("{}{}", help_text, status))
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}


/// プレビュー画面を描画（オーバーレイ）
fn render_preview(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // 背景をクリア
    frame.render_widget(Clear, area);

    // レイアウト：タイトル、コンテンツ、フッター
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // タイトル
            Constraint::Min(0),    // コンテンツ
            Constraint::Length(3), // フッター
        ])
        .split(area);

    // プレビュー高さを更新
    app.preview_height = chunks[1].height;

    // 選択中の記事を取得
    let article = match app.articles.get(app.selected_article) {
        Some(a) => a,
        None => return,
    };

    // タイトルバー
    let title = format!(" {} ", article.title);
    let title_paragraph = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(title_paragraph, chunks[0]);

    // コンテンツエリア
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let content_lines: Vec<Line> = app
        .preview_content
        .iter()
        .skip(app.preview_scroll)
        .take(visible_height)
        .map(|s| Line::from(s.as_str()))
        .collect();

    let content_paragraph = Paragraph::new(content_lines)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content_paragraph, chunks[1]);

    // スクロールバー（コンテンツが表示領域より多い場合）
    if app.preview_content.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(app.preview_content.len())
            .position(app.preview_scroll)
            .viewport_content_length(visible_height);

        let scrollbar_area = Rect {
            x: chunks[1].x + chunks[1].width - 1,
            y: chunks[1].y + 1,
            width: 1,
            height: chunks[1].height.saturating_sub(2),
        };

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // フッター（ヘルプ）
    let scroll_info = format!(
        " Line {}/{} ",
        app.preview_scroll + 1,
        app.preview_content.len().max(1)
    );
    let help_text = "Esc/p/q:Close  j/k:Scroll  PgUp/Dn  ^u/d:Half  g/G:Top/End  o:Open";
    
    let footer = Paragraph::new(format!("{}{}", help_text, scroll_info))
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}
