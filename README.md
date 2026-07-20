# Rusty Dragon v0.9.9

A compact, dependency-free chess engine written in pure Rust, implementing full
FIDE-legal move generation and the UCI (Universal Chess Interface) protocol
in a single 369-line source file.

**Author:** Gokul Chandar
**Version:** v0.9.9 — July 2026
**Language:** Rust (stable, `std` only — zero external crates)
**Source:** `Rusty Dragon.rs` (369 lines, ~14.5 KB)

---

## Acknowledgments

This engine was designed, written, debugged, and verified in collaboration
with **Claude** (Anthropic), acting as the primary engineering partner
throughout the project. Claude was responsible for:

- Architecting the board representation, move generation, and search
- Writing the full legal-move generator, including castling, en passant,
  promotion, and check/checkmate/stalemate detection
- Implementing the UCI protocol loop from scratch
- Designing and running the `perft` verification suite used to prove
  correctness against known-good node counts
- Diagnosing and fixing edge cases (discovered-check en passant, simultaneous
  promotion + castling-rights edge cases, etc.)
- Iterating on the codebase based on direction and review from Gokul Chandar

In short: the correctness guarantees in this README exist *because* of a
structured, test-driven collaboration with Claude, and that contribution is
gratefully and explicitly acknowledged here. This project is a good example
of what a human directing an AI collaborator, with clear verification
standards at each step, can produce quickly and reliably.

---

## Features

### Board & Rules
- 8x8 mailbox board representation (`[i8; 64]`), no bitboards — chosen to
  keep the source small and readable rather than maximize raw speed
- Full legal move generation for all six piece types
- **Castling** — both sides, both colors, with correct handling of:
  - Castling rights tracked incrementally (king/rook moves and rook captures)
  - King may not castle out of, through, or into check
- **En passant** — including the classic discovered-check edge case (an
  en passant capture that would expose the king to a rook/queen on the same
  rank is correctly forbidden)
- **Promotion** — full underpromotion support (Q, R, B, N), not just
  auto-queen
- **Check, checkmate, and stalemate** detection
- **50-move rule** draw detection
- **Threefold repetition** draw detection (simple position-hash history)

### UCI Protocol
Supported commands:
- `uci` → returns `id name Rusty Dragon v0.9.9 July 2026`, `id author Gokul Chandar`, `uciok`
- `isready` → `readyok`
- `ucinewgame`
- `position startpos [moves ...]`
- `position fen <fen> [moves ...]`
- `go [depth N]` → returns `bestmove <move>`
- `stop`
- `quit`

Non-standard extension for testing:
- `perft <depth>` → prints the perft node count from the current position
  (not part of the UCI spec, but included because it's how the engine's
  correctness was verified — see below)

### Search & Evaluation
- Negamax with alpha-beta pruning
- Material-only evaluation (P=100, N=320, B=330, R=500, Q=900)
- No move ordering, no quiescence search, no transposition table

---

## Correctness Verification

Move generators are notoriously easy to get subtly wrong. Rather than assert
correctness, the engine was verified with **perft** (performance test) — a
standard technique that counts the exact number of legal move sequences from
a position to a fixed depth and compares against known-correct values.

| Position | Depth | Rusty Dragon Result | Known-Correct Value | Match |
|---|---|---:|---:|:---:|
| Starting position | 1 | 20 | 20 | ✅ |
| Starting position | 2 | 400 | 400 | ✅ |
| Starting position | 3 | 8,902 | 8,902 | ✅ |
| Starting position | 4 | 197,281 | 197,281 | ✅ |
| Starting position | 5 | 4,865,609 | 4,865,609 | ✅ |
| Kiwipete (castling/pin stress test) | 1–3 | 48 / 2,039 / 97,862 | same | ✅ |
| Kiwipete | 4 | 4,085,603 | 4,085,603 | ✅ |
| Kiwipete | 5 | 193,690,690 | 193,690,690 | ✅ |
| Position 3 (en passant discovered-check trap) | 1–5 | 14 / 191 / 2,812 / 43,238 / 674,624 | same | ✅ |
| Position 4 (promotion + castling-rights stress test) | 1–4 | 6 / 264 / 9,467 / 422,333 | same | ✅ |

Every test matches the published reference values exactly. These specific
positions are the standard suite used by the chess-programming community
because each one is designed to expose a particular class of bug (illegal
en passant through discovered check, castling rights not being stripped on
rook capture, promotion interactions, etc.) — passing all of them at depth
4–5 is a strong (though not formally exhaustive) correctness signal.

---

## Limitations

This is intentionally a **small, readable** engine, not a competitive one.
Known limitations, by design:

### Performance
- **No incremental make/unmake** — every move clones the entire board
  (`Board` is `#[derive(Clone)]` and `make()` returns a new copy). This is
  simple and safe but slow: Kiwipete at perft depth 5 (~194M nodes) takes
  ~33 seconds, whereas a bitboard engine with proper unmake would do this
  in well under a second.
- **No bitboards** — sliding piece attacks are computed by walking rays
  square-by-square rather than with precomputed attack tables.
- **No move ordering** — moves are searched in generation order, not
  ordered by likely strength (captures first, killer moves, etc.), so
  alpha-beta pruning is far less effective than it could be.
- **No transposition table** — repeated positions reached via different
  move orders are re-searched from scratch every time.
- **No quiescence search** — the search evaluates a static position at the
  search horizon even if a capture is "in progress," which causes the
  well-known horizon effect (the engine can misjudge tactical sequences
  right at the edge of its search depth).
- **No time management** — `go` only respects an explicit `depth N`; it
  does not parse or respect `wtime`, `btime`, `winc`, `binc`, `movetime`,
  or `nodes`. If none of these matter to your UCI harness, depth 4
  (~hundreds of thousands of nodes) is used by default.
- **`stop` is a no-op** — because search is single-threaded and
  synchronous, there is no background search to interrupt. A GUI sending
  `stop` will simply wait for the current (bounded) search to finish.

### Playing Strength
- **Material-only evaluation** — no piece-square tables, king safety,
  pawn structure, mobility, or endgame-specific evaluation. The engine
  will happily misplace pieces that a stronger evaluation would penalize.
- **Shallow default depth** — depth 4 is fast but tactically weak;
  reasonable club-level play would need iterative deepening well beyond
  what fixed-depth search here practically allows in a normal game clock.
- **No opening book, no endgame tablebases.**

### Protocol Coverage
- Implements the commands needed for a basic GUI/engine match, but does
  **not** implement the full UCI specification — notably missing:
  `setoption`, `register`, `debug`, `ponderhit`, multi-PV output (`info`
  strings during search), and `go infinite` / `go ponder` semantics.
- FEN parsing assumes well-formed input; it does not validate or reject
  malformed FEN strings.

### Code Style
- Optimized for **line count and readability**, not for micro-optimized
  Rust idioms. There is no `unsafe`, no SIMD, no manual memory layout
  tuning — this trades raw performance for a source file a person can
  read end-to-end in one sitting.

---

## Build & Run

```bash
rustc -O -o "Rusty Dragon" "Rusty Dragon.rs"
echo -e "uci\nposition startpos\ngo depth 4\nquit" | ./"Rusty Dragon"
```

Run a correctness check yourself:

```bash
echo -e "position startpos\nperft 5\nquit" | ./"Rusty Dragon"
# Expect: perft(5) = 4865609
```

---

## License

No license has been specified. Treat as "all rights reserved" by the
author, Gokul Chandar, unless you hear otherwise.
