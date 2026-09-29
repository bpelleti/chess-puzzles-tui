#!/usr/bin/env python3
"""One-time puzzle-pack builder for chess-puzzles-tui.

Streams the Lichess CC0 puzzle CSV (zstd) over HTTP, decompresses on the fly,
and STOPS EARLY once enough mate-in-1/2/3 candidates are collected. Nothing but
data/puzzles.json is written; the full dump is never stored.

Requires: pip install zstandard chess   (python-chess pre-applies the opponent's
first move and produces SAN for the solution, so the Rust app needs no chess lib.)

Usage:  python3 tools/make_pack.py [--per-level 100] [--max-rows 1500000]
"""
import argparse, csv, io, json, math, os, sys, urllib.request

import chess
import zstandard

URL = "https://database.lichess.org/lichess_db_puzzle.csv.zst"
LEVELS = {"mateIn1": 1, "mateIn2": 2, "mateIn3": 3}
BANDS = [(600 + 200 * i, 800 + 200 * i) for i in range(7)]  # 600..2000
MIN_POP, MIN_PLAYS = 80, 300


def band_of(r):
    for i, (lo, hi) in enumerate(BANDS):
        if lo <= r < hi:
            return i
    return None


def score(row):
    return int(row["Popularity"]) * math.log10(int(row["NbPlays"]) + 10)


def convert(row, mate_in):
    b = chess.Board(row["FEN"])
    moves = row["Moves"].split()
    b.push_uci(moves[0])  # opponent's move -> puzzle position
    puzzle_fen = b.fen()
    side = "white" if b.turn == chess.WHITE else "black"
    san = []
    for m in moves[1:]:
        mv = chess.Move.from_uci(m)
        san.append(b.san(mv))
        b.push(mv)
    assert b.is_checkmate(), row["PuzzleId"]
    return {
        "id": row["PuzzleId"],
        "fen": row["FEN"],
        "moves": moves,
        "puzzle_fen": puzzle_fen,
        "side_to_move": side,
        "solution_uci": moves[1:],
        "solution_san": san,
        "rating": int(row["Rating"]),
        "popularity": int(row["Popularity"]),
        "nb_plays": int(row["NbPlays"]),
        "themes": row["Themes"].split(),
        "mate_in": mate_in,
        "url": f"https://lichess.org/training/{row['PuzzleId']}",
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--per-level", type=int, default=100)
    ap.add_argument("--max-rows", type=int, default=1_500_000)
    ap.add_argument("--out", default=os.path.join(os.path.dirname(__file__), "..", "data", "puzzles.json"))
    a = ap.parse_args()

    per_band = math.ceil(a.per_level / len(BANDS))
    want = per_band * 4  # collect 4x candidates per band, then keep the best
    pool = {lvl: [[] for _ in BANDS] for lvl in LEVELS.values()}

    def full():
        return all(len(b) >= want for bands in pool.values() for b in bands)

    req = urllib.request.Request(URL, headers={"User-Agent": "chess-puzzles-tui make_pack"})
    rows = 0
    with urllib.request.urlopen(req) as resp:
        text = io.TextIOWrapper(zstandard.ZstdDecompressor().stream_reader(resp), encoding="utf-8")
        for row in csv.DictReader(text):
            rows += 1
            themes = row["Themes"].split()
            lvl = next((LEVELS[t] for t in themes if t in LEVELS), None)
            if lvl is None or int(row["Popularity"]) < MIN_POP or int(row["NbPlays"]) < MIN_PLAYS:
                pass
            else:
                bi = band_of(int(row["Rating"]))
                if bi is not None and len(pool[lvl][bi]) < want:
                    pool[lvl][bi].append(row)
            if rows % 100_000 == 0:
                print(f"  scanned {rows:,} rows", file=sys.stderr)
            if full() or rows >= a.max_rows:
                break  # stop early; closing the response aborts the download

    out = []
    for lvl, bands in pool.items():
        chosen = []
        for b in bands:
            chosen += sorted(b, key=score, reverse=True)[:per_band]
        chosen = sorted(chosen, key=score, reverse=True)[: a.per_level]
        out += [convert(r, lvl) for r in sorted(chosen, key=lambda r: int(r["Rating"]))]

    pack = {
        "source": "Lichess puzzle database (CC0), https://database.lichess.org/",
        "convention": "fen/moves are raw Lichess CSV (fen is BEFORE the opponent's move moves[0]). "
                      "puzzle_fen is the position shown to the solver (moves[0] already applied); "
                      "solution_uci/solution_san are the solver's line, ending in mate.",
        "rows_scanned": rows,
        "puzzles": out,
    }
    with open(a.out, "w") as f:
        json.dump(pack, f, separators=(",", ":"))
    counts = {l: sum(p["mate_in"] == l for p in out) for l in LEVELS.values()}
    print(f"scanned {rows:,} rows; wrote {len(out)} puzzles {counts} -> {a.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
