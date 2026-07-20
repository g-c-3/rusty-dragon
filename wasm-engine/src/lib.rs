// Rusty Dragon — WebAssembly build.
//
// This is the same rules engine as the native `Rusty Dragon.rs` UCI binary
// (board representation, legal move generation, negamax search), kept as a
// small, self-contained copy here so it can be compiled to wasm32 with
// wasm-bindgen and a `[cdylib]` crate type, which the native UCI binary is
// not set up for. The two files are logically identical; if you change the
// rules in one, mirror the change in the other.

use wasm_bindgen::prelude::*;

const P: i8 = 1; const N: i8 = 2; const B: i8 = 3; const R: i8 = 4; const Q: i8 = 5; const K: i8 = 6;

#[derive(Clone)]
struct Board {
    sq: [i8; 64],
    side: i8,
    castle: u8,
    ep: i8,
    half: u16,
    hist: Vec<u64>,
}

#[derive(Clone, Copy, PartialEq)]
struct Mv { from: i8, to: i8, promo: i8 }

fn file(s: i8) -> i8 { s % 8 }
fn rank(s: i8) -> i8 { s / 8 }
fn on_board(r: i8, f: i8) -> bool { (0..8).contains(&r) && (0..8).contains(&f) }

impl Board {
    fn start() -> Board { Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1") }

    fn from_fen(fen: &str) -> Board {
        let mut sq = [0i8; 64];
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let mut idx: i32 = 56;
        for c in parts[0].chars() {
            match c {
                '/' => idx -= 16,
                d if d.is_ascii_digit() => idx += d.to_digit(10).unwrap() as i32,
                c => {
                    let p = match c.to_ascii_lowercase() {
                        'p' => P, 'n' => N, 'b' => B, 'r' => R, 'q' => Q, 'k' => K, _ => 0,
                    };
                    sq[idx as usize] = if c.is_uppercase() { p } else { -p };
                    idx += 1;
                }
            }
        }
        let side = if parts.get(1) == Some(&"b") { -1 } else { 1 };
        let mut castle = 0u8;
        if let Some(cs) = parts.get(2) {
            if cs.contains('K') { castle |= 1 }
            if cs.contains('Q') { castle |= 2 }
            if cs.contains('k') { castle |= 4 }
            if cs.contains('q') { castle |= 8 }
        }
        let ep = match parts.get(3) {
            Some(s) if *s != "-" => sq_from_str(s),
            _ => -1,
        };
        let half = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
        let mut b = Board { sq, side, castle, ep, half, hist: vec![] };
        let k = b.key();
        b.hist.push(k);
        b
    }

    fn to_fen(&self) -> String {
        let mut s = String::new();
        for r in (0..8).rev() {
            let mut empty = 0;
            for f in 0..8 {
                let p = self.sq[(r*8+f) as usize];
                if p == 0 { empty += 1; continue }
                if empty > 0 { s.push_str(&empty.to_string()); empty = 0; }
                let c = piece_char(p.abs());
                s.push(if p > 0 { c.to_ascii_uppercase() } else { c });
            }
            if empty > 0 { s.push_str(&empty.to_string()); }
            if r > 0 { s.push('/'); }
        }
        s.push(' ');
        s.push(if self.side == 1 { 'w' } else { 'b' });
        s.push(' ');
        let mut cs = String::new();
        if self.castle & 1 != 0 { cs.push('K') }
        if self.castle & 2 != 0 { cs.push('Q') }
        if self.castle & 4 != 0 { cs.push('k') }
        if self.castle & 8 != 0 { cs.push('q') }
        s.push_str(if cs.is_empty() { "-" } else { &cs });
        s.push(' ');
        s.push_str(&if self.ep >= 0 { sq_to_str(self.ep) } else { "-".to_string() });
        s.push(' ');
        s.push_str(&self.half.to_string());
        s.push_str(" 1");
        s
    }

    fn key(&self) -> u64 {
        let mut h: u64 = 1469598103934665603;
        for &p in self.sq.iter() {
            h ^= p as u8 as u64;
            h = h.wrapping_mul(1099511628211);
        }
        h ^= (self.side as i64) as u64;
        h = h.wrapping_mul(1099511628211);
        h ^= self.castle as u64;
        h = h.wrapping_mul(1099511628211);
        h ^= self.ep as u8 as u64;
        h
    }

    fn attacked(&self, s: i8, by: i8) -> bool {
        let r0 = rank(s); let f0 = file(s);
        for (dr, df) in [(1,2),(2,1),(-1,2),(-2,1),(1,-2),(2,-1),(-1,-2),(-2,-1)] {
            let (r,f) = (r0+dr, f0+df);
            if on_board(r,f) && self.sq[(r*8+f) as usize] == by*N { return true }
        }
        for dr in -1..=1 { for df in -1..=1 { if dr==0 && df==0 { continue }
            let (r,f) = (r0+dr, f0+df);
            if on_board(r,f) && self.sq[(r*8+f) as usize] == by*K { return true }
        }}
        let pr = r0 - by;
        for df in [-1,1] {
            let f = f0 + df;
            if on_board(pr,f) && self.sq[(pr*8+f) as usize] == by*P { return true }
        }
        for (dr,df) in [(1,1),(1,-1),(-1,1),(-1,-1)] {
            let (mut r, mut f) = (r0+dr, f0+df);
            while on_board(r,f) {
                let p = self.sq[(r*8+f) as usize];
                if p != 0 { if p == by*B || p == by*Q { return true } break }
                r += dr; f += df;
            }
        }
        for (dr,df) in [(1,0),(-1,0),(0,1),(0,-1)] {
            let (mut r, mut f) = (r0+dr, f0+df);
            while on_board(r,f) {
                let p = self.sq[(r*8+f) as usize];
                if p != 0 { if p == by*R || p == by*Q { return true } break }
                r += dr; f += df;
            }
        }
        false
    }

    fn king_sq(&self, side: i8) -> i8 {
        self.sq.iter().position(|&p| p == side*K).unwrap() as i8
    }

    fn in_check(&self, side: i8) -> bool { self.attacked(self.king_sq(side), -side) }

    fn pseudo_moves(&self) -> Vec<Mv> {
        let mut mv = Vec::with_capacity(48);
        let s = self.side;
        for from in 0..64i8 {
            let p = self.sq[from as usize];
            if p == 0 || p.signum() != s { continue }
            let (r0, f0) = (rank(from), file(from));
            let pt = p.abs();
            match pt {
                P => {
                    let dir = s;
                    let start_rank = if s == 1 { 1 } else { 6 };
                    let promo_rank = if s == 1 { 7 } else { 0 };
                    let r1 = r0 + dir;
                    if on_board(r1, f0) && self.sq[(r1*8+f0) as usize] == 0 {
                        push_pawn(&mut mv, from, r1*8+f0, r1==promo_rank);
                        if r0 == start_rank {
                            let r2 = r0 + 2*dir;
                            if self.sq[(r2*8+f0) as usize] == 0 { mv.push(Mv{from, to:r2*8+f0, promo:0}) }
                        }
                    }
                    for df in [-1,1] {
                        let f1 = f0+df;
                        if !on_board(r1,f1) { continue }
                        let to = r1*8+f1;
                        let target = self.sq[to as usize];
                        if target != 0 && target.signum() == -s { push_pawn(&mut mv, from, to, r1==promo_rank) }
                        else if to == self.ep { mv.push(Mv{from, to, promo:0}) }
                    }
                }
                N => for (dr,df) in [(1,2),(2,1),(-1,2),(-2,1),(1,-2),(2,-1),(-1,-2),(-2,-1)] {
                    let (r,f) = (r0+dr, f0+df);
                    if on_board(r,f) {
                        let t = self.sq[(r*8+f) as usize];
                        if t.signum() != s { mv.push(Mv{from, to:r*8+f, promo:0}) }
                    }
                },
                K => {
                    for dr in -1..=1 { for df in -1..=1 { if dr==0 && df==0 { continue }
                        let (r,f) = (r0+dr, f0+df);
                        if on_board(r,f) {
                            let t = self.sq[(r*8+f) as usize];
                            if t.signum() != s { mv.push(Mv{from, to:r*8+f, promo:0}) }
                        }
                    }}
                    let (kfrom, kside_bit, qside_bit, rank0): (u8,u8,u8,i8) = if s==1 { (4,1,2,0) } else { (60,4,8,7) };
                    if from as u8 == kfrom && !self.in_check(s) {
                        if self.castle & kside_bit != 0
                            && self.sq[(rank0*8+5) as usize]==0 && self.sq[(rank0*8+6) as usize]==0
                            && !self.attacked(rank0*8+5, -s) && !self.attacked(rank0*8+6, -s) {
                            mv.push(Mv{from, to: rank0*8+6, promo:0});
                        }
                        if self.castle & qside_bit != 0
                            && self.sq[(rank0*8+1) as usize]==0 && self.sq[(rank0*8+2) as usize]==0 && self.sq[(rank0*8+3) as usize]==0
                            && !self.attacked(rank0*8+3, -s) && !self.attacked(rank0*8+2, -s) {
                            mv.push(Mv{from, to: rank0*8+2, promo:0});
                        }
                    }
                }
                _ => {
                    let dirs: &[(i8,i8)] = match pt {
                        B => &[(1,1),(1,-1),(-1,1),(-1,-1)],
                        R => &[(1,0),(-1,0),(0,1),(0,-1)],
                        _ => &[(1,1),(1,-1),(-1,1),(-1,-1),(1,0),(-1,0),(0,1),(0,-1)],
                    };
                    for &(dr,df) in dirs {
                        let (mut r, mut f) = (r0+dr, f0+df);
                        while on_board(r,f) {
                            let t = self.sq[(r*8+f) as usize];
                            if t == 0 { mv.push(Mv{from, to:r*8+f, promo:0}) }
                            else { if t.signum() != s { mv.push(Mv{from, to:r*8+f, promo:0}) } break }
                            r += dr; f += df;
                        }
                    }
                }
            }
        }
        mv
    }

    fn make(&self, m: &Mv) -> Board {
        let mut b = self.clone();
        let s = self.side;
        let piece = b.sq[m.from as usize];
        let pt = piece.abs();
        let capture = b.sq[m.to as usize] != 0;

        if pt == P && m.to == self.ep && file(m.to) != file(m.from) {
            b.sq[(rank(m.from)*8 + file(m.to)) as usize] = 0;
        }
        if pt == K && (m.to - m.from).abs() == 2 {
            let rank0 = rank(m.from);
            if m.to > m.from {
                b.sq[(rank0*8+5) as usize] = b.sq[(rank0*8+7) as usize];
                b.sq[(rank0*8+7) as usize] = 0;
            } else {
                b.sq[(rank0*8+3) as usize] = b.sq[(rank0*8+0) as usize];
                b.sq[(rank0*8+0) as usize] = 0;
            }
        }

        b.sq[m.to as usize] = if m.promo != 0 { m.promo * s } else { piece };
        b.sq[m.from as usize] = 0;

        for &(sqidx, bit) in &[(0u8,2u8),(7u8,1u8),(56u8,8u8),(63u8,4u8)] {
            if m.from as u8 == sqidx || m.to as u8 == sqidx { b.castle &= !bit; }
        }
        if pt == K { b.castle &= if s==1 {!3u8} else {!12u8}; }

        b.ep = if pt == P && (m.to - m.from).abs() == 16 { (m.from + m.to)/2 } else { -1 };
        b.half = if pt == P || capture { 0 } else { self.half + 1 };
        b.side = -s;
        let k = b.key();
        b.hist.push(k);
        b
    }

    fn legal_moves(&self) -> Vec<Mv> {
        let s = self.side;
        self.pseudo_moves().into_iter().filter(|m| {
            let nb = self.make(m);
            !nb.in_check(s)
        }).collect()
    }

    fn is_repetition(&self) -> bool {
        let k = *self.hist.last().unwrap();
        self.hist.iter().filter(|&&x| x == k).count() >= 3
    }

    fn eval(&self) -> i32 {
        let val = |p: i8| match p.abs() { P=>100, N=>320, B=>330, R=>500, Q=>900, K=>0, _=>0 };
        let mut score = 0;
        for &p in self.sq.iter() { if p != 0 { score += val(p) * p.signum() as i32 } }
        score * self.side as i32
    }
}

fn piece_char(p: i8) -> char {
    match p { 1=>'p', 2=>'n', 3=>'b', 4=>'r', 5=>'q', 6=>'k', _=>'?' }
}

fn push_pawn(mv: &mut Vec<Mv>, from: i8, to: i8, promote: bool) {
    if promote { for pc in [Q,R,B,N] { mv.push(Mv{from, to, promo:pc}) } }
    else { mv.push(Mv{from, to, promo:0}) }
}

fn sq_from_str(s: &str) -> i8 {
    let b = s.as_bytes();
    ((b[1]-b'1') as i8)*8 + (b[0]-b'a') as i8
}
fn sq_to_str(s: i8) -> String {
    format!("{}{}", (b'a'+file(s) as u8) as char, (b'1'+rank(s) as u8) as char)
}
fn mv_to_str(m: &Mv) -> String {
    let promo = match m.promo { Q=>"q", R=>"r", B=>"b", N=>"n", _=>"" };
    format!("{}{}{}", sq_to_str(m.from), sq_to_str(m.to), promo)
}

fn parse_uci(s: &str) -> Option<Mv> {
    if s.len() < 4 { return None }
    let bytes = s.as_bytes();
    if bytes.len() < 4 { return None }
    let from = sq_from_str(&s[0..2]);
    let to = sq_from_str(&s[2..4]);
    let promo = if s.len() > 4 {
        match bytes[4] { b'q'=>Q, b'r'=>R, b'b'=>B, b'n'=>N, _=>0 }
    } else { 0 };
    Some(Mv{from, to, promo})
}

fn negamax(b: &Board, depth: i32, mut alpha: i32, beta: i32) -> i32 {
    if b.half >= 100 || b.is_repetition() { return 0 }
    let moves = b.legal_moves();
    if moves.is_empty() {
        return if b.in_check(b.side) { -30000 + (5-depth).max(0) } else { 0 };
    }
    if depth == 0 { return b.eval() }
    let mut best = i32::MIN + 1;
    for m in moves {
        let nb = b.make(&m);
        let score = -negamax(&nb, depth-1, -beta, -alpha);
        if score > best { best = score }
        if best > alpha { alpha = best }
        if alpha >= beta { break }
    }
    best
}

fn search(b: &Board, depth: i32) -> (Option<Mv>, i32) {
    let moves = b.legal_moves();
    if moves.is_empty() { return (None, if b.in_check(b.side) { -30000 } else { 0 }) }
    let (mut best_m, mut best_s) = (moves[0], i32::MIN+1);
    let (mut alpha, beta) = (i32::MIN+1, i32::MAX-1);
    for m in moves {
        let nb = b.make(&m);
        let score = -negamax(&nb, depth-1, -beta, -alpha);
        if score > best_s { best_s = score; best_m = m; }
        if best_s > alpha { alpha = best_s }
    }
    (Some(best_m), best_s)
}

fn perft(b: &Board, depth: u32) -> u64 {
    if depth == 0 { return 1 }
    let moves = b.legal_moves();
    if depth == 1 { return moves.len() as u64 }
    moves.iter().map(|m| perft(&b.make(m), depth-1)).sum()
}

// ---------------------------------------------------------------------
// wasm-bindgen surface — this is what index.html's JS module calls.
// ---------------------------------------------------------------------

#[wasm_bindgen]
pub struct Engine {
    board: Board,
}

#[wasm_bindgen]
impl Engine {
    /// Create a new engine at the standard starting position.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine {
        Engine { board: Board::start() }
    }

