use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, FilterField, KeywordMode, Screen};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    match app.screen {
        Screen::LanguageSelect => draw_language_select(f, app, area),
        Screen::PlatformSelect => draw_platform_select(f, app, area),
        Screen::TokenInput => draw_token_input(f, app, area),
        Screen::Filter => draw_filter(f, app, area),
        Screen::Loading => draw_loading(f, app, area),
        Screen::ConversationList => draw_conversation_list(f, app, area),
        Screen::Downloading => draw_downloading(f, app, area),
        Screen::Done => draw_done(f, app, area),
        Screen::Error => draw_error(f, app, area),
    }
}

fn draw_language_select(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(" LLM Chat Export")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let langs = [
        ("中文", "Chinese - 使用中文界面"),
        ("English", "English - Use English interface"),
    ];
    let items: Vec<ListItem> = langs
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let prefix = if i == app.lang_cursor { " > " } else { "   " };
            let style = if i == app.lang_cursor {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(vec![
                Line::from(Span::styled(format!("{}{}", prefix, name), style)),
                Line::from(Span::styled(
                    format!("     {}", desc),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Select Language / 选择语言 "),
    );
    f.render_widget(list, chunks[1]);

    let help = Paragraph::new(" Up/Down | Enter | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn draw_platform_select(f: &mut Frame, app: &App, area: Rect) {
    let t = app.texts();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(format!(" {}", t.app_title))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let descs = [t.desc_chatgpt, t.desc_claude_web, t.desc_claude_code, t.desc_cursor];
    let items: Vec<ListItem> = app
        .platforms
        .iter()
        .enumerate()
        .map(|(i, (name, _, _))| {
            let prefix = if i == app.platform_cursor {
                " > "
            } else {
                "   "
            };
            let style = if i == app.platform_cursor {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(vec![
                Line::from(Span::styled(format!("{}{}", prefix, name), style)),
                Line::from(Span::styled(
                    format!("     {}", descs[i]),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(t.select_platform),
    );
    f.render_widget(list, chunks[1]);

    let help = Paragraph::new(format!(
        " {} | {} | {} | {}",
        t.help_nav, t.help_enter_select, t.help_esc_back, t.help_q_quit
    ))
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn draw_token_input(f: &mut Frame, app: &App, area: Rect) {
    let t = app.texts();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    let platform_name = match app.platform_cursor {
        0 => "ChatGPT Access Token",
        1 => "Claude Session Key",
        _ => "Token",
    };

    let title = Paragraph::new(format!(" {}: {}", t.input_label, platform_name))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let display = if app.token_visible {
        format!("{}_", app.token)
    } else if app.token.is_empty() {
        "_".to_string()
    } else {
        format!("{}_", "*".repeat(app.token.len()))
    };

    let input = Paragraph::new(display)
        .block(Block::default().borders(Borders::ALL).title(t.token_label))
        .wrap(Wrap { trim: false });
    f.render_widget(input, chunks[1]);

    let hint_text = match app.platform_cursor {
        0 => t.hint_chatgpt,
        1 => t.hint_claude,
        _ => "",
    };
    let hint = Paragraph::new(hint_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(t.hint_label))
        .wrap(Wrap { trim: false });
    f.render_widget(hint, chunks[2]);

    let help = Paragraph::new(format!(
        " {} | {} | {}",
        t.help_enter_confirm, t.help_tab_show_hide, t.help_esc_back
    ))
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[3]);
}

fn draw_filter(f: &mut Frame, app: &App, area: Rect) {
    let t = app.texts();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(format!(" {}", t.app_title))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let focus = app.filter_focus;
    let active_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default();
    let dim_style = Style::default().fg(Color::DarkGray);

    let mode_label = match app.filter_keyword_mode {
        KeywordMode::Include => t.include,
        KeywordMode::Exclude => t.exclude,
    };

    let arrow = |field: FilterField| -> &str {
        if focus == field {
            " > "
        } else {
            "   "
        }
    };
    let style_for = |field: FilterField| -> Style {
        if focus == field {
            active_style
        } else {
            normal_style
        }
    };
    let cursor = |field: FilterField, text: &str| -> String {
        if focus == field {
            format!("{}_", text)
        } else if text.is_empty() {
            "-".to_string()
        } else {
            text.to_string()
        }
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                arrow(FilterField::KeywordMode),
                style_for(FilterField::KeywordMode),
            ),
            Span::styled(format!("{}  ", t.keyword_mode), normal_style),
            Span::styled(
                format!("[ {} ]", mode_label),
                style_for(FilterField::KeywordMode),
            ),
            Span::styled(format!("  {}", t.space_to_toggle), dim_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                arrow(FilterField::Keywords),
                style_for(FilterField::Keywords),
            ),
            Span::styled(t.keywords, normal_style),
            Span::styled(
                cursor(FilterField::Keywords, &app.filter_keywords),
                style_for(FilterField::Keywords),
            ),
        ]),
        Line::from(Span::styled(
            format!("                    {}", t.keywords_hint),
            dim_style,
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                arrow(FilterField::StartDate),
                style_for(FilterField::StartDate),
            ),
            Span::styled(t.start_date, normal_style),
            Span::styled(
                cursor(FilterField::StartDate, &app.filter_start_date),
                style_for(FilterField::StartDate),
            ),
            Span::styled("  (YYYY-MM-DD)", dim_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                arrow(FilterField::EndDate),
                style_for(FilterField::EndDate),
            ),
            Span::styled(t.end_date, normal_style),
            Span::styled(
                cursor(FilterField::EndDate, &app.filter_end_date),
                style_for(FilterField::EndDate),
            ),
            Span::styled("  (YYYY-MM-DD)", dim_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("    {}", t.filter_empty_hint),
            dim_style,
        )),
    ];

    let form = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(t.filter_title),
    );
    f.render_widget(form, chunks[1]);

    let enter_text = if app.has_fetched {
        t.help_enter_apply
    } else {
        t.help_enter_fetch
    };
    let help = Paragraph::new(format!(
        " {} | {} | {} | {}",
        t.help_tab_switch, t.help_space_mode, enter_text, t.help_esc_back
    ))
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn draw_loading(f: &mut Frame, app: &App, area: Rect) {
    let t = app.texts();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(t.filter_title);
    let text = Paragraph::new(format!(
        "\n  {}\n\n  {}",
        t.loading_msg, t.help_esc_cancel
    ))
    .style(Style::default().fg(Color::Yellow))
    .block(block);
    f.render_widget(text, area);
}

fn draw_conversation_list(f: &mut Frame, app: &mut App, area: Rect) {
    let t = app.texts();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    let selected_count = app.conversations.iter().filter(|c| c.selected).count();
    let info = format!(
        " {}: {} | {}: {}",
        t.total_label,
        app.conversations.len(),
        t.selected_label,
        selected_count
    );
    let header = Paragraph::new(info)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.conversations),
        );
    f.render_widget(header, chunks[0]);

    let cursor_idx = app.list_state.selected().unwrap_or(0);
    let items: Vec<ListItem> = app
        .conversations
        .iter()
        .enumerate()
        .map(|(i, conv)| {
            let checkbox = if conv.selected { "[x]" } else { "[ ]" };
            let cur = if i == cursor_idx { " > " } else { "   " };

            let style = if i == cursor_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if conv.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let title_display: String = if conv.title.chars().count() > 50 {
                format!("{}...", conv.title.chars().take(47).collect::<String>())
            } else {
                conv.title.clone()
            };

            let time_display = if conv.last_chat_time.is_empty() {
                &conv.created_at
            } else {
                &conv.last_chat_time
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{}{} ", cur, checkbox), style),
                Span::styled(title_display, style),
                Span::styled(
                    format!("  {}", time_display),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default());

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    let help = Paragraph::new(format!(
        " {} | {} | {} | {} | {} | {}",
        t.help_space_toggle,
        t.help_a_select_all,
        t.help_enter_download,
        t.help_f_filter,
        t.help_esc_back,
        t.help_q_quit
    ))
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn draw_downloading(f: &mut Frame, app: &App, area: Rect) {
    let t = app.texts();
    let progress_text = if app.download_total > 0 {
        let pct = app.download_current as f64 / app.download_total as f64 * 100.0;
        format!(
            "  {} ({}/{}) {:.0}%",
            t.downloading_msg, app.download_current, app.download_total, pct
        )
    } else {
        format!("  {}", t.preparing_msg)
    };

    let bar_width = (area.width as usize).saturating_sub(6);
    let filled = if app.download_total > 0 {
        bar_width * app.download_current / app.download_total
    } else {
        0
    };
    let bar = format!(
        "  [{}{}]",
        "#".repeat(filled),
        "-".repeat(bar_width.saturating_sub(filled))
    );

    let text = format!("\n{}\n\n{}", progress_text, bar);
    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(t.downloading_title),
        );
    f.render_widget(widget, area);
}

fn draw_done(f: &mut Frame, app: &App, area: Rect) {
    let t = app.texts();
    let text = format!(
        "\n  {}\n\n  {} {} {}\n  {}\n\n  {} | {}",
        t.done_msg,
        t.exported_to,
        app.download_total,
        t.conversations.trim(),
        app.download_path,
        t.help_enter_back,
        t.help_q_quit
    );
    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title(t.done_title));
    f.render_widget(widget, area);
}

fn draw_error(f: &mut Frame, app: &App, area: Rect) {
    let t = app.texts();
    let text = format!(
        "\n  {}\n\n  {}\n\n  {} | {}",
        t.error_title.trim(),
        app.error_msg,
        t.help_enter_back,
        t.help_q_quit
    );
    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL).title(t.error_title))
        .wrap(Wrap { trim: false });
    f.render_widget(widget, area);
}
