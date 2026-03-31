use devicons::FileIcon;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::path::PathBuf;

use crate::app::{App, AppState};
use crate::utils;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(f.area());

    render_file_list(f, app, chunks[0]);
    render_status_bar(f, app, chunks[1]);
    render_overlay(f, app);
}

pub fn render_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .explorer
        .files()
        .iter()
        .map(|file| {
            if file.name() == "../" {
                let line = Line::from(vec![
                    Span::styled("↩ ".to_string(), Color::Yellow),
                    Span::raw(".."),
                ]);
                ListItem::new(line)
            } else {
                if file.is_dir() || file.path().extension().map_or(false, |ext| ext == "csv") {
                    // 目录和 CSV 文件使用默认颜色
                    let icon_data = FileIcon::from(file.path());
                    let icon_color = utils::hex_to_color(icon_data.color);
                    let line = Line::from(vec![
                        Span::styled(format!("{} ", icon_data.icon), icon_color),
                        Span::raw(file.name()),
                    ]);
                    ListItem::new(line)
                } else {
                    // 其他文件使用灰色显示
                    let icon_data = FileIcon::from(file.path());
                    let icon_color = utils::hex_to_color(icon_data.color);
                    let line = Line::from(vec![
                        Span::styled(format!("{} ", icon_data.icon), icon_color),
                        Span::styled(file.name(), Color::DarkGray),
                    ]);
                    ListItem::new(line)
                }
            }
        })
        .collect();

    let current_path = utils::format_path(app.explorer.cwd());
    let list_widget = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" 📂 路径: {} ", current_path))
                .title_alignment(Alignment::Left),
        )
        .highlight_symbol("➜ ")
        .highlight_style(Style::default().bg(Color::Indexed(237)));

    let mut state = ListState::default();
    state.select(Some(app.explorer.selected_idx()));
    f.render_stateful_widget(list_widget, area, &mut state);
}

pub fn render_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_widget(
        Paragraph::new(app.status_msg.as_str()).block(Block::new().borders(Borders::TOP)),
        area,
    );
}

pub fn render_overlay(f: &mut Frame, app: &mut App) {
    match &app.state {
        AppState::SelectingFormat(_) => render_format_selection(f),
        AppState::Naming { .. } => render_naming_input(f, app),
        AppState::ConfirmingOverwrite { dest, .. } => render_confirm_overwrite(f, dest),
        AppState::Error(msg) => render_error(f, msg),
        _ => {}
    }
}

pub fn render_format_selection(f: &mut Frame) {
    let area = utils::centered_rect(30, 20, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(" Select Format ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let text = vec![
        Line::from("1. JSON"),
        Line::from("2. YAML"),
        Line::from("3. TOML"),
        Line::from("[Esc] Back"),
    ];
    f.render_widget(Paragraph::new(text).block(block), area);
}

pub fn render_naming_input(f: &mut Frame, app: &mut App) {
    let area = utils::centered_rect(50, 15, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(" Save As ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let input_widget = Paragraph::new(app.input.value()).block(block);
    f.render_widget(input_widget, area);
    f.set_cursor_position((area.x + app.input.visual_cursor() as u16 + 1, area.y + 1));
}

pub fn render_confirm_overwrite(f: &mut Frame, dest: &PathBuf) {
    let area = utils::centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);
    let msg = vec![
        Line::from(vec![
            Span::raw("文件 "),
            Span::styled(
                dest.file_name().unwrap().to_string_lossy(),
                Style::default().fg(Color::Red),
            ),
            Span::raw(" 已存在"),
        ]),
        Line::from("是否覆盖？ (y/n)"),
    ];
    let block = Block::default()
        .title(" 确认覆盖 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    f.render_widget(
        Paragraph::new(msg)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}

pub fn render_error(f: &mut Frame, msg: &str) {
    let area = utils::centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .title(" 错误 / Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    let text = vec![
        Line::from(Span::styled(
            "操作未能完成: ",
            Style::default().add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::raw(msg)),
        Line::from(""),
        Line::from(Span::styled(
            "按任意键返回",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    f.render_widget(
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center),
        area,
    );
}