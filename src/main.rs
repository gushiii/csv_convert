use crossterm::event::{self, Event};
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
        app.update_tick();
        // let event = read()?;
        // if app.handle_event(&event)? {
        //     break;
        // }

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if app.handle_event(&Event::Key(key))? {
                    break;
                }
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    Ok(())
}
