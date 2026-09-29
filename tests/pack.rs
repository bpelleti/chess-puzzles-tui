use chess_puzzles_tui::app::{App, Screen};
use chess_puzzles_tui::pack::{pack_path, Pack};

fn pack() -> Pack {
    Pack::load(&pack_path()).expect("data/puzzles.json loads")
}

#[test]
fn pack_loads_and_each_level_has_puzzles() {
    let p = pack();
    for lvl in 1..=3 {
        assert!(p.count(lvl) >= 50, "mate-in-{lvl} has {}", p.count(lvl));
    }
}

#[test]
fn every_puzzle_is_well_formed() {
    for z in pack().puzzles {
        assert!(!z.fen.trim().is_empty(), "{} empty fen", z.id);
        assert!(!z.puzzle_fen.trim().is_empty(), "{} empty puzzle_fen", z.id);
        // opponent move + solver's line: solver makes mate_in moves
        assert_eq!(z.moves.len(), 2 * z.mate_in as usize, "{} move count", z.id);
        assert_eq!(z.solution_san.len(), z.moves.len() - 1, "{} san", z.id);
        assert!(
            z.solution_san.last().unwrap().ends_with('#'),
            "{} not mate",
            z.id
        );
    }
}

#[test]
fn menu_navigation_stub() {
    let mut a = App::new(pack());
    a.on_char('e');
    assert_eq!(a.screen, Screen::Endings);
    a.back();
    assert_eq!(a.screen, Screen::Menu);
    a.on_char('q');
    assert!(a.quit);
}
