use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui_explorer::FileExplorer;
use ratatui_explorer::Input::*;
use std::error::Error;
use std::path::PathBuf;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::utils;

// --- 常量定义 ---
const STATUS_MSG_BROWSING: &str = "F/[Enter]: Select File | Q: Quit | ↑/↓: Navigate";
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
    pub input: Input,
    pub state: AppState,
    pub status_msg: String,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            explorer: FileExplorer::new()?,
            input: Input::default(),
            state: AppState::Browsing,
            status_msg: STATUS_MSG_BROWSING.into(),
        })
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
                    }
                }
                Ok(false)
            }
            KeyCode::Down => {
                self.explorer.handle(Down)?;
                Ok(false)
            }
            KeyCode::Up => {
                self.explorer.handle(Up)?;
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
