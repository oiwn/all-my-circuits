pub mod app;
pub mod components;
pub mod layout;

pub fn run_tui() -> anyhow::Result<()> {
    use crossterm::{
        ExecutableCommand,
        event::{self, Event, KeyCode, KeyEvent},
        terminal,
    };
    use ratatui::{Terminal, backend::CrosstermBackend};

    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::AppState::new();

    loop {
        terminal.draw(|f| layout::draw(f, &app))?;

        // waiting for key events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                // Exit only on 'q' or 'Q'
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    _ => {
                        app.handle_key(code);
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    std::io::stdout().execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}
