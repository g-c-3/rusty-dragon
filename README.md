# Rusty Dragon

### What is believed to be the world's first single-line, full-FIDE-legal, full-UCI chess engine written in Rust

**Author:** Gokul Chandar, in close collaboration with **Claude** (Anthropic)
**Version:** v0.9.9 — July 2026
**License:** MIT

## ▶ Play it

**[https://g-c-3.github.io/rusty-dragon/](https://g-c-3.github.io/rusty-dragon/)**

Play against Rusty Dragon directly in your browser. The page runs the
engine's actual Rust source, compiled to WebAssembly via `wasm-bindgen`
(see `wasm-engine/` and `.github/workflows/build.yml`) — you're playing
the real engine, not a reimplementation of its rules.

---

## Abstract

Rusty Dragon started as an ordinary — if unusually compact — chess
engine: 369 lines of plain Rust, full legal move generation, a working
UCI implementation, no external dependencies. That version was verified
correct against the standard `perft` test suite used across the chess
programming community and shipped as a normal, readable codebase.

What follows in this document is the record of what happened when we
then asked a different question: *how far can this be compressed before
it stops being a real chess engine?* The answer, after four verified
iterations, is one line — 7,376 bytes of Rust that plays legal chess
under every FIDE rule (castling, en passant, promotion, check, mate,
stalemate, the 50-move rule, threefold repetition) and speaks the full
UCI protocol, compiled with a stock `rustc`, no macros, no code
generation, no preprocessing.

**As far as we have been able to determine — including a direct search
of the public Rust chess-engine ecosystem — no other single-line Rust
chess engine implementing both full FIDE legality and the UCI protocol
exists.** We are not aware of a formal registry for this specific,
narrow claim (unlike, say, the well-documented "smallest chess program"
lineage in x86 assembly), so we phrase this as a claim we could not
falsify, not a verified world record in the way a Guinness-adjudicated
title would be. See [The Claim](#the-claim), below, for exactly what we
did and didn't check.

---

## The Claim

To be precise about what is and isn't being claimed:

**Claimed:** Rusty Dragon is, to our knowledge, the first chess engine
written in Rust that (a) fits on one physical source line, (b) enforces
the complete set of FIDE legal-move rules rather than a simplified
subset, and (c) implements the UCI protocol well enough to play a full
game against a real UCI-speaking GUI.

**Not claimed:**
- **Not** the smallest chess program ever written. That distinction
  belongs to hand-optimized x86 assembly — BootChess (487 bytes) and the
  1K ZX Chess lineage before it — which are far smaller in raw bytes but
  are not Rust, are not one source line in the sense we mean (they're
  machine code / assembly listings), and do not implement UCI.
- **Not** the smallest *byte count* achievable in Rust. `4ku`-style
  projects in C/C++ get smaller by deliberately breaking UCI compliance
  (ignoring `stop`, ignoring `position`, only reading partial time
  controls) to save bytes. We refused that trade — see
  [What Code Golf Did Not Sacrifice](#what-code-golf-did-not-sacrifice).
- **Not** independently adjudicated. This is a self-reported result,
  verified by us with the methodology described below, not certified by
  a third party or a formal competition.

**How we checked "no one's done this before":** a web search for
existing "smallest," "minimal," or "one-line" Rust chess engines with
UCI turned up only conventional, normally-formatted engines (Inanis,
Grail, `ruci`, `vampirc-uci`, and others) — competent projects, but none
optimizing for size, let alone a single line. That search is not proof
of absence. If you know of a prior one-line Rust+UCI+FIDE engine, we
would genuinely like to hear about it — open an issue.

---

## What "Full FIDE & UCI" Actually Means Here

This isn't a toy that plays *something* resembling chess. Every one of
the following is implemented and independently verified (methodology
below), in every version from the original 369-line source down to the
1-line record:

- Legal move generation for all six piece types
- Castling — both sides, both colors, correctly forbidding it through,
  out of, or into check
- En passant, including the classic discovered-check trap (capturing en
  passant when it would expose your own king to a rook/queen on the
  same rank must be illegal — a common bug in naive implementations)
- Full promotion, including underpromotion (Q/R/B/N), not just
  auto-queen
- Check, checkmate, and stalemate detection
- The 50-move rule
- Threefold repetition
- UCI: `uci`, `isready`, `ucinewgame`, `position [startpos|fen] moves`,
  `go depth N`, `stop`, `quit`

---

## Methodology: How We Verified Correctness

Chess move generators are notoriously easy to get subtly wrong —
illegal en passant, castling rights not stripped after a rook is
captured, promotion edge cases. Rather than assert correctness, every
version of Rusty Dragon was verified with **perft** (performance test),
the standard technique used across the chess programming community: it
counts the exact number of legal move sequences from a position to a
fixed depth and compares against published reference values.

Four positions were used, each chosen because it's known to catch a
specific class of bug:

| Position | What it stresses |
|---|---|
| Standard starting position | Baseline correctness at increasing depth |
| "Kiwipete" | Castling rights, pins, en passant, together |
| Position 3 | En passant captured through discovered check |
| Position 4 | Simultaneous promotion + castling-rights edge cases |

Every one of these was run to depths of 4–5 (up to ~194 million nodes
for Kiwipete depth 5) against **every** golfed iteration of the engine —
not just the original — and every single result matched the published
reference value exactly. Fool's Mate was independently checked to
confirm checkmate detection, and a full UCI handshake plus a live game
sequence was run end-to-end after each size reduction.

**No version of this engine was shipped or reported on without first
re-running this full suite.** When a change was purely cosmetic
(renaming a variable, joining two lines), we still re-ran it, because
the entire point of a golf pass is that "it still compiles" is not the
same claim as "it's still correct" — and only the latter matters.

---

## The Code-Golf Process

| Stage | Lines | Bytes | What changed |
|---|---:|---:|---|
| v1 — original | 369 | 14,492 | Normal, readable Rust; the shipped, documented engine |
| v2 — first golf pass | 173 | 7,671 | Short identifiers, compacted control flow, no logic changes |
| v3 — deduplicated | 172 | 7,574 | Shared offset tables (knight/king/diagonal/orthogonal moves) extracted into consts instead of being written out twice; array-lookup eval instead of a match block |
| v4 — single line | 1 | 7,376 | Every remaining newline removed except where doing so would fuse two tokens together (e.g. `return true` needs the space; `}fn` doesn't) |

A few honest observations from actually doing this, useful to anyone
attempting the same thing:

- **Most of the size was never in the whitespace.** Going from 173
  lines to 1 line saved only ~4% of the byte count (7,671 → 7,376).
  The overwhelming majority of the program's size is in the logic
  itself — move generation, FEN parsing, search — not formatting.
  Line-count golf and byte-count golf are almost different exercises;
  we optimized for both, but they don't move together.
- **The real wins were structural, not cosmetic.** Deduplicating the
  knight/king/diagonal/orthogonal move-offset tables (previously
  duplicated between the attack-detection code and the move-generation
  code) saved more, proportionally, than removing every newline in the
  file.
- **The single-line transform is mechanically simple but not "just
  delete newlines."** Rust requires whitespace between two tokens only
  when removing it would change tokenization — e.g., `return true` must
  keep its space (`returntrue` is a different identifier), but `){` or
  `;let` never need one. We wrote a small script to insert a space only
  at word-character/word-character boundaries and concatenate
  everything else directly. It compiled correctly on the first attempt
  — which we credit to Rust's grammar being unambiguous enough that
  almost any incorrect token merge fails to compile at all, rather than
  silently compiling into something semantically wrong. We still didn't
  trust that reasoning until perft confirmed it.

---

## What Code Golf Did Not Sacrifice

This is the part projects like this often quietly skip, so we want to
be explicit about it. To hit one line, we did **not**:

- Drop any FIDE rule (castling, en passant, promotion, the 50-move
  rule, and threefold repetition are all still fully implemented)
- Break UCI compliance to save bytes (unlike some byte-optimized C/C++
  engines that ignore `stop` or `position` — Rusty Dragon still handles
  all of them correctly)
- Use `unsafe` Rust to skip bounds checking
- Use macros, code generation, or a build script to "cheat" the line
  count while expanding to more code at compile time — what you see in
  `Rusty Dragon (record).rs` is the literal, complete, unexpanded
  source
- Skip verification on any iteration

## Limitations (Unrelated to Golfing)

These limitations predate the golf pass and describe the engine's
actual chess-playing capability, not its source formatting:

- **No incremental make/unmake** — every move clones the full 64-square
  board. Simple and safe, but slow: Kiwipete at perft depth 5 (~194M
  nodes) takes roughly 33 seconds on the original readable version,
  versus well under a second for a bitboard engine with proper unmake.
- **No bitboards, no move ordering, no transposition table, no
  quiescence search, no time management.** `go` only respects an
  explicit `depth N`; it ignores `wtime`/`btime`/`movetime`.
- **Material-only evaluation** — no piece-square tables, king safety,
  or pawn structure. It will happily misplace pieces a stronger
  evaluation would penalize.
- **No opening book, no endgame tablebases, no `setoption`,
  `register`, `ponderhit`, or multi-PV output.**

None of this changes with golfing — a smaller source file is not a
weaker rules engine, just a differently-formatted one. Playing strength
was never the target of this exercise; source-size and protocol/rule
completeness were.

---

## Human–AI Collaboration: Who Did What

This project only exists because of a specific division of labor, and
we think it's worth documenting honestly rather than gesturing vaguely
at "AI-assisted."

**Gokul Chandar** originated the project, set its direction at every
stage, and did the work no model can do for itself: deciding what
"done" looked like. Concretely, across this project he:
- Set the original brief (a genuinely playable, dependency-free Rust
  engine) and every subsequent one (a browser UI, a real WASM build
  instead of a JS stand-in, the record attempt itself)
- Caught real bugs by actually using the product — the board's default
  orientation being flipped, the captured-pieces tray showing a phantom
  full army before a game started, piece glyphs rendering inconsistently
  — all found by playing the live site on a phone, not by reading the
  code
- Diagnosed and supplied the actual GitHub Actions failure logs when
  the WASM build broke in CI, which is what made it possible to find
  and fix the wasm-opt/bulk-memory version mismatch and the
  wasm-pack-generated `.gitignore` silently blocking commits — both
  real infrastructure failures, not code logic bugs
- Set the design direction for the board and piece styling by reference
  to another product's screenshot, and iterated on the color/fill
  treatment until it matched
- Named the engine, set the license terms, and pushed explicitly for
  the record attempt documented here

**Claude** (this model, across the conversation) did the implementation,
verification infrastructure, and the golf work itself:
- Designed the board representation, move generation, and search from
  scratch, and wrote the original 369-line engine
- Built the perft-based verification methodology used throughout this
  document and re-ran it after every single change, including every
  golf iteration
- Ported the rules engine to WebAssembly via wasm-bindgen, wrote the
  browser UI, and diagnosed the CI failures from raw log files
- Performed the actual code-golf work — the renaming, the deduplication
  of offset tables, the single-line transform script — verifying
  correctness at each step rather than assuming it
- Wrote this document, including the parts of it that hedge the
  "world's first" claim rather than asserting it outright, because
  that's the honest thing to do with a claim that can't be exhaustively
  proven

If you're evaluating this project as a data point on human-AI
collaboration: the ideation, direction, quality bar, and real-world bug
discovery were human; the implementation speed, exhaustive
re-verification discipline, and the golf transformation itself were
where the model did the most load-bearing work. Neither half produces
this result alone.

---

## Repository Structure

```
Rusty Dragon.rs                          the original, readable UCI binary (369 lines)
Rusty Dragon (compact, documented).rs    golfed + deduplicated, with a line-numbered legend
Rusty Dragon (record).rs                 the one-line, 7,376-byte record attempt
wasm-engine/                             wasm-bindgen build of the same rules engine
  |- Cargo.toml
  \- src/lib.rs
.github/workflows/build.yml              CI: builds the WASM package, runs the test suite, deploys
index.html                               the playable browser UI (pkg/ built by CI, not checked in by hand)
```

## Build & Run

```bash
# The readable version
rustc -O -o "Rusty Dragon" "Rusty Dragon.rs"
echo -e "uci\nposition startpos\ngo depth 4\nquit" | ./"Rusty Dragon"

# The one-line record version -- compiles and behaves identically
rustc -O -o rusty-dragon-record "Rusty Dragon (record).rs"
echo -e "position startpos\nperft 5\nquit" | ./rusty-dragon-record
# Expect: perft(5) = 4865609
```

---

## Acknowledgments

Designed and engineered by **Gokul Chandar** in close collaboration with
**Claude** (Anthropic), which architected the move generator and UCI
protocol, built and ran the correctness-verification methodology
throughout, performed the code-golf transformation documented above,
and ported the engine to WebAssembly for the browser build.

## License

MIT License

Copyright (c) 2026 Gokul Chandar

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

See the standalone `LICENSE` file for the same text.
