//! # CSV Convert
//!
//! 一个基于Rust的终端用户界面(TUI)应用程序，用于转换和处理CSV文件。
//!
//! ## 主要功能
//!
//! - 交互式终端用户界面
//! - 事件驱动的应用架构
//! - 实时界面更新和渲染
//!
//! ## 架构
//!
//! 应用采用典型的TUI架构：
//! 1. **事件循环** - 持续监听用户输入和定时更新
//! 2. **状态管理** - `App` 结构体管理应用状态
//! 3. **UI渲染** - 根据状态渲染界面
//! 4. **事件处理** - 处理键盘事件并更新状态

use crossterm::event::{self, Event};
use ratatui::{Terminal, prelude::CrosstermBackend};

mod app;
mod ui;
mod utils;

/// 应用程序的主入口点
///
/// # 工作流程
///
/// 1. 启用原始模式和交替屏幕缓冲区
/// 2. 创建Terminal和App实例
/// 3. 运行事件循环直到用户退出
/// 4. 恢复终端状态
///
/// # 错误处理
///
/// 返回`Result`以处理可能的IO和终端操作错误
///
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 启用原始模式，禁用规范输入处理
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    // 进入交替屏幕缓冲区
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // 初始化应用状态
    let mut app = app::App::new()?;

    // 事件循环
    loop {
        // 绘制UI
        terminal.draw(|f| ui::ui(f, &mut app))?;

        // 更新应用定时器
        app.update_tick();

        // 以50毫秒的间隔检查事件
        if event::poll(std::time::Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
        {
            // 处理键盘事件，如果返回true则退出
            if app.handle_event(&Event::Key(key))? {
                break;
            }
        }
    }

    // 恢复终端状态
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
