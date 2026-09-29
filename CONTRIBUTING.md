# Contributing

Thanks for taking an interest in **chess-puzzles-tui**.

## Build

```bash
git clone https://github.com/bpelleti/chess-puzzles-tui.git
cd chess-puzzles-tui
cargo build --release
chmod +x play   # the clone may not keep the executable bit
./play
# or: cargo run
```

Requires Rust edition 2021 (MSRV 1.85). A color-capable terminal (`TERM=xterm-256color`) is recommended. `./play` cds to the project root, then rebuilds the release binary when `src/`, `Cargo.toml`, or `Cargo.lock` is newer than it.

## Test

```bash
cargo test
```

`tests/pack.rs` checks the committed puzzle pack and menu navigation. `tests/drill.rs` checks the ASCII board, the timer, hint / reveal / grade, and a few render sizes.

## Coding notes

- Keep behavior SSH-safe: fixed-width ASCII only. Wide glyphs and emoji break the column alignment.
- Screen flow lives in `src/app.rs`. Drawing lives in `src/ui.rs`. The board and piece list live in `src/board.rs`. Pack loading lives in `src/pack.rs`.
- The puzzle pack `data/puzzles.json` is committed. Rebuild it only with `tools/make_pack.py` when you mean to change the pack.
- Do not commit `data/progress*.json`. Those files are runtime history (still to be written) and are already listed in `.gitignore`.
- Grades are self-reported. Do not add engine validation of the player's moves unless that is discussed first.
- Prefer small, focused PRs. `rustfmt` defaults are fine.

## License

By contributing you agree your changes are licensed under the MIT License (see `LICENSE`).
