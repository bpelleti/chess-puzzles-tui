//! Chess Puzzles TUI. Loads `data/puzzles.json` (relative to CWD) at startup.

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use chess_puzzles_tui::app::App;
use chess_puzzles_tui::pack::{pack_path, Pack};
use chess_puzzles_tui::ui;

fn main() {
    if let Err(e) = run() {
        eprintln!("chess-puzzles-tui error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let pack = Pack::load(&pack_path())
        .map_err(|e| format!("{e}\n\nHint: run from the project root (use ./play)."))?;
    let mut app = App::new(pack);

    enable_raw_mode().map_err(|e| e.to_string())?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen).map_err(|e| e.to_string())?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out)).map_err(|e| e.to_string())?;

    let result = event_loop(&mut terminal, &mut app);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), String> {
    while !app.quit {
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(|e| e.to_string())?;
        // Timeout redraws the countdown without a keypress.
        if !event::poll(Duration::from_millis(250)).map_err(|e| e.to_string())? {
            continue;
        }
        let Event::Key(key) = event::read().map_err(|e| e.to_string())? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Esc => app.back(),
            KeyCode::Up => app.up(),
            KeyCode::Down => app.down(),
            KeyCode::Left => app.left(),
            KeyCode::Right => app.right(),
            KeyCode::Enter => app.enter(),
            KeyCode::Char(c) => app.on_char(c),
            _ => {}
        }
    }
    Ok(())
}
