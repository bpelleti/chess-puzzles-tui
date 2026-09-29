use chess_puzzles_tui::app::{App, Grade, Phase, Screen};
use chess_puzzles_tui::board::{
    border_line, empty_glyph, file_label, format_solution, parse_board, piece_lines, rank_caps,
    row_cells, square_is_light, BOARD_WIDTH,
};
use chess_puzzles_tui::pack::{pack_path, Pack};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn pack() -> Pack {
    Pack::load(&pack_path()).expect("data/puzzles.json loads")
}

fn render(app: &App, w: u16, h: u16) -> String {
    let backend = TestBackend::new(w, h);
    let mut term = Terminal::new(backend).expect("test terminal");
    term.draw(|f| chess_puzzles_tui::ui::draw(f, app))
        .expect("draw");
    let buf = term.backend().buffer();
    let mut out = String::new();
    for y in 0..buf.area().height {
        for x in 0..buf.area().width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn file_labels_sit_on_the_piece_letters() {
    assert_eq!(file_label().len(), 28);
    assert_eq!(border_line().len(), 28);
    let cells = " X ".repeat(8);
    let (left, right) = rank_caps(8);
    let rank = format!("{left}{cells}{right}");
    assert_eq!(rank.len(), BOARD_WIDTH);
    let files = file_label();
    for i in 0..8 {
        let at = left.len() + i * 3 + 1;
        assert_eq!(rank.chars().nth(at), Some('X'), "piece column {i}");
        assert_eq!(
            files.chars().nth(at),
            Some((b'a' + i as u8) as char),
            "file column {i}"
        );
    }
    assert!(!square_is_light(0, 1), "a1 is dark");
    assert!(square_is_light(7, 1), "h1 is light");
    assert_eq!(empty_glyph(false), " . ");
    assert_eq!(empty_glyph(true), " : ");
}

#[test]
fn parses_a_sparse_position_and_lists_pieces() {
    let fen = "8/8/8/4k3/8/3Q4/8/4K3 w - - 0 1";
    let board = parse_board(fen).expect("fen");
    assert_eq!(board[3][4], Some('k')); // e5
    assert_eq!(board[5][3], Some('Q')); // d3
    assert_eq!(board[7][4], Some('K')); // e1
    let cells = row_cells(&board, 7);
    assert!(!cells[0].light);
    assert!(cells[0].piece.is_none());
    assert!(cells[7].light);
    let lines: Vec<_> = piece_lines(&board).into_iter().map(|l| l.text).collect();
    assert_eq!(lines[0], "2 white    1 black");
    assert!(lines.iter().any(|l| l == "White  K  e1"));
    assert!(lines.iter().any(|l| l == "White  Q  d3"));
    assert!(lines.iter().any(|l| l == "Black  K  e5"));
}

#[test]
fn rejects_a_short_fen() {
    assert!(parse_board("8/8 w - - 0 1").is_err());
    assert!(parse_board("9/8/8/8/8/8/8/8 w").is_err());
}

#[test]
fn formats_the_solver_line() {
    let san = |xs: &[&str]| xs.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
    assert_eq!(format_solution(&san(&["Rh1#"])), "1. Rh1#");
    assert_eq!(
        format_solution(&san(&["Kh2", "Bf1", "g3#"])),
        "1. Kh2 Bf1  2. g3#"
    );
    assert_eq!(
        format_solution(&san(&["Qh5", "g6", "Qh8+", "Kf7", "Qg7#"])),
        "1. Qh5 g6  2. Qh8+ Kf7  3. Qg7#"
    );
}

#[test]
fn every_puzzle_board_parses_and_matches_the_side_to_move() {
    for puzzle in pack().puzzles {
        let board =
            parse_board(&puzzle.puzzle_fen).unwrap_or_else(|e| panic!("{}: {e}", puzzle.id));
        let kings = board
            .iter()
            .flatten()
            .filter(|c| matches!(c, Some('K' | 'k')))
            .count();
        assert_eq!(kings, 2, "{}", puzzle.id);
        let side = puzzle.puzzle_fen.split_whitespace().nth(1);
        let expect = if puzzle.side_to_move == "white" {
            "w"
        } else {
            "b"
        };
        assert_eq!(side, Some(expect), "{}", puzzle.id);
        assert!(
            !format_solution(&puzzle.solution_san).is_empty(),
            "{}",
            puzzle.id
        );
    }
}

#[test]
fn drill_picks_by_level_and_keeps_a_separate_place() {
    let mut app = App::new(pack());
    app.on_char('e');
    assert_eq!(app.screen, Screen::Endings);
    assert_eq!(app.phase, Phase::Setup);
    assert_eq!(app.level, 1);
    assert_eq!(app.timer_secs, 60);
    let level1: Vec<_> = app.pack.by_level(1).map(|p| p.id.clone()).collect();
    let level2: Vec<_> = app.pack.by_level(2).map(|p| p.id.clone()).collect();
    assert_eq!(app.current().unwrap().id, level1[0]);
    assert_eq!(app.current().unwrap().mate_in, 1);

    app.on_char('n');
    assert_eq!(app.current().unwrap().id, level1[1]);
    app.left();
    assert_eq!(app.current().unwrap().id, level1[0]);
    app.left();
    assert_eq!(app.current().unwrap().id, level1[level1.len() - 1]);
    app.right();
    assert_eq!(app.current().unwrap().id, level1[0]);

    app.on_char('+');
    assert_eq!(app.timer_secs, 90);
    app.on_char('1');
    assert_eq!(app.timer_secs, 90, "same level keeps the timer");
    app.on_char('2');
    assert_eq!(app.level, 2);
    assert_eq!(app.timer_secs, 180);
    assert_eq!(app.current().unwrap().id, level2[0]);
    assert_eq!(app.current().unwrap().mate_in, 2);
    app.on_char('n');
    app.on_char('1');
    assert_eq!(
        app.current().unwrap().id,
        level1[0],
        "level 1 place was not moved"
    );
    app.on_char('2');
    assert_eq!(app.current().unwrap().id, level2[1]);
}

#[test]
fn timer_clamps_and_ignores_adjustments_while_solving() {
    let mut app = App::new(pack());
    app.on_char('e');
    for _ in 0..10 {
        app.on_char('-');
    }
    assert_eq!(app.timer_secs, 30);
    app.on_char('3');
    assert_eq!(app.timer_secs, 300);
    for _ in 0..80 {
        app.on_char('+');
    }
    assert_eq!(app.timer_secs, 30 * 60);

    app.enter();
    assert_eq!(app.phase, Phase::Solving);
    app.on_char('-');
    app.on_char('1');
    assert_eq!(app.phase, Phase::Solving);
    assert_eq!(app.level, 3);
    assert_eq!(app.timer_secs, 30 * 60);
    app.back();
    assert_eq!(app.phase, Phase::Setup);
    assert_eq!(app.screen, Screen::Endings);
    assert!(app.attempts.is_empty());
}

#[test]
fn hint_reveal_and_grade_record_a_session_attempt() {
    let mut app = App::new(pack());
    app.on_char('e');
    let id = app.current().unwrap().id.clone();
    let first = app.current().unwrap().solution_san[0].clone();

    app.enter();
    app.on_char('y');
    assert!(
        app.attempts.is_empty(),
        "no grade before the line is revealed"
    );
    assert_eq!(app.phase, Phase::Solving);

    app.on_char('h');
    assert!(app.hint_used);
    assert_eq!(app.phase, Phase::Solving);

    app.on_char('r');
    assert_eq!(app.phase, Phase::Review);
    app.on_char('2');
    assert_eq!(app.level, 1, "2 during review is a grade, not mate-in-2");
    assert_eq!(app.phase, Phase::Setup);
    assert_eq!(app.attempts.len(), 1);
    assert_eq!(app.attempts[0].puzzle_id, id);
    assert_eq!(app.attempts[0].grade, Grade::SecondTry);
    assert!(app.attempts[0].hint_used);
    assert_eq!(app.attempts[0].mate_in, 1);
    assert_ne!(app.current().unwrap().id, id);
    assert_eq!(app.session_counts().second, 1);

    let screen = render(&app, 80, 24);
    assert!(screen.contains("Mate in 1"), "{screen}");
    assert!(screen.contains("2/100"), "{screen}");
    assert!(
        !screen.contains(&first),
        "next puzzle must not still show the previous hint"
    );

    app.enter();
    app.on_char('r');
    app.on_char('h');
    assert_eq!(app.attempts[1].grade, Grade::Hint);
    app.enter();
    app.on_char('r');
    app.on_char('n');
    assert_eq!(app.attempts[2].grade, Grade::Missed);
    app.enter();
    app.on_char('r');
    app.on_char('y');
    assert_eq!(app.attempts[3].grade, Grade::Solved);
    assert!(!app.attempts[3].hint_used);

    app.back();
    app.on_char('p');
    assert_eq!(app.screen, Screen::Progress);
    let progress = render(&app, 80, 24);
    assert!(progress.contains("solved 1"), "{progress}");
    assert!(progress.contains("second 1"), "{progress}");
    assert!(progress.contains("hint 1"), "{progress}");
    assert!(progress.contains("missed 1"), "{progress}");
    assert!(progress.contains("week 3"), "{progress}");
}

#[test]
fn endings_screen_shows_the_board_the_piece_list_and_the_clock() {
    let mut app = App::new(pack());
    app.on_char('e');
    let screen = render(&app, 80, 28);
    assert!(
        screen.contains("WHITE TO MOVE") || screen.contains("BLACK TO MOVE"),
        "{screen}"
    );
    assert!(screen.contains("Mate in 1"), "{screen}");
    assert!(screen.contains("1/100"), "{screen}");
    assert!(screen.contains("timer 1:00"), "{screen}");
    assert!(screen.contains("a1 lower left"), "{screen}");
    assert!(screen.contains("Enter"), "{screen}");
    assert!(screen.contains("white"), "{screen}");
    assert!(!screen.contains("coming in week 2"), "{screen}");
    // First puzzle has a white knight on e7.
    assert!(screen.contains("e7"), "{screen}");

    app.enter();
    app.on_char('h');
    let solving = render(&app, 80, 28);
    assert!(solving.contains("Hint:"), "{solving}");
    assert!(solving.contains("H hint"), "{solving}");

    app.on_char('r');
    let review = render(&app, 100, 30);
    assert!(review.contains("Solution:"), "{review}");
    assert!(review.contains("1. "), "{review}");
    assert!(review.contains("Y solved"), "{review}");

    // Narrow and short frames must still draw.
    let _ = render(&app, 40, 16);
    let _ = render(&app, 10, 8);
}
