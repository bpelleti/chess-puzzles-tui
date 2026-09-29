//! App state. Menu, endings drill, and a session-only progress tally.
//!
//! Grades live in memory for this process. Writing `data/progress.json` is
//! the week 3 leg.

use std::time::Instant;

use crate::pack::{Pack, Puzzle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Endings,
    Progress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Board and piece list are up. The clock has not started.
    Setup,
    /// Clock is running. The solution is hidden.
    Solving,
    /// Solution is visible. Waiting for a self-grade.
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Solved,
    SecondTry,
    Hint,
    Missed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub puzzle_id: String,
    pub mate_in: u8,
    pub grade: Grade,
    pub hint_used: bool,
    pub elapsed_secs: u64,
    pub timer_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SessionTally {
    pub solved: u32,
    pub second: u32,
    pub hint: u32,
    pub missed: u32,
}

pub const MENU_ITEMS: [(&str, &str); 3] = [
    ("E", "Endings drill (mate in N)"),
    ("P", "Progress"),
    ("Q", "Quit"),
];

const TIMER_MIN: u64 = 30;
const TIMER_MAX: u64 = 30 * 60;
const TIMER_STEP: u64 = 30;

pub struct App {
    pub pack: Pack,
    pub screen: Screen,
    pub menu_sel: usize,
    pub level: u8,
    pub phase: Phase,
    pub timer_secs: u64,
    pub hint_used: bool,
    pub attempts: Vec<Attempt>,
    pub quit: bool,
    /// Place in each mate-in level (pack order, which is rating order).
    index: [usize; 3],
    started: Option<Instant>,
    elapsed_at_review: u64,
}

impl App {
    pub fn new(pack: Pack) -> Self {
        App {
            pack,
            screen: Screen::Menu,
            menu_sel: 0,
            level: 1,
            phase: Phase::Setup,
            timer_secs: default_timer(1),
            hint_used: false,
            attempts: Vec::new(),
            quit: false,
            index: [0; 3],
            started: None,
            elapsed_at_review: 0,
        }
    }

    pub fn current(&self) -> Option<&Puzzle> {
        let n = self.pack.count(self.level);
        if n == 0 {
            None
        } else {
            self.pack.by_level(self.level).nth(self.puzzle_index())
        }
    }

    pub fn puzzle_index(&self) -> usize {
        let n = self.pack.count(self.level);
        if n == 0 {
            0
        } else {
            self.index[self.slot()] % n
        }
    }

    pub fn position_label(&self) -> String {
        let n = self.pack.count(self.level);
        if n == 0 {
            "0/0".into()
        } else {
            format!("{}/{}", self.puzzle_index() + 1, n)
        }
    }

    pub fn elapsed_secs(&self) -> u64 {
        match self.phase {
            Phase::Setup => 0,
            Phase::Solving => self.started.map(|t| t.elapsed().as_secs()).unwrap_or(0),
            Phase::Review => self.elapsed_at_review,
        }
    }

    pub fn session_counts(&self) -> SessionTally {
        self.tally(|_| true)
    }

    pub fn session_counts_for(&self, mate_in: u8) -> SessionTally {
        self.tally(|a| a.mate_in == mate_in)
    }

    pub fn activate(&mut self, idx: usize) {
        match idx {
            0 => {
                self.abandon_attempt();
                self.screen = Screen::Endings;
            }
            1 => self.screen = Screen::Progress,
            _ => self.quit = true,
        }
    }

    pub fn on_char(&mut self, c: char) {
        let c = c.to_ascii_lowercase();
        match self.screen {
            Screen::Menu => match c {
                'e' => self.activate(0),
                'p' => self.activate(1),
                'q' => self.activate(2),
                _ => {}
            },
            Screen::Endings => self.on_endings_char(c),
            Screen::Progress => {
                if c == 'q' {
                    self.back();
                }
            }
        }
    }

    pub fn up(&mut self) {
        if self.screen == Screen::Menu {
            self.menu_sel = (self.menu_sel + MENU_ITEMS.len() - 1) % MENU_ITEMS.len();
        }
    }

    pub fn down(&mut self) {
        if self.screen == Screen::Menu {
            self.menu_sel = (self.menu_sel + 1) % MENU_ITEMS.len();
        }
    }

    pub fn left(&mut self) {
        if self.screen == Screen::Endings {
            self.cycle_puzzle(-1);
        }
    }

    pub fn right(&mut self) {
        if self.screen == Screen::Endings {
            self.cycle_puzzle(1);
        }
    }

    pub fn enter(&mut self) {
        match self.screen {
            Screen::Menu => self.activate(self.menu_sel),
            Screen::Endings => self.start(),
            Screen::Progress => {}
        }
    }

    /// Esc / q. From a running or revealed puzzle, step back to setup with
    /// no grade. From setup (or any other screen), return to the menu.
    /// From the menu, quit.
    pub fn back(&mut self) {
        match self.screen {
            Screen::Menu => self.quit = true,
            Screen::Endings if self.phase != Phase::Setup => self.abandon_attempt(),
            _ => self.screen = Screen::Menu,
        }
    }

    fn on_endings_char(&mut self, c: char) {
        match c {
            'q' => self.back(),
            '1' | '3' => self.set_level(c as u8 - b'0'),
            '2' => {
                if self.phase == Phase::Review {
                    self.grade(Grade::SecondTry);
                } else {
                    self.set_level(2);
                }
            }
            'n' => match self.phase {
                Phase::Setup => self.cycle_puzzle(1),
                Phase::Review => self.grade(Grade::Missed),
                Phase::Solving => {}
            },
            'b' => self.cycle_puzzle(-1),
            '+' | '=' => self.adjust_timer(1),
            '-' | '_' => self.adjust_timer(-1),
            ' ' => self.start(),
            'r' => self.reveal(),
            'h' => match self.phase {
                Phase::Solving => self.take_hint(),
                Phase::Review => self.grade(Grade::Hint),
                Phase::Setup => {}
            },
            'y' => self.grade(Grade::Solved),
            _ => {}
        }
    }

    fn set_level(&mut self, level: u8) {
        if self.screen != Screen::Endings || self.phase != Phase::Setup || !(1..=3).contains(&level)
        {
            return;
        }
        if self.level != level {
            self.level = level;
            self.timer_secs = default_timer(level);
        }
        self.hint_used = false;
    }

    fn adjust_timer(&mut self, steps: i64) {
        if self.screen != Screen::Endings || self.phase != Phase::Setup {
            return;
        }
        let next = self.timer_secs as i64 + steps * TIMER_STEP as i64;
        self.timer_secs = next.clamp(TIMER_MIN as i64, TIMER_MAX as i64) as u64;
    }

    fn start(&mut self) {
        if self.screen != Screen::Endings || self.phase != Phase::Setup || self.current().is_none()
        {
            return;
        }
        self.started = Some(Instant::now());
        self.elapsed_at_review = 0;
        self.hint_used = false;
        self.phase = Phase::Solving;
    }

    fn take_hint(&mut self) {
        if self.screen != Screen::Endings || self.phase != Phase::Solving {
            return;
        }
        let has_move = self.current().is_some_and(|p| !p.solution_san.is_empty());
        if has_move {
            self.hint_used = true;
        }
    }

    fn reveal(&mut self) {
        if self.screen != Screen::Endings || self.phase != Phase::Solving {
            return;
        }
        self.elapsed_at_review = self.started.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        self.started = None;
        self.phase = Phase::Review;
    }

    fn grade(&mut self, grade: Grade) {
        if self.screen != Screen::Endings || self.phase != Phase::Review {
            return;
        }
        let Some((puzzle_id, mate_in)) = self.current().map(|p| (p.id.clone(), p.mate_in)) else {
            return;
        };
        self.attempts.push(Attempt {
            puzzle_id,
            mate_in,
            grade,
            hint_used: self.hint_used,
            elapsed_secs: self.elapsed_secs(),
            timer_secs: self.timer_secs,
        });
        self.abandon_attempt();
        self.cycle_puzzle(1);
    }

    fn cycle_puzzle(&mut self, dir: isize) {
        if self.screen != Screen::Endings || self.phase != Phase::Setup {
            return;
        }
        let n = self.pack.count(self.level);
        if n == 0 {
            return;
        }
        let next = (self.puzzle_index() as isize + dir).rem_euclid(n as isize) as usize;
        self.index[self.slot()] = next;
        self.hint_used = false;
    }

    fn abandon_attempt(&mut self) {
        self.phase = Phase::Setup;
        self.hint_used = false;
        self.started = None;
        self.elapsed_at_review = 0;
    }

    fn slot(&self) -> usize {
        (self.level as usize).saturating_sub(1).min(2)
    }

    fn tally(&self, pred: impl Fn(&Attempt) -> bool) -> SessionTally {
        let mut t = SessionTally::default();
        for attempt in self.attempts.iter().filter(|a| pred(a)) {
            match attempt.grade {
                Grade::Solved => t.solved += 1,
                Grade::SecondTry => t.second += 1,
                Grade::Hint => t.hint += 1,
                Grade::Missed => t.missed += 1,
            }
        }
        t
    }
}

/// Mate in 1 / 2 / 3 defaults: 1:00, 3:00, 5:00.
fn default_timer(level: u8) -> u64 {
    match level {
        1 => 60,
        2 => 180,
        3 => 300,
        _ => 180,
    }
}

pub fn fmt_clock(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub fn grade_label(grade: Grade) -> &'static str {
    match grade {
        Grade::Solved => "solved",
        Grade::SecondTry => "second try",
        Grade::Hint => "hint",
        Grade::Missed => "missed",
    }
}
