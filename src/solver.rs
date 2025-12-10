use std::collections::HashMap;
use std::time::Instant;

use crate::{board, cache};
use crate::board::Board;

const MAX_SCORE: i8 = 21;
const WIN_SCORE_BASE: i8 = 22;
const CACHE_DEPTH_SKIP: u64 = 2;
const MOVE_ORDERING_MAX_DEPTH: u64 = 20;

fn win_score(stones_for_winner: u32) -> i8 {
    (WIN_SCORE_BASE as i32 - stones_for_winner as i32) as i8
}

fn loss_score(stones_for_winner: u32) -> i8 {
    -win_score(stones_for_winner)
}

pub struct ScoreCache {
    table: HashMap<(u64, u64), i8>,
}

impl ScoreCache {
    pub fn new() -> Self {
        ScoreCache {
            table: HashMap::new(),
        }
    }

    pub fn lookup(&self, board: cache::CacheBoard) -> Option<i8> {
        let (first, second) = board.parts();
        self.table.get(&(first, second)).copied()
    }

    pub fn store(&mut self, board: cache::CacheBoard, score: i8) {
        let (first, second) = board.parts();
        self.table.insert((first, second), score);
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }
}

fn order_columns(board: Board, moves: crate::bitboard::BitBoard, depth: u64) -> [u64; 7] {
    let mut col_order = [3, 2, 4, 1, 5, 6, 0];
    if depth >= MOVE_ORDERING_MAX_DEPTH {
        return col_order;
    }

    let mut scores: [u32; 7] = [0; 7];
    for col in 0..scores.len() {
        let move_ = moves.for_column(col as u64);
        if move_.empty() {
            scores[col] = 0;
        } else {
            scores[col] = board.wins_involving(move_);
        }
        col_order[col] = col as u64;
        for i in (0..col).rev() {
            let s1 = scores[col_order[i + 1] as usize];
            let s0 = scores[col_order[i] as usize];
            if s1 > s0
                || (s1 == s0
                    && col_order[i + 1].abs_diff(3) < col_order[i].abs_diff(3))
            {
                col_order.swap(i + 1, i);
            }
        }
    }
    col_order
}

struct NegamaxSolver<'cache> {
    cache: &'cache mut ScoreCache,
    use_cache: bool,
    nodes: u64,
}

impl<'cache> NegamaxSolver<'cache> {
    fn negamax(&mut self, board: Board, depth: u64, mut alpha: i8, beta: i8) -> i8 {
        self.nodes += 1;

        // Terminal: the previous player already won.
        if board.other_won() {
            return loss_score(board.count_other());
        }

        let moves = board.moves();
        if moves.empty() {
            // Full board draw.
            return 0;
        }

        if self.use_cache && depth % CACHE_DEPTH_SKIP == 0 {
            if let Some(score) = self.cache.lookup(cache::board_exact(board)) {
                return score;
            }
        }

        if board.can_win(moves) {
            // Current player can win immediately.
            return win_score(board.count_current() + 1);
        }

        let moves = board.safe_moves(moves);
        if moves.empty() {
            // Every move lets the opponent win right away.
            return loss_score(board.count_other() + 1);
        }

        let mut best = -MAX_SCORE - 1;
        let mut cut = false;
        let col_order = order_columns(board, moves, depth);
        for col in col_order {
            let move_ = moves.for_column(col);
            if move_.empty() {
                continue;
            }
            let child = board.do_move(move_);
            let score = -self.negamax(child, depth + 1, -beta, -alpha);
            if score > best {
                best = score;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                cut = true;
                break;
            }
        }

        if self.use_cache && depth % CACHE_DEPTH_SKIP == 0 && !cut {
            self.cache.store(cache::board_exact(board), best);
        }

        best
    }
}

pub fn solve(board: Board, cache: &mut ScoreCache) -> (i8, u64) {
    let mut solver = NegamaxSolver {
        cache,
        use_cache: std::env::var_os("DISABLE_CACHE").is_none(),
        nodes: 0,
    };
    let score = solver.negamax(board, 0, -MAX_SCORE, MAX_SCORE);
    (score, solver.nodes)
}

pub fn solve_moves(moves: &str, cache: &mut ScoreCache) -> Result<(i8, u64, u32), String> {
    let board = board_from_moves(moves)?;
    let start = Instant::now();
    let (score, nodes) = solve(board, cache);
    let elapsed_micros = start.elapsed().as_micros() as u32;
    Ok((score, nodes, elapsed_micros))
}

pub fn board_from_moves(moves: &str) -> Result<Board, String> {
    let mut board = board::empty();
    for (idx, ch) in moves.chars().enumerate() {
        let digit = ch
            .to_digit(10)
            .ok_or_else(|| format!("Invalid character '{}' in position {}", ch, idx))?;
        if digit == 0 || digit > 7 {
            return Err(format!(
                "Column {} out of range at position {}",
                digit, idx
            ));
        }
        let column = digit - 1;
        let mv = board.moves().for_column(column as u64);
        if mv.empty() {
            return Err(format!(
                "Column {} is full when applying move index {}",
                digit, idx
            ));
        }
        board = board.do_move(mv);
    }
    Ok(board)
}
