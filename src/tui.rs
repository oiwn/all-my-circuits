pub mod app;
pub mod components;
pub mod layout;

use std::io;

pub fn run_tui() -> anyhow::Result<()> {
    use crossterm::{
        ExecutableCommand,
        event::{self, Event, KeyCode, KeyEvent},
        terminal,
    };
    use ratatui::{Terminal, backend::CrosstermBackend};

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = app::AppState::new();

    loop {
        terminal.draw(|f| layout::draw(f, &app))?;

        // Wait for key event
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    // terminal.draw(|f| layout::draw(f, &app))?;

    terminal::disable_raw_mode()?;
    io::stdout().execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}