    /// Reset to the standard starting position.
    pub fn reset(&mut self) {
        self.board = Board::start();
    }

    /// Load a position from FEN.
    pub fn set_fen(&mut self, fen: &str) {
        self.board = Board::from_fen(fen);
    }

    /// Current position as FEN.
    pub fn fen(&self) -> String {
        self.board.to_fen()
    }

    /// 1 if White to move, -1 if Black to move.
    pub fn side_to_move(&self) -> i32 {
        self.board.side as i32
    }

    /// Whether the side to move is currently in check.
    pub fn is_check(&self) -> bool {
        self.board.in_check(self.board.side)
    }

    /// Whether the current position is a draw by the 50-move rule.
    pub fn is_fifty_move_draw(&self) -> bool {
        self.board.half >= 100
    }

    /// Whether the current position has occurred three times.
    pub fn is_repetition(&self) -> bool {
        self.board.is_repetition()
    }

    /// All legal moves from the current position, as a space-separated
    /// string of UCI move strings (e.g. "e2e4 g1f3 e7e8q").
    pub fn legal_moves(&self) -> String {
        self.board.legal_moves().iter().map(mv_to_str).collect::<Vec<_>>().join(" ")
    }

    /// Apply a move given in UCI notation (e.g. "e2e4", "e7e8q").
    /// Returns true if the move was legal and was applied.
    pub fn make_move(&mut self, uci: &str) -> bool {
        let mv = match parse_uci(uci) { Some(m) => m, None => return false };
        let legal = self.board.legal_moves();
        if !legal.contains(&mv) { return false }
        self.board = self.board.make(&mv);
        true
    }

