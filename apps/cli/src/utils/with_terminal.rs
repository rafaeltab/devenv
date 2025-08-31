use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::CrosstermBackend;
use ratatui::{Terminal};

/// Run `f` with a ratatui Terminal set up, always restoring the terminal
/// even if `f` returns an error or panics.
pub fn with_terminal<F, T>(f: F) -> io::Result<T>
where
    F: FnOnce(&mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<T>,
{
    // Setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run, catching panics so we can restore before unwinding
    let result = catch_unwind(AssertUnwindSafe(|| f(&mut terminal)));

    // Cleanup (best-effort)
    let _ = terminal.flush();
    let backend = terminal.backend_mut();
    let _ = execute!(backend, LeaveAlternateScreen, DisableMouseCapture);
    let _ = disable_raw_mode();

    // Propagate result or panic
    match result {
        Ok(r) => r, // io::Result<T> from f
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
