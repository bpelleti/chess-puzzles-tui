//! Rendering. Fixed-width ASCII only (SSH/Tailscale safe, no wide unicode).

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{symbols, Frame};

use crate::app::{fmt_clock, grade_label, App, Phase, Screen, MENU_ITEMS};
use crate::board::{
    border_line, empty_glyph, file_label, format_solution, parse_board, piece_lines, rank_caps,
    row_cells, PieceLine, BOARD_WIDTH,
};
use crate::pack::Puzzle;

const GOLD: Color = Color::Rgb(240, 200, 80);
const CYAN: Color = Color::Rgb(100, 190, 230);
const GREEN: Color = Color::Rgb(110, 210, 120);
const RED: Color = Color::Rgb(230, 90, 90);
const AMBER: Color = Color::Rgb(230, 160, 70);
const MUTED: Color = Color::Rgb(130, 135, 145);
const TEXT: Color = Color::Rgb(215, 215, 215);
const WHITE_PIECE: Color = Color::Rgb(245, 240, 220);
const BLACK_PIECE: Color = Color::Rgb(120, 210, 235);
const LIGHT_SQ: Color = Color::Rgb(78, 84, 96);
const DARK_SQ: Color = Color::Rgb(38, 42, 50);

const BANNER: [&str; 5] = [
    "+==========================================+",
    "|                                          |",
    "|     C H E S S    P U Z Z L E S           |",
    "|     mate-in-N  *  set up a real board    |",
    "+==========================================+",
];

/// Plain ASCII border set (no box-drawing glyphs).
const ASCII: symbols::border::Set = symbols::border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

fn block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(ASCII)
        .border_style(Style::default().fg(MUTED))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ))
}

fn status_line(app: &App) -> Line<'static> {
    let p = &app.pack;
    Line::from(vec![
        Span::styled(" Pack: ", Style::default().fg(MUTED)),
        Span::styled(
            format!(
                "M1 {}  M2 {}  M3 {}  ({} total)",
                p.count(1),
                p.count(2),
                p.count(3),
                p.puzzles.len()
            ),
            Style::default().fg(GREEN),
        ),
        Span::styled("   Lichess CC0", Style::default().fg(MUTED)),
    ])
}

pub fn draw(f: &mut Frame, app: &App) {
    let [body, status, footer] = Layout::vertical([
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(f.area());

    match app.screen {
        Screen::Menu => draw_menu(f, app, body),
        Screen::Endings => draw_endings(f, app, body),
        Screen::Progress => draw_progress(f, app, body),
    }
    f.render_widget(Paragraph::new(status_line(app)), status);
    let keys = match app.screen {
        Screen::Menu => " Up/Down Enter  E Endings  P Progress  Q/Esc Quit",
        Screen::Endings => match app.phase {
            Phase::Setup => " 1/2/3 level  N/B puzzle  +/- time  Enter start  Q/Esc menu",
            Phase::Solving => " H hint  R reveal  Q/Esc setup",
            Phase::Review => " Y solved  2 second try  H hint  N missed  Q/Esc setup",
        },
        Screen::Progress => " Q/Esc back",
    };
    f.render_widget(
        Paragraph::new(Span::styled(keys, Style::default().fg(CYAN))),
        footer,
    );
}

fn draw_menu(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = BANNER
        .iter()
        .map(|s| Line::from(Span::styled(format!("  {s}"), Style::default().fg(GOLD))))
        .collect();
    lines.push(Line::from(""));
    for (i, (key, label)) in MENU_ITEMS.iter().enumerate() {
        let sel = i == app.menu_sel;
        let marker = if sel { " > " } else { "   " };
        let st = if sel {
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT)
        };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(GOLD)),
            Span::styled(format!("[{key}] "), Style::default().fg(CYAN)),
            Span::styled(*label, st),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "   Set the puzzle up on a real board, solve it, then check yourself.",
        Style::default().fg(MUTED),
    )));
    f.render_widget(Paragraph::new(lines).block(block("Chess Puzzles")), area);
}

