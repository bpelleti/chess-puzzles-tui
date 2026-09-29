# Chess Puzzles TUI

Terminal trainer for **mate-in-N endings** with a physical-board workflow: the app
shows a puzzle (FEN + side to move), you set it up on a real board, solve it, then
check yourself. Fixed-width ASCII, SSH/Tailscale friendly. Sibling of the Play Nine
and Yahtzee scorekeepers.

**Stack:** Rust 1.85 - ratatui 0.28.1 - crossterm - serde_json (offline; no network at runtime)

**Site:** https://bpelleti.github.io/chess-puzzles-tui/

## Run

```bash
git clone https://github.com/bpelleti/chess-puzzles-tui.git
cd chess-puzzles-tui
chmod +x play    # once, if the clone did not keep the executable bit
./play           # builds the release binary on first run, then launches it
cargo test       # pack, board, and drill checks
```

The working copy on this machine is `~/Projects/ChessPuzzles` and uses the same remote.
Run from the project root (`./play` does this): the pack is read from `data/puzzles.json`
relative to the CWD. Over SSH, `export TERM=xterm-256color` if colors look off.

| Screen        | Keys                                                                 |
|---------------|----------------------------------------------------------------------|
| Menu          | Up/Down + Enter, `E` Endings, `P` Progress, `Q`/Esc quit             |
| Endings setup | `1`/`2`/`3` level, `N`/Right next, `B`/Left previous, `+`/`-` time, Enter start, `Q`/Esc menu |
| Solving       | `H` first-move hint, `R` reveal the line, `Q`/Esc back to setup (no grade) |
| Review        | `Y` solved, `2` second try, `H` used a hint, `N` missed, `Q`/Esc cancel |
| Progress      | `Q`/Esc back. Shows this session's grades. Saving them is [issue #1](https://github.com/bpelleti/chess-puzzles-tui/issues/1). |

## Endings drill

1. Pick mate in **1 / 2 / 3**. Each level keeps its own place in the pack (rating order).
2. The board is white at the bottom, a1 at the lower left. Empty dark squares are `.`, light squares are `:`. Uppercase letters are white pieces and lowercase are black. The piece list is what you set on a real board.
3. Timer defaults are **1:00 / 3:00 / 5:00**. `+` and `-` step 30 seconds (0:30 through 30:00) before Enter starts the clock. The clock turns amber under 10 seconds and counts up in red after it runs out; it does not auto-fail you.
4. `H` shows only the first move. `R` shows the whole mate and stops the clock.
5. Grade yourself. That stores a session result and loads the next puzzle. Esc before grading drops the attempt.

Grades last until you quit. Nothing is written to disk yet.

## Puzzle pack

`data/puzzles.json` (~130 KB, committed) holds **300 puzzles: 100 each of mateIn1,
mateIn2, mateIn3**, ratings spread over 600-2000 (7 bands of 200), preferring
Popularity >= 80 and more NbPlays. It was made once with:

```bash
pip install zstandard chess
python3 tools/make_pack.py        # --per-level 100 --max-rows 1500000
```

The script streams `https://database.lichess.org/lichess_db_puzzle.csv.zst`, decompresses
on the fly and **stops early** once each level/band has enough candidates (about 110k rows
scanned); the full dump is never stored.

**FEN convention** (both forms are stored per puzzle):

- `fen`, `moves` - raw Lichess CSV fields. `fen` is the position **before** the opponent's
  move `moves[0]`; the solver's line starts at `moves[1]`.
- `puzzle_fen` - the position to show/set up (`moves[0]` already applied), plus
  `side_to_move`.
- `solution_uci` / `solution_san` - the solver's line (alternating with replies), ending in
  mate, e.g. `Kh2 Bf1 g3#`.

Other fields: `id`, `rating`, `popularity`, `nb_plays`, `themes`, `mate_in` (1/2/3),
`url` (`https://lichess.org/training/<id>`).

Puzzle data: [Lichess open database](https://database.lichess.org/), released under
**CC0**. Thank you, Lichess.

## Scope / roadmap

- **Week 1 (done):** puzzle pack, menu, stub Endings and Progress screens, tests.
- **Week 2 (done):** Endings drill - pick puzzle by level, ASCII board + piece list for
  board setup, configurable timer, reveal solution, self-grade (Y / 2nd try / hint / N).
  Session tallies show on Progress; they are not saved.
- **Week 3:** [Save progress](https://github.com/bpelleti/chess-puzzles-tui/issues/1) —
  history JSON (`data/progress.json`), points, per-level stats.
- **Week 4:** repo and Pages are up. Still open:
  [celebration on a clean solve](https://github.com/bpelleti/chess-puzzles-tui/issues/2).

Out of scope for the Lean MVP: openings trainer, typed-move validation.

## License

MIT. See [LICENSE](LICENSE). Puzzle data is Lichess CC0, separate from the code license.
