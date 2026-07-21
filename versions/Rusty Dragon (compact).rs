use std::io::{self,BufRead,Write};
const P:i8=1;const N:i8=2;const B:i8=3;const R:i8=4;const Q:i8=5;const K:i8=6;
const KN:[(i8,i8);8]=[(1,2),(2,1),(-1,2),(-2,1),(1,-2),(2,-1),(-1,-2),(-2,-1)];const KG:[(i8,i8);8]=[(1,1),(1,0),(1,-1),(0,1),(0,-1),(-1,1),(-1,0),(-1,-1)];
const DG:[(i8,i8);4]=[(1,1),(1,-1),(-1,1),(-1,-1)];const OR:[(i8,i8);4]=[(1,0),(-1,0),(0,1),(0,-1)];const VA:[i32;7]=[0,100,320,330,500,900,0];
fn fl(s:i8)->i8{s%8}
fn rk(s:i8)->i8{s/8}
fn ob(r:i8,f:i8)->bool{(0..8).contains(&r)&&(0..8).contains(&f)}
#[derive(Clone)]
struct S{q:[i8;64],s:i8,c:u8,e:i8,h:u16,t:Vec<u64>}
#[derive(Clone,Copy,PartialEq)]
struct M{f:i8,t:i8,p:i8}
impl S{
fn su()->S{S::fe("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")}
fn fe(fen:&str)->S{
let mut q=[0i8;64];let pt:Vec<&str>=fen.split_whitespace().collect();let mut i:i32=56;
for c in pt[0].chars(){match c{
'/'=>i-=16,
d if d.is_ascii_digit()=>i+=d.to_digit(10).unwrap()as i32,
c=>{let p=match c.to_ascii_lowercase(){'p'=>P,'n'=>N,'b'=>B,'r'=>R,'q'=>Q,'k'=>K,_=>0};q[i as usize]=if c.is_uppercase(){p}else{-p};i+=1;}
}}
let s=if pt.get(1)==Some(&"b"){-1}else{1};
let mut c=0u8;
if let Some(cs)=pt.get(2){if cs.contains('K'){c|=1}if cs.contains('Q'){c|=2}if cs.contains('k'){c|=4}if cs.contains('q'){c|=8}}
let e=match pt.get(3){Some(s)if *s!="-"=>sf(s),_=>-1};
let h=pt.get(4).and_then(|s|s.parse().ok()).unwrap_or(0);
let mut b=S{q,s,c,e,h,t:vec![]};let k=b.ky();b.t.push(k);b
}
fn ky(&self)->u64{
let mut h:u64=1469598103934665603;
for &p in self.q.iter(){h^=p as u8 as u64;h=h.wrapping_mul(1099511628211);}
h^=(self.s as i64)as u64;h=h.wrapping_mul(1099511628211);
h^=self.c as u64;h=h.wrapping_mul(1099511628211);
h^=self.e as u8 as u64;h
}
fn at(&self,s:i8,by:i8)->bool{
let r0=rk(s);let f0=fl(s);
for(dr,df)in KN{let(r,f)=(r0+dr,f0+df);if ob(r,f)&&self.q[(r*8+f)as usize]==by*N{return true}}
for(dr,df)in KG{let(r,f)=(r0+dr,f0+df);if ob(r,f)&&self.q[(r*8+f)as usize]==by*K{return true}}
let pr=r0-by;
for df in[-1,1]{let f=f0+df;if ob(pr,f)&&self.q[(pr*8+f)as usize]==by*P{return true}}
for(dr,df)in DG{
let(mut r,mut f)=(r0+dr,f0+df);
while ob(r,f){let p=self.q[(r*8+f)as usize];if p!=0{if p==by*B||p==by*Q{return true}break}r+=dr;f+=df;}}
for(dr,df)in OR{
let(mut r,mut f)=(r0+dr,f0+df);
while ob(r,f){let p=self.q[(r*8+f)as usize];if p!=0{if p==by*R||p==by*Q{return true}break}r+=dr;f+=df;}}
false
}
fn kq(&self,sd:i8)->i8{self.q.iter().position(|&p|p==sd*K).unwrap()as i8}
fn ic(&self,sd:i8)->bool{self.at(self.kq(sd),-sd)}
fn pm(&self)->Vec<M>{
let mut v=vec![];let s=self.s;
for from in 0..64i8{
let p=self.q[from as usize];if p==0||p.signum()!=s{continue}
let(r0,f0)=(rk(from),fl(from));let pt=p.abs();
match pt{
P=>{
let dir=s;let sr=if s==1{1}else{6};let prm=if s==1{7}else{0};let r1=r0+dir;
if ob(r1,f0)&&self.q[(r1*8+f0)as usize]==0{
pp(&mut v,from,r1*8+f0,r1==prm);
if r0==sr{let r2=r0+2*dir;if self.q[(r2*8+f0)as usize]==0{v.push(M{f:from,t:r2*8+f0,p:0})}}}
for df in[-1,1]{let f1=f0+df;if!ob(r1,f1){continue}
let to=r1*8+f1;let tg=self.q[to as usize];
if tg!=0&&tg.signum()==-s{pp(&mut v,from,to,r1==prm)}
else if to==self.e{v.push(M{f:from,t:to,p:0})}}
}
N=>for(dr,df)in KN{
let(r,f)=(r0+dr,f0+df);if ob(r,f){let t=self.q[(r*8+f)as usize];if t.signum()!=s{v.push(M{f:from,t:r*8+f,p:0})}}},
K=>{
for(dr,df)in KG{
let(r,f)=(r0+dr,f0+df);if ob(r,f){let t=self.q[(r*8+f)as usize];if t.signum()!=s{v.push(M{f:from,t:r*8+f,p:0})}}}
let(kf,kb,qb,r0_):(u8,u8,u8,i8)=if s==1{(4,1,2,0)}else{(60,4,8,7)};
if from as u8==kf&&!self.ic(s){
if self.c&kb!=0&&self.q[(r0_*8+5)as usize]==0&&self.q[(r0_*8+6)as usize]==0&&!self.at(r0_*8+5,-s)&&!self.at(r0_*8+6,-s){v.push(M{f:from,t:r0_*8+6,p:0});}
if self.c&qb!=0&&self.q[(r0_*8+1)as usize]==0&&self.q[(r0_*8+2)as usize]==0&&self.q[(r0_*8+3)as usize]==0&&!self.at(r0_*8+3,-s)&&!self.at(r0_*8+2,-s){v.push(M{f:from,t:r0_*8+2,p:0});}
}
}
_=>{
let dirs:&[(i8,i8)]=match pt{B=>&DG,R=>&OR,_=>&[(1,1),(1,-1),(-1,1),(-1,-1),(1,0),(-1,0),(0,1),(0,-1)]};
for &(dr,df)in dirs{
let(mut r,mut f)=(r0+dr,f0+df);
while ob(r,f){let t=self.q[(r*8+f)as usize];
if t==0{v.push(M{f:from,t:r*8+f,p:0})}else{if t.signum()!=s{v.push(M{f:from,t:r*8+f,p:0})}break}
r+=dr;f+=df;}}
}
}}
v
}
fn mk(&self,m:&M)->S{
let mut b=self.clone();let s=self.s;let pc=b.q[m.f as usize];let pt=pc.abs();let cap=b.q[m.t as usize]!=0;
if pt==P&&m.t==self.e&&fl(m.t)!=fl(m.f){b.q[(rk(m.f)*8+fl(m.t))as usize]=0;}
if pt==K&&(m.t-m.f).abs()==2{
let r0=rk(m.f);
if m.t>m.f{b.q[(r0*8+5)as usize]=b.q[(r0*8+7)as usize];b.q[(r0*8+7)as usize]=0;}
else{b.q[(r0*8+3)as usize]=b.q[(r0*8+0)as usize];b.q[(r0*8+0)as usize]=0;}
}
b.q[m.t as usize]=if m.p!=0{m.p*s}else{pc};b.q[m.f as usize]=0;
for &(si,bit)in &[(0u8,2u8),(7u8,1u8),(56u8,8u8),(63u8,4u8)]{if m.f as u8==si||m.t as u8==si{b.c&=!bit;}}
if pt==K{b.c&=if s==1{!3u8}else{!12u8};}
b.e=if pt==P&&(m.t-m.f).abs()==16{(m.f+m.t)/2}else{-1};
b.h=if pt==P||cap{0}else{self.h+1};
b.s=-s;let k=b.ky();b.t.push(k);b
}
fn lm(&self)->Vec<M>{let s=self.s;self.pm().into_iter().filter(|m|{let nb=self.mk(m);!nb.ic(s)}).collect()}
fn ir(&self)->bool{let k=*self.t.last().unwrap();self.t.iter().filter(|&&x|x==k).count()>=3}
fn ev(&self)->i32{
let mut s=0;for &p in self.q.iter(){if p!=0{s+=VA[p.abs()as usize]*p.signum()as i32}}s*self.s as i32
}
}
fn pp(v:&mut Vec<M>,f:i8,t:i8,pr:bool){if pr{for pc in[Q,R,B,N]{v.push(M{f,t,p:pc})}}else{v.push(M{f,t,p:0})}}
fn sf(s:&str)->i8{let b=s.as_bytes();((b[1]-b'1')as i8)*8+(b[0]-b'a')as i8}
fn ss(s:i8)->String{format!("{}{}",(b'a'+fl(s)as u8)as char,(b'1'+rk(s)as u8)as char)}
fn ms(m:&M)->String{let p=match m.p{Q=>"q",R=>"r",B=>"b",N=>"n",_=>""};format!("{}{}{}",ss(m.f),ss(m.t),p)}
fn nm(b:&S,d:i32,mut a:i32,be:i32)->i32{
if b.h>=100||b.ir(){return 0}
let mv=b.lm();
if mv.is_empty(){return if b.ic(b.s){-30000+(5-d).max(0)}else{0}}
if d==0{return b.ev()}
let mut best=i32::MIN+1;
for m in mv{let nb=b.mk(&m);let sc=-nm(&nb,d-1,-be,-a);if sc>best{best=sc}if best>a{a=best}if a>=be{break}}
best
}
fn se(b:&S,d:i32)->(Option<M>,i32){
let mv=b.lm();
if mv.is_empty(){return(None,if b.ic(b.s){-30000}else{0})}
let(mut bm,mut bs)=(mv[0],i32::MIN+1);let(mut a,be)=(i32::MIN+1,i32::MAX-1);
for m in mv{let nb=b.mk(&m);let sc=-nm(&nb,d-1,-be,-a);if sc>bs{bs=sc;bm=m;}if bs>a{a=bs}}
(Some(bm),bs)
}
fn pf(b:&S,d:u32)->u64{
if d==0{return 1}
let mv=b.lm();
if d==1{return mv.len()as u64}
mv.iter().map(|m|pf(&b.mk(m),d-1)).sum()
}
fn main(){
let mut b=S::su();
for line in io::stdin().lock().lines(){
let line=line.unwrap();let tk:Vec<&str>=line.split_whitespace().collect();
if tk.is_empty(){continue}
match tk[0]{
"uci"=>print!("id name Rusty Dragon v0.9.9 July 2026\nid author Gokul Chandar\nuciok\n"),
"isready"=>println!("readyok"),
"ucinewgame"=>b=S::su(),
"position"=>{
let mut i=1;
if tk.get(1)==Some(&"startpos"){b=S::su();i=2;}
else if tk.get(1)==Some(&"fen"){
let fen=tk[2..].iter().take_while(|&&t|t!="moves").cloned().collect::<Vec<_>>().join(" ");
b=S::fe(&fen);i=2+fen.split_whitespace().count();
}
if tk.get(i)==Some(&"moves"){
for ms_ in &tk[i+1..]{
let f=sf(&ms_[0..2]);let t=sf(&ms_[2..4]);
let p=if ms_.len()>4{match ms_.as_bytes()[4]{b'q'=>Q,b'r'=>R,b'b'=>B,b'n'=>N,_=>0}}else{0};
b=b.mk(&M{f,t,p});
}
}
}
"go"=>{
let d=tk.iter().position(|&t|t=="depth").and_then(|i|tk.get(i+1)).and_then(|s|s.parse().ok()).unwrap_or(4);
let(m,_)=se(&b,d);
match m{Some(m)=>println!("bestmove {}",ms(&m)),None=>println!("bestmove 0000")}
}
"perft"=>{let d:u32=tk.get(1).and_then(|s|s.parse().ok()).unwrap_or(1);println!("perft({}) = {}",d,pf(&b,d));}
"stop"=>{}
"quit"=>break,
_=>{}
}
io::stdout().flush().unwrap();
}
}

