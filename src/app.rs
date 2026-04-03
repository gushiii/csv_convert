use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui_explorer::FileExplorer;
use ratatui_explorer::Input::*;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::utils;

// --- 常量定义 ---
const STATUS_MSG_BROWSING: &str = "F/[Enter]: Select File | Q: Quit | ↑/↓: Navigate | J/K/[Home]/[End]: Preview Scroll";
const FORMAT_JSON: &str = "json";
const FORMAT_YAML: &str = "yaml";
const FORMAT_TOML: &str = "toml";

// --- 状态机定义 ---
#[derive(Debug)]
pub enum AppState {
    Browsing,
    SelectingFormat(PathBuf),
    Naming {
        src: PathBuf,
        format: String,
    },
    ConfirmingOverwrite {
        dest: PathBuf,
        src: PathBuf,
        format: String,
    },
    Error(String),
}

#[derive(Debug)]
pub struct App {
    pub explorer: FileExplorer,
    pub preview_cache: String,
    pub preview_scroll: u16,
    pub preview_rx: Receiver<String>,
    pub preview_tx: Sender<PathBuf>,
    pub input: Input,
    pub state: AppState,
    pub status_msg: String,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (request_tx, request_rx) = mpsc::channel::<PathBuf>();
        let (response_tx, response_rx) = mpsc::channel::<String>();

        thread::spawn(move || {
            while let Ok(path) = request_rx.recv() {
                let content = if path.is_dir() {
                    format!("目录: {}\n(选中以进入)", utils::format_path(&path))
                } else {
                    std::fs::read_to_string(&path)
                        .map(|s| s.lines().collect::<Vec<_>>().join("\n"))
                        .unwrap_or_else(|_| "无法读取该文件内容 (可能是二进制文件)".into())
                };
                let _ = response_tx.send(content);
            }
        });

        Ok(Self {
            explorer: FileExplorer::new()?,
            preview_cache: "等待选择文件...".into(),
            preview_scroll: 0,
            preview_rx: response_rx,
            preview_tx: request_tx,
            input: Input::default(),
            state: AppState::Browsing,
            status_msg: STATUS_MSG_BROWSING.into(),
        })
    }

    pub fn update_tick(&mut self) {
        while let Ok(content) = self.preview_rx.try_recv() {
            self.preview_cache = content;
        }
    }

    fn request_preview(&mut self) {
        if let Some(file) = self.explorer.files().get(self.explorer.selected_idx()) {
            self.preview_scroll = 0;
            let _ = self.preview_tx.send(file.path().to_path_buf());
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<bool, Box<dyn Error>> {
        if let Event::Key(key) = event {
            if key.kind != event::KeyEventKind::Press {
                return Ok(false);
            }

            let should_quit = match &self.state {
                AppState::Browsing => self.handle_browsing(key)?,
                AppState::SelectingFormat(path) => {
                    self.handle_selecting_format(key, path.clone())?
                }
                AppState::Naming { src, format } => {
                    self.handle_naming(key, src.clone(), format.clone())?
                }
                AppState::ConfirmingOverwrite { dest, src, format } => self
                    .handle_confirming_overwrite(key, dest.clone(), src.clone(), format.clone())?,
                AppState::Error(_) => {
                    self.state = AppState::Browsing;
                    false
                }
            };
            Ok(should_quit)
        } else {
            Ok(false)
        }
    }

    fn handle_browsing(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<bool, Box<dyn Error>> {
        match key.code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Enter | KeyCode::Char('F') | KeyCode::Char('f') => {
                if let Some(file) = self.explorer.files().get(self.explorer.selected_idx()) {
                    if file.is_file() {
                        if file.path().extension().map_or(false, |ext| ext == "csv") {
                            self.state = AppState::SelectingFormat(file.path().to_path_buf());
                        }
                    } else if file.is_dir() {
                        self.explorer.handle(Right)?;
                        self.request_preview();
                    }
                }
                Ok(false)
            }
            KeyCode::PageDown | KeyCode::Char('k') => {
                self.preview_scroll = self.preview_scroll.saturating_add(5);
                Ok(false)
            }
            KeyCode::PageUp  | KeyCode::Char('j') => {
                self.preview_scroll = self.preview_scroll.saturating_sub(5);
                Ok(false)
            }
            KeyCode::Home => {
                self.preview_scroll = 0;
                Ok(false)
            }
            KeyCode::End => {
                self.preview_scroll = u16::MAX; // UI 渲染里会做下界裁剪
                Ok(false)
            }
            KeyCode::Down => {
                self.explorer.handle(Down)?;
                self.request_preview();
                Ok(false)
            }
            KeyCode::Up => {
                self.explorer.handle(Up)?;
                self.request_preview();
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn handle_selecting_format(
        &mut self,
        key: &crossterm::event::KeyEvent,
        path: PathBuf,
    ) -> Result<bool, Box<dyn Error>> {
        match key.code {
            KeyCode::Char('1') => {
                let default_name = path
                    .with_extension(FORMAT_JSON)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                self.input = Input::new(default_name);
                self.state = AppState::Naming {
                    src: path,
                    format: FORMAT_JSON.to_string(),
                };
            }
            KeyCode::Char('2') => {
                let default_name = path
                    .with_extension(FORMAT_YAML)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                self.input = Input::new(default_name);
                self.state = AppState::Naming {
                    src: path,
                    format: FORMAT_YAML.to_string(),
                };
            }
            KeyCode::Char('3') => {
                let default_name = path
                    .with_extension(FORMAT_TOML)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                self.input = Input::new(default_name);
                self.state = AppState::Naming {
                    src: path,
                    format: FORMAT_TOML.to_string(),
                };
            }
            KeyCode::Esc => self.state = AppState::Browsing,
            _ => {}
        }
        Ok(false)
    }

    fn handle_naming(
        &mut self,
        key: &crossterm::event::KeyEvent,
        src: PathBuf,
        format: String,
    ) -> Result<bool, Box<dyn Error>> {
        match key.code {
            KeyCode::Enter => {
                let dest = src.with_file_name(self.input.value());
                if dest.exists() {
                    self.state = AppState::ConfirmingOverwrite { dest, src, format };
                } else {
                    match utils::try_save(&dest, &src, &format) {
                        Ok(_) => {
                            let path = self.explorer.cwd().to_path_buf();
                            self.explorer.set_cwd(path)?;
                            self.state = AppState::Browsing;
                        }
                        Err(e) => self.state = AppState::Error(e),
                    }
                }
            }
            KeyCode::Esc => self.state = AppState::Browsing,
            _ => {
                self.input.handle_event(&Event::Key(*key));
            }
        }
        Ok(false)
    }

    fn handle_confirming_overwrite(
        &mut self,
        key: &crossterm::event::KeyEvent,
        dest: PathBuf,
        src: PathBuf,
        format: String,
    ) -> Result<bool, Box<dyn Error>> {
        match key.code {
            KeyCode::Char('Y') | KeyCode::Char('y') => {
                match utils::try_save(&dest, &src, &format) {
                    Ok(_) => {
                        let path = self.explorer.cwd().to_path_buf();
                        self.explorer.set_cwd(path)?;
                        self.state = AppState::Browsing;
                    }
                    Err(e) => self.state = AppState::Error(e),
                }
            }
            KeyCode::Char('N') | KeyCode::Char('n') | KeyCode::Esc => {
                self.state = AppState::Naming { src, format };
            }
            _ => {}
        }
        Ok(false)
    }
}
