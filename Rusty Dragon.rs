// nanochess-rs: a tiny, self-contained, dependency-free chess engine.
// Full legal move generation (castling, en passant, promotion, check/mate/stalemate,
// 50-move rule, threefold repetition) + a real UCI loop. No external crates.

use std::io::{self, BufRead, Write};

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

fn main() {
    let mut board = Board::start();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.is_empty() { continue }
        match toks[0] {
            "uci" => {
                println!("id name Rusty Dragon v0.9.9 July 2026");
                println!("id author Gokul Chandar");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => board = Board::start(),
            "position" => {
                let mut i = 1;
                if toks.get(1) == Some(&"startpos") {
                    board = Board::start();
                    i = 2;
                } else if toks.get(1) == Some(&"fen") {
                    let fen = toks[2..].iter().take_while(|&&t| t != "moves").cloned().collect::<Vec<_>>().join(" ");
                    board = Board::from_fen(&fen);
                    i = 2 + fen.split_whitespace().count();
                }
                if toks.get(i) == Some(&"moves") {
                    for mstr in &toks[i+1..] {
                        let from = sq_from_str(&mstr[0..2]);
                        let to = sq_from_str(&mstr[2..4]);
                        let promo = if mstr.len() > 4 {
                            match mstr.as_bytes()[4] { b'q'=>Q, b'r'=>R, b'b'=>B, b'n'=>N, _=>0 }
                        } else { 0 };
                        board = board.make(&Mv{from, to, promo});
                    }
                }
            }
            "go" => {
                let depth = toks.iter().position(|&t| t=="depth")
                    .and_then(|i| toks.get(i+1)).and_then(|s| s.parse().ok())
                    .unwrap_or(4);
                let (m, _) = search(&board, depth);
                match m {
                    Some(m) => println!("bestmove {}", mv_to_str(&m)),
                    None => println!("bestmove 0000"),
                }
            }
            "perft" => {
                let depth: u32 = toks.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                println!("perft({}) = {}", depth, perft(&board, depth));
            }
            "stop" => {}
            "quit" => break,
            _ => {}
        }
        io::stdout().flush().unwrap();
    }
}
