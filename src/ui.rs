use crate::app::{App, AppState};
use crate::utils;
use devicons::FileIcon;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::path::PathBuf;

pub fn ui(f: &mut Frame, app: &mut App) {
    // 1. 垂直布局：主内容区 + 底部状态栏
    let main_chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(f.area());

    // 2. 水平布局：左侧文件列表 + 右侧预览窗格
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // 列表宽度
            Constraint::Percentage(80), // 预览宽度
        ])
        .split(main_chunks[0]);

    // 渲染各个组件
    render_file_list(f, app, body_chunks[0]);
    render_preview(f, app, body_chunks[1]); // 新增预览渲染
    render_status_bar(f, app, main_chunks[1]);

    // 渲染悬浮层（如命名输入、错误提示等）
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
                let icon_data = FileIcon::from(file.path());
                let icon_color = utils::hex_to_color(icon_data.color);

                // 判断是否为高亮文件 (目录或 CSV)
                let is_highlight =
                    file.is_dir() || file.path().extension().map_or(false, |ext| ext == "csv");

                let text_style = if is_highlight {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let line = Line::from(vec![
                    Span::styled(format!("{} ", icon_data.icon), icon_color),
                    Span::styled(file.name(), text_style),
                ]);
                ListItem::new(line)
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
        .highlight_style(
            Style::default()
                .bg(Color::Indexed(237))
                .add_modifier(Modifier::BOLD),
        );

    // 注意：建议在 App 中维护这个 state 以保持滚动位置
    let mut state = ListState::default();
    state.select(Some(app.explorer.selected_idx()));
    f.render_stateful_widget(list_widget, area, &mut state);
}

/// 新增：渲染预览窗格
pub fn render_preview(f: &mut Frame, app: &mut App, area: Rect) {
    // 从 app 中获取异步加载好的预览文本
    // 如果正在加载或为空，可以显示提示
    let preview_text = if app.preview_cache.is_empty() {
        "\n  正在加载或无法预览该文件..."
    } else {
        &app.preview_cache
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 📝 预览 / Preview ")
        .border_style(Style::default().fg(Color::Indexed(244))); // 灰色边框，区分主列表

    let p = Paragraph::new(preview_text)
        .block(block)
        .wrap(Wrap { trim: false }) // 文件内容通常不希望修剪空格
        .alignment(Alignment::Left);

    f.render_widget(p, area);
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