    /// Search to `depth` plies and return the best move in UCI notation,
    /// or an empty string if there are no legal moves.
    pub fn best_move(&self, depth: i32) -> String {
        let (m, _) = search(&self.board, depth.max(1));
        match m { Some(m) => mv_to_str(&m), None => String::new() }
    }

    /// perft(depth) node count, returned as a string (perft(6)+ exceeds
    /// what JS can represent exactly as a f64-backed Number).
    pub fn perft(&self, depth: u32) -> String {
        perft(&self.board, depth).to_string()
    }
}

impl Default for Engine {
    fn default() -> Self { Engine::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_startpos() {
        let b = Board::start();
        assert_eq!(perft(&b, 1), 20);
        assert_eq!(perft(&b, 2), 400);
        assert_eq!(perft(&b, 3), 8902);
        assert_eq!(perft(&b, 4), 197281);
    }

    #[test]
    fn perft_kiwipete() {
        let b = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&b, 1), 48);
        assert_eq!(perft(&b, 2), 2039);
        assert_eq!(perft(&b, 3), 97862);
    }

    #[test]
    fn perft_position3_en_passant_pin() {
        let b = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&b, 1), 14);
        assert_eq!(perft(&b, 2), 191);
        assert_eq!(perft(&b, 3), 2812);
        assert_eq!(perft(&b, 4), 43238);
    }

    #[test]
    fn perft_position4_promotion_castling() {
        let b = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(perft(&b, 1), 6);
        assert_eq!(perft(&b, 2), 264);
        assert_eq!(perft(&b, 3), 9467);
        assert_eq!(perft(&b, 4), 422333);
    }

    #[test]
    fn fen_round_trip() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        assert_eq!(Board::from_fen(fen).to_fen(), fen);
    }

    #[test]
    fn en_passant_square_recorded_in_fen() {
        let mut eng = Engine::new();
        assert!(eng.make_move("e2e4"));
        assert!(eng.make_move("e7e5"));
        assert!(eng.fen().contains(" e6 "));
    }

    #[test]
    fn engine_rejects_illegal_move() {
        let mut eng = Engine::new();
        assert!(!eng.make_move("e2e5")); // pawns can't jump three squares
        assert!(eng.make_move("e2e4"));
        assert!(!eng.make_move("e2e4")); // already moved, no longer legal from e2
    }

    #[test]
    fn checkmate_detected_via_engine_api() {
        let mut eng = Engine::new();
        eng.set_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3");
        assert!(eng.is_check());
        assert!(eng.legal_moves().is_empty());
    }

    #[test]
    fn best_move_returns_a_legal_move() {
        let eng = Engine::new();
        let bm = eng.best_move(3);
        assert!(eng.legal_moves().split(' ').any(|m| m == bm));
    }
}
