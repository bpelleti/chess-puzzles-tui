//! ASCII board and the piece list used to set up a physical board.
//!
//! Diagrams are white at the bottom, a1 at the lower left. Pieces are ASCII
//! letters (uppercase white, lowercase black). No wide glyphs.

/// Width of one rank line, including both rank numbers. File labels and
/// borders are padded out to this so the piece list starts in one column.
pub const BOARD_WIDTH: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellView {
    pub piece: Option<char>,
    pub light: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PieceLine {
    /// True for a white piece row or a white continuation row.
    pub white: bool,
    pub text: String,
}

/// `true` for a light square. a1 is dark, h1 is light.
pub fn square_is_light(file: usize, rank: u8) -> bool {
    (file + rank as usize) % 2 == 0
}

/// Three-column glyph for an empty square. Dark squares are `.` and light
/// squares are `:`, so the grid stays visible with colors turned off.
pub fn empty_glyph(light: bool) -> &'static str {
    if light {
        " : "
    } else {
        " . "
    }
}

/// File letters aligned over the piece character of each 3-column cell.
pub fn file_label() -> String {
    let mut s = String::from("    ");
    for file in b'a'..=b'h' {
        s.push(file as char);
        s.push_str("  ");
    }
    s
}

pub fn border_line() -> String {
    format!("  +{}+", "-".repeat(24))
}

/// Left and right caps of a rank line (`"8 |"`, `"| 8"`). Cells sit between them.
pub fn rank_caps(rank: u8) -> (String, String) {
    (format!("{rank} |"), format!("| {rank}"))
}

/// Parse the placement field of a FEN. Row 0 is rank 8 (top of the diagram).
pub fn parse_board(fen: &str) -> Result<[[Option<char>; 8]; 8], String> {
    let placement = fen.split_whitespace().next().unwrap_or("");
    let ranks: Vec<&str> = placement.split('/').collect();
    if ranks.len() != 8 {
        return Err(format!("expected 8 ranks in FEN, got {}", ranks.len()));
    }
    let mut board = [[None; 8]; 8];
    for (row, rank) in ranks.iter().enumerate() {
        let mut file = 0usize;
        for ch in rank.chars() {
            if ch.is_ascii_digit() {
                file += ch.to_digit(10).unwrap() as usize;
            } else if "prnbqkPRNBQK".contains(ch) {
                if file >= 8 {
                    return Err(format!("rank {} overflows", 8 - row));
                }
                board[row][file] = Some(ch);
                file += 1;
            } else {
                return Err(format!("bad FEN character {ch}"));
            }
        }
        if file != 8 {
            return Err(format!("rank {} has {file} squares", 8 - row));
        }
    }
    Ok(board)
}

pub fn row_cells(board: &[[Option<char>; 8]; 8], row: usize) -> [CellView; 8] {
    let rank = (8 - row) as u8;
    let mut cells = [CellView {
        piece: None,
        light: false,
    }; 8];
    for file in 0..8 {
        cells[file] = CellView {
            piece: board[row][file],
            light: square_is_light(file, rank),
        };
    }
    cells
}

/// Piece list for setting up a real board. One summary line, then white and
/// black grouped K Q R B N P, squares in file order.
pub fn piece_lines(board: &[[Option<char>; 8]; 8]) -> Vec<PieceLine> {
    const KINDS: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];
    let mut white_n = 0usize;
    let mut black_n = 0usize;
    let mut white: [Vec<(usize, usize)>; 6] = Default::default();
    let mut black: [Vec<(usize, usize)>; 6] = Default::default();
    for (row, rank) in board.iter().enumerate() {
        for (file, piece) in rank.iter().enumerate() {
            let Some(ch) = *piece else { continue };
            let Some(idx) = KINDS.iter().position(|k| *k == ch.to_ascii_uppercase()) else {
                continue;
            };
            if ch.is_ascii_uppercase() {
                white_n += 1;
                white[idx].push((row, file));
            } else {
                black_n += 1;
                black[idx].push((row, file));
            }
        }
    }
    let mut lines = vec![PieceLine {
        white: true,
        text: format!("{white_n} white    {black_n} black"),
    }];
    push_side(&mut lines, true, &white);
    push_side(&mut lines, false, &black);
    lines
}

fn push_side(out: &mut Vec<PieceLine>, white: bool, groups: &[Vec<(usize, usize)>; 6]) {
    const KINDS: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];
    let side = if white { "White" } else { "Black" };
    for (idx, squares) in groups.iter().enumerate() {
        if squares.is_empty() {
            continue;
        }
        let mut squares = squares.clone();
        squares.sort_by(|a, b| {
            let ra = 8 - a.0;
            let rb = 8 - b.0;
            a.1.cmp(&b.1).then(ra.cmp(&rb))
        });
        let names: Vec<String> = squares
            .iter()
            .map(|(row, file)| square_name(*row, *file))
            .collect();
        for (chunk_i, chunk) in names.chunks(8).enumerate() {
            let text = if chunk_i == 0 {
                format!("{side}  {}  {}", KINDS[idx], chunk.join(" "))
            } else {
                format!("          {}", chunk.join(" "))
            };
            out.push(PieceLine { white, text });
        }
    }
}

fn square_name(row: usize, file: usize) -> String {
    let rank = 8 - row;
    format!("{}{rank}", (b'a' + file as u8) as char)
}

/// Solver line with move numbers. `san` alternates solver, reply, solver, ...
/// and already ends in mate. Example: `1. Kh2 Bf1  2. g3#`.
pub fn format_solution(san: &[String]) -> String {
    let mut out = String::new();
    let mut number = 1usize;
    let mut i = 0;
    while i < san.len() {
        if !out.is_empty() {
            out.push_str("  ");
        }
        out.push_str(&format!("{number}. {}", san[i]));
        i += 1;
        if i < san.len() {
            out.push(' ');
            out.push_str(&san[i]);
            i += 1;
        }
        number += 1;
    }
    out
}
