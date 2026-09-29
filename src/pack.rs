//! Puzzle pack (`data/puzzles.json`) produced once by `tools/make_pack.py`.
//!
//! Convention: `fen` + `moves` are the raw Lichess CSV fields (`fen` is the
//! position BEFORE the opponent's move `moves[0]`). `puzzle_fen` is the
//! position the solver sees (that move already applied), and
//! `solution_uci` / `solution_san` are the solver's line ending in mate.

use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Puzzle {
    pub id: String,
    pub fen: String,
    pub moves: Vec<String>,
    pub puzzle_fen: String,
    pub side_to_move: String,
    pub solution_uci: Vec<String>,
    pub solution_san: Vec<String>,
    pub rating: u32,
    #[serde(default)]
    pub popularity: i32,
    #[serde(default)]
    pub nb_plays: u32,
    pub themes: Vec<String>,
    pub mate_in: u8,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pack {
    pub source: String,
    pub puzzles: Vec<Puzzle>,
}

impl Pack {
    pub fn load(path: &Path) -> Result<Pack, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        serde_json::from_str(&raw).map_err(|e| format!("bad puzzle pack {}: {e}", path.display()))
    }

    /// Number of puzzles for a mate-in level (1..=3).
    pub fn count(&self, mate_in: u8) -> usize {
        self.puzzles.iter().filter(|p| p.mate_in == mate_in).count()
    }

    pub fn by_level(&self, mate_in: u8) -> impl Iterator<Item = &Puzzle> {
        self.puzzles.iter().filter(move |p| p.mate_in == mate_in)
    }
}

/// Pack path relative to the CWD (`./play` cds to the project root).
pub fn pack_path() -> PathBuf {
    PathBuf::from("data").join("puzzles.json")
}