// ---------------------------------------------------------------------
// Legend (kept out of the code body so nothing above this line is a
// comment — every line above is executable). Line numbers refer to
// this file as written.
// ---------------------------------------------------------------------
// L1      imports (io traits for stdin/stdout)
// L2      piece-type constants: P N B R Q K
// L3      KN = knight move offsets, KG = king move offsets (8 each)
// L4      DG = bishop/diagonal offsets, OR = rook/orthogonal offsets,
//         VA = material values indexed by piece type
// L5      fl(sq)  -> file 0-7 of a square index 0-63
// L6      rk(sq)  -> rank 0-7 of a square index 0-63
// L7      ob(r,f) -> is (rank,file) on the board
// L8-9    struct S = position: q=64 squares, s=side to move (+1/-1),
//         c=castling rights bitmask, e=en-passant square, h=halfmove
//         clock, t=history of position hashes (repetition detection)
// L10-11  struct M = a move: f=from, t=to, p=promotion piece (0=none)
// L12-13  su()  -> starting position (via fe() on the standard FEN)
// L14-27  fe(fen) -> parse a FEN string into a position S
// L28-34  ky()  -> 64-bit hash of the position, used for repetition
// L35-48  at(sq,by) -> is `sq` attacked by side `by`
// L49     kq(side)  -> square index of that side's king
// L50     ic(side)  -> is that side's king in check
// L51-88  pm()  -> pseudo-legal moves (not yet filtered for self-check)
// L89-103 mk(mv) -> apply a move, return the resulting position
// L104    lm()  -> legal moves = pseudo-legal, minus those leaving
//         one's own king in check
// L105    ir()  -> true if the current position has occurred 3+ times
// L106-108 ev() -> static material evaluation, from side-to-move's view
// L110    pp()  -> push a pawn move, expanding to 4 moves if promoting
// L111    sf(str) -> parse a square string ("e4") to an index
// L112    ss(sq)  -> format a square index back to a string
// L113    ms(mv)  -> format a move as UCI text, e.g. "e7e8q"
// L114-122 nm()  -> negamax search with alpha-beta pruning
// L123-129 se()  -> top-level search: pick the best move at a depth
// L130-135 pf()  -> perft: count leaf nodes at a given depth (testing)
// L136-172 main() -> UCI loop: uci / isready / ucinewgame / position /
//         go / perft (non-standard, for testing) / stop / quit
