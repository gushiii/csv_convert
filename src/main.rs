use crossterm::event::read;
use ratatui::{Terminal, prelude::CrosstermBackend};

mod app;
mod ui;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = app::App::new()?;

    loop {
        terminal.draw(|f| ui::ui(f, &mut app))?;
        let event = read()?;
        if app.handle_event(&event)? {
            break;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
