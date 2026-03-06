use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, read};

use devicons::FileIcon;
use ratatui::{
    Frame, Terminal,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use ratatui_explorer::FileExplorer;
use ratatui_explorer::Input::*;
use std::error::Error;
use std::path::Path;
use tui_input::{Input, backend::crossterm::EventHandler};

// --- 状态机定义 ---
enum AppState {
    Browsing,
    SelectingFormat(PathBuf),
    Naming {
        src: PathBuf,
        format: &'static str,
    },
    ConfirmingOverwrite {
        dest: PathBuf,
        src: PathBuf,
        format: &'static str,
    },
    Error(String),
    Info(String),
}

struct App {
    explorer: FileExplorer,
    input: Input,
    state: AppState,
    status_msg: String, // 底部状态栏信息
}

impl App {
    fn new() -> Self {
        Self {
            explorer: FileExplorer::new().unwrap(),
            input: Input::default(),
            state: AppState::Browsing,
            status_msg: "F/[Enter]: Select File | Q: Quit | ↑/↓: Navigate".into(),
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(f.area());

    // 基础层: 文件浏览器
    // 1. 准备 ListItem (包含图标和颜色)
    let items: Vec<ListItem> = app
        .explorer
        .files()
        .iter()
        .map(|file| {
            // 1. 判断是否为“上一级目录”标记
            if file.name() == "../" {
                let line = Line::from(vec![
                    Span::styled("↩ ".to_string(), Color::Yellow), // 使用返回图标
                    Span::raw(".."),
                ]);
                return ListItem::new(line);
            }

            // 2. 正常处理其他文件和文件夹
            let icon_data = FileIcon::from(file.path());
            let icon_color = hex_to_color(icon_data.color);

            let line = Line::from(vec![
                Span::styled(format!("{} ", icon_data.icon), icon_color),
                Span::raw(file.name()),
            ]);

            ListItem::new(line)
        })
        .collect();

    // 2. 创建 Widget 并配置高亮样式
    // 获取当前路径并转换为字符串
    let current_path = format_path(app.explorer.cwd());
    let list_widget = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" 📂 路径: {} ", current_path)) // 在这里显示路径
                .title_alignment(Alignment::Left),
        )
        .highlight_symbol("➜ ")
        .highlight_style(Style::default().bg(Color::Indexed(237))); // 使用 256 色中的深灰

    // 3. 维护并渲染状态
    let mut state = ListState::default();
    state.select(Some(app.explorer.selected_idx()));

    f.render_stateful_widget(list_widget, chunks[0], &mut state);
    f.render_widget(
        Paragraph::new(app.status_msg.as_str()).block(Block::new().borders(Borders::TOP)),
        chunks[1],
    );

    match &app.state {
        AppState::SelectingFormat(_) => {
            let area = centered_rect(30, 20, f.area());
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
        AppState::Naming { .. } => {
            let area = centered_rect(50, 15, f.area());
            f.render_widget(Clear, area);
            let block = Block::default()
                .title(" Save As ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let input_widget = Paragraph::new(app.input.value()).block(block);
            f.render_widget(input_widget, area);

            // 绘制光标
            f.set_cursor_position((area.x + app.input.visual_cursor() as u16 + 1, area.y + 1));
        }
        AppState::ConfirmingOverwrite { dest, .. } => {
            let area = centered_rect(40, 20, f.area());
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
        AppState::Error(msg) => {
            let area = centered_rect(50, 30, f.area());
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
                Line::from(Span::raw(msg)), // 显示错误信息
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
        AppState::Info(msg) => {
            let area = centered_rect(50, 30, f.area());
            f.render_widget(Clear, area);
            let block = Block::default()
                .title(" 信息 / Info ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

            let text = vec![
                Line::from(Span::styled(
                    "打开文件失败: ",
                    Style::default().add_modifier(Modifier::ITALIC),
                )),
                Line::from(""),
                Line::from(Span::raw(msg)), // 显示信息
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
        _ => {}
    }
}

// 辅助函数: 计算居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn convert_csv(input_path: &Path, target_ext: &str) -> Result<String, Box<dyn Error>> {
    // 1. 读取 CSV 并转为通用 Map 列表
    let mut reader = csv::Reader::from_path(input_path)?;
    let mut data = Vec::new();

    // 获取表头
    let headers = reader.headers()?.clone();

    for result in reader.records() {
        let record = result?;
        let mut map = std::collections::HashMap::new();
        for (header, field) in headers.iter().zip(record.iter()) {
            map.insert(header.to_string(), field.to_string());
        }
        data.push(map);
    }

    // 2. 根据目标扩展名序列化
    let output = match target_ext {
        "json" => serde_json::to_string_pretty(&data)?,
        "yaml" | "yml" => serde_yaml::to_string(&data)?,
        "toml" => {
            // TOML 需要一个根表，所以包装一层
            let wrapped = serde_json::json!({ "rows": data });
            toml::to_string_pretty(&wrapped)?
        }
        _ => return Err("Unsupported format".into()),
    };

    Ok(output)
}

// 执行转换并返回结果
fn try_save(dest: &PathBuf, src: &PathBuf, format: &str) -> Result<(), String> {
    let content = convert_csv(src, format).map_err(|e| format!("解析失败: {}", e))?;
    std::fs::write(dest, content).map_err(|e| format!("写入失败: {}", e))?;

    Ok(())
}

// 辅助函数: 将十六进制颜色字符串转换为 ratatui 的 Color
fn hex_to_color(hex: &str) -> Color {
    // 移除可能的 # 前缀
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Color::White; // 格式错误时返回默认色
    }

    // 将十六进制字符串解析为 RGB 数值
    if let Ok(rgb) = u32::from_str_radix(hex, 16) {
        let r = ((rgb >> 16) & 0xFF) as u8;
        let g = ((rgb >> 8) & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;
        Color::Rgb(r, g, b)
    } else {
        Color::White
    }
}

// 辅助函数: 格式化路径，使用 ~ 代表 home 目录
fn format_path(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path_str.starts_with(&*home_str) {
            return path_str.replacen(&*home_str, "~", 1);
        }
    }
    path_str.into_owned()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化终端
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let event = read()?;

        if let Event::Key(key) = event {
            if key.kind != event::KeyEventKind::Press {
                continue;
            }

            match &mut app.state {
                // 第一级: 浏览状态
                AppState::Browsing => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Enter | KeyCode::Char('F') | KeyCode::Char('f') => {
                        if let Some(file) = app.explorer.files().get(app.explorer.selected_idx()) {
                            if file.is_file() {
                                if file.path().extension().map_or(false, |ext| ext == "csv") {
                                    app.state =
                                        AppState::SelectingFormat(file.path().to_path_buf());
                                } else {
                                    app.state = AppState::Info("请选择一个 CSV 文件".into());
                                }
                            } else if file.is_dir() {
                                app.explorer.handle(Right)?;
                            }
                        }
                    }
                    KeyCode::Down => app.explorer.handle(Down)?,
                    KeyCode::Up => app.explorer.handle(Up)?,
                    _ => {}
                },

                // 第二级: 选择格式
                AppState::SelectingFormat(path) => match key.code {
                    KeyCode::Char('1') => {
                        let default_name = path
                            .with_extension("json")
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned();
                        app.input = Input::new(default_name);
                        app.state = AppState::Naming {
                            src: path.clone(),
                            format: "json",
                        };
                    }
                    KeyCode::Char('2') => {
                        let default_name = path
                            .with_extension("yaml")
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned();
                        app.input = Input::new(default_name);
                        app.state = AppState::Naming {
                            src: path.clone(),
                            format: "yaml",
                        };
                    }
                    KeyCode::Char('3') => {
                        let default_name = path
                            .with_extension("toml")
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned();
                        app.input = Input::new(default_name);
                        app.state = AppState::Naming {
                            src: path.clone(),
                            format: "toml",
                        };
                    }
                    KeyCode::Esc => {
                        app.state = AppState::Browsing;
                    }
                    _ => {}
                },

                // 第三级: 输入文件名并保存
                AppState::Naming { src, format } => match key.code {
                    KeyCode::Enter => {
                        let dest = src.with_file_name(app.input.value());
                        if dest.exists() {
                            // 如果目标文件已存在，进入确认覆盖状态
                            app.state = AppState::ConfirmingOverwrite {
                                dest,
                                src: src.clone(),
                                format,
                            };
                        } else {
                            // 尝试保存，失败则进入错误状态
                            match try_save(&dest, src, format) {
                                Ok(_) => {
                                    let path = app.explorer.cwd().to_path_buf();
                                    app.explorer.set_cwd(path)?;
                                    app.state = AppState::Browsing;
                                }
                                Err(e) => app.state = AppState::Error(e),
                            }
                        }
                    }
                    KeyCode::Esc => app.state = AppState::Browsing,
                    _ => {
                        app.input.handle_event(&event);
                    }
                },

                // 第四级: 确认覆盖逻辑
                AppState::ConfirmingOverwrite { dest, src, format } => match key.code {
                    KeyCode::Char('Y') | KeyCode::Char('y') => match try_save(&dest, src, format) {
                        Ok(_) => {
                            let path = app.explorer.cwd().to_path_buf();
                            app.explorer.set_cwd(path)?;
                            app.state = AppState::Browsing;
                        }
                        Err(e) => app.state = AppState::Error(e),
                    },
                    KeyCode::Char('N') | KeyCode::Char('n') | KeyCode::Esc => {
                        // 用户选择不覆盖，返回命名状态
                        app.state = AppState::Naming {
                            src: src.clone(),
                            format,
                        };
                    }
                    _ => {}
                },

                // 错误状态: 任何按键返回浏览状态
                AppState::Error(_) => {
                    app.state = AppState::Browsing;
                },

                // 信息状态: 任何按键返回浏览状态
                AppState::Info(_) => {
                    app.state = AppState::Browsing;
                }
            }
        }
    }

    // 恢复终端
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