fn draw_endings(f: &mut Frame, app: &App, area: Rect) {
    let puzzle = app.current().cloned();
    let mut lines = vec![
        meta_line(app, puzzle.as_ref()),
        clock_line(app, puzzle.as_ref()),
    ];
    match puzzle
        .as_ref()
        .and_then(|p| parse_board(&p.puzzle_fen).ok())
    {
        Some(grid) => lines.extend(board_lines(&grid)),
        None => lines.push(Line::from(Span::styled(
            "  No puzzle at this level.",
            Style::default().fg(TEXT),
        ))),
    }
    lines.push(instruction_line(app, puzzle.as_ref()));
    if app.phase == Phase::Review {
        if let Some(blurb) = puzzle.as_ref().and_then(|p| theme_blurb(&p.themes)) {
            lines.push(Line::from(Span::styled(
                format!("  {blurb}"),
                Style::default().fg(MUTED),
            )));
        }
    }
    lines.push(session_line(app));
    f.render_widget(Paragraph::new(lines).block(block("Endings")), area);
}

fn meta_line(app: &App, puzzle: Option<&Puzzle>) -> Line<'static> {
    let rating = puzzle
        .map(|p| p.rating.to_string())
        .unwrap_or_else(|| "-".into());
    let id = puzzle.map(|p| p.id.clone()).unwrap_or_else(|| "-".into());
    Line::from(vec![
        Span::styled(
            format!("  Mate in {}   ", app.level),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled(app.position_label(), Style::default().fg(TEXT)),
        Span::styled(format!("    {rating}"), Style::default().fg(TEXT)),
        Span::styled(format!("    {id}"), Style::default().fg(MUTED)),
    ])
}

fn clock_line(app: &App, puzzle: Option<&Puzzle>) -> Line<'static> {
    let (side, side_color) = match puzzle.map(|p| p.side_to_move.as_str()) {
        Some("white") => ("WHITE TO MOVE".to_string(), GOLD),
        Some("black") => ("BLACK TO MOVE".to_string(), CYAN),
        Some(other) => (other.to_string(), TEXT),
        None => ("NO PUZZLE".to_string(), MUTED),
    };
    let (clock, clock_color) = clock_text(app);
    Line::from(vec![
        Span::styled(
            format!("  {side:<16}"),
            Style::default().fg(side_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {clock}"),
            Style::default()
                .fg(clock_color)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn clock_text(app: &App) -> (String, Color) {
    let limit = app.timer_secs;
    let elapsed = app.elapsed_secs();
    match app.phase {
        Phase::Setup => (format!("timer {}", fmt_clock(limit)), TEXT),
        Phase::Solving if elapsed < limit => {
            let left = limit - elapsed;
            let color = if left <= 10 { AMBER } else { GREEN };
            (format!("{} left", fmt_clock(left)), color)
        }
        Phase::Solving => (format!("over +{}", fmt_clock(elapsed - limit)), RED),
        Phase::Review if elapsed <= limit => (format!("used {}", fmt_clock(elapsed)), GREEN),
        Phase::Review => (format!("used {} (over)", fmt_clock(elapsed)), RED),
    }
}

fn board_lines(grid: &[[Option<char>; 8]; 8]) -> Vec<Line<'static>> {
    let pieces = piece_lines(grid);
    let mut rows: Vec<Vec<Span<'static>>> = Vec::new();
    rows.push(vec![Span::styled(format!("  {}", file_label()), muted())]);
    rows.push(vec![Span::styled(format!("  {}", border_line()), muted())]);
    for row in 0..8 {
        let rank = (8 - row) as u8;
        let (left, right) = rank_caps(rank);
        let mut spans = vec![Span::styled(format!("  {left}"), muted())];
        for cell in row_cells(grid, row) {
            spans.push(cell_span(cell.piece, cell.light));
        }
        spans.push(Span::styled(right, muted()));
        rows.push(spans);
    }
    rows.push(vec![Span::styled(format!("  {}", border_line()), muted())]);
    rows.push(vec![Span::styled(format!("  {}", file_label()), muted())]);

    let mut lines = Vec::with_capacity(rows.len() + pieces.len());
    for (i, mut spans) in rows.into_iter().enumerate() {
        let used: usize = spans.iter().map(Span::width).sum();
        // The two-space indent is already inside the spans. Pad to the
        // indented board width so the piece column lines up.
        let target = 2 + BOARD_WIDTH;
        if used < target {
            spans.push(Span::raw(" ".repeat(target - used)));
        }
        if let Some(piece) = pieces.get(i) {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(piece.text.clone(), piece_style(piece)));
        }
        lines.push(Line::from(spans));
    }
    for piece in pieces.iter().skip(lines.len()) {
        lines.push(Line::from(Span::styled(
            format!("  {}", piece.text),
            piece_style(piece),
        )));
    }
    lines
}

fn cell_span(piece: Option<char>, light: bool) -> Span<'static> {
    let bg = if light { LIGHT_SQ } else { DARK_SQ };
    match piece {
        Some(ch) if ch.is_ascii_uppercase() => Span::styled(
            format!(" {ch} "),
            Style::default()
                .fg(WHITE_PIECE)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        ),
        Some(ch) => Span::styled(
            format!(" {ch} "),
            Style::default()
                .fg(BLACK_PIECE)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        ),
        None => Span::styled(empty_glyph(light), Style::default().fg(MUTED).bg(bg)),
    }
}

fn piece_style(line: &PieceLine) -> Style {
    if line.text.starts_with('W') || (line.white && line.text.starts_with(' ')) {
        Style::default().fg(GOLD)
    } else if line.text.starts_with('B') || (!line.white && line.text.starts_with(' ')) {
        Style::default().fg(CYAN)
    } else {
        Style::default().fg(TEXT)
    }
}

fn instruction_line(app: &App, puzzle: Option<&Puzzle>) -> Line<'static> {
    let elapsed = app.elapsed_secs();
    let text = match app.phase {
        Phase::Setup => {
            "White at the bottom (a1 lower left). Set the pieces, then Enter.".to_string()
        }
        Phase::Solving if app.hint_used => {
            let mv = puzzle
                .and_then(|p| p.solution_san.first())
                .map(String::as_str)
                .unwrap_or("?");
            format!("Hint: {mv}  (the rest stays hidden)")
        }
        Phase::Solving if elapsed >= app.timer_secs => {
            "Time is up. R reveals the line when you want to grade it.".to_string()
        }
        Phase::Solving => "Clock is running. Solve it on your board.".to_string(),
        Phase::Review => {
            let sol = puzzle
                .map(|p| format_solution(&p.solution_san))
                .unwrap_or_default();
            if app.hint_used {
                format!("Hint used.  Solution: {sol}")
            } else {
                format!("Solution: {sol}")
            }
        }
    };
    let color = if app.phase == Phase::Review {
        GOLD
    } else {
        TEXT
    };
    Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(color),
    ))
}

