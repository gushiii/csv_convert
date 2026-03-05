use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, read};

use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};
use ratatui_explorer::FileExplorer;
use std::error::Error;
use std::path::Path;
use tui_input::{Input, backend::crossterm::EventHandler};

// --- 状态机定义 ---
enum AppState {
    Browsing,
    SelectingFormat(PathBuf),
    Naming { src: PathBuf, format: &'static str },
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
            status_msg: "F: Select File | Q: Quit".into(),
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // 基础层: 文件浏览器
    f.render_widget(&app.explorer.widget(), chunks[0]);
    f.render_widget(
        Paragraph::new(app.status_msg.as_str()).block(Block::new().borders(Borders::TOP)),
        chunks[1],
    );

    // 弹出层处理
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
                    KeyCode::Enter => {
                        if let Some(file) = app.explorer.files().get(app.explorer.selected_idx()) {
                            if file.is_file()
                                && file.path().extension().map_or(false, |ext| ext == "csv") {
                                app.state = AppState::SelectingFormat(file.path().to_path_buf());
                            }
                        }
                    }
                    _ => {
                        app.explorer.handle(&event)?;
                    } // 使用内置处理器
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

                        // 执行定义的 convert_csv 逻辑
                        if let Ok(content) = convert_csv(src, format) {
                            std::fs::write(&dest, content).ok();
                        }

                        app.status_msg = format!("Saved to {:?}", dest);
                        app.state = AppState::Browsing;
                    }
                    KeyCode::Esc => app.state = AppState::Browsing,
                    _ => {app.input.handle_event(&event);}
                },
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