fn session_line(app: &App) -> Line<'static> {
    let t = app.session_counts();
    let summary = format!(
        "solved {}  |  second {}  |  hint {}  |  missed {}",
        t.solved, t.second, t.hint, t.missed
    );
    let text = match app.attempts.last() {
        Some(a) => format!("Last {} {}   {summary}", a.puzzle_id, grade_label(a.grade)),
        None => format!("Session  {summary}"),
    };
    Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(MUTED),
    ))
}

fn theme_blurb(themes: &[String]) -> Option<String> {
    let kept: Vec<&str> = themes
        .iter()
        .map(String::as_str)
        .filter(|t| !matches!(*t, "mate" | "endgame" | "oneMove") && !t.starts_with("mateIn"))
        .collect();
    if kept.is_empty() {
        None
    } else {
        Some(kept.join(" "))
    }
}

fn draw_progress(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  This session (not saved yet)",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        )),
        tally_line("All", app.session_counts()),
    ];
    for level in 1..=3 {
        lines.push(tally_line(
            &format!("Mate in {level}"),
            app.session_counts_for(level),
        ));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Saved history, points, and a record file land in week 3.",
        Style::default().fg(CYAN),
    )));
    f.render_widget(Paragraph::new(lines).block(block("Progress")), area);
}

fn tally_line(label: &str, t: crate::app::SessionTally) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {label:<10}"), Style::default().fg(MUTED)),
        Span::styled(
            format!("solved {:<3}", t.solved),
            Style::default().fg(GREEN),
        ),
        Span::styled(format!("second {:<3}", t.second), Style::default().fg(GOLD)),
        Span::styled(format!("hint {:<3}", t.hint), Style::default().fg(CYAN)),
        Span::styled(format!("missed {:<3}", t.missed), Style::default().fg(TEXT)),
    ])
}

fn muted() -> Style {
    Style::default().fg(MUTED)
}
