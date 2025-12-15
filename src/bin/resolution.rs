#![feature(test)]
#![feature(portable_simd)]

#[path = "../bitboard.rs"]
mod bitboard;
#[path = "../board.rs"]
mod board;
#[path = "../cache.rs"]
mod cache;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;
use std::collections::HashMap;

use board::Board;
// Depth-limited mate-distance search (plies). We search far enough to reach the
// end of the game for `test-data/end-easy` (move sequences are 29..41 plies long,
// so remaining depth is at most 13).
struct Solver {
    cache: HashMap<(u64, u64, u8), i32>,
    nodes: u64,
    max_depth: u8,
}

impl Solver {
    fn new(max_depth: u8) -> Self {
        Self {
            cache: HashMap::new(),
            nodes: 0,
            max_depth,
        }
    }

    fn key(board: Board, depth: u8) -> (u64, u64, u8) {
        let (b1, b2) = cache::board(board).parts();
        (b1, b2, depth)
    }

    // Returns signed ply distance (current player viewpoint):
    //   >0 : win in N plies (including current move)
    //    0 : draw/unknown within horizon
    //   <0 : loss in N plies
    fn solve(&mut self, board: Board, depth: u8) -> i32 {
        self.nodes += 1;

        // Terminal: the previous player already won.
        if board.other_won() {
            return -1;
        }

        let key = Self::key(board, depth);
        if let Some(v) = self.cache.get(&key) {
            return *v;
        }

        let moves = board.moves();
        if moves.empty() {
            // Full board draw.
            self.cache.insert(key, 0);
            return 0;
        }

        // Immediate win available.
        if board.can_win(moves) {
            self.cache.insert(key, 1);
            return 1;
        }

        let moves = board.safe_moves(moves);

        // No safe moves means every move allows the opponent to win right away.
        if moves.empty() {
            self.cache.insert(key, -2);
            return -2;
        }

        if depth == 0 {
            self.cache.insert(key, 0);
            return 0;
        }

        let mut best: Option<i32> = None;

        for col in [3, 2, 4, 1, 5, 6, 0] {
            let mv = moves.for_column(col);
            if mv.empty() {
                continue;
            }
            let child = board.do_move(mv);
            let child_score = self.solve(child, depth - 1);

            // Translate child's perspective to current and add this ply.
            let score = if child_score > 0 {
                -(child_score + 1)
            } else if child_score < 0 {
                (-child_score) + 1
            } else {
                0
            };

            match best {
                None => best = Some(score),
                Some(b) => {
                    if score > 0 {
                        if b <= 0 || score < b {
                            best = Some(score);
                        }
                    } else if score == 0 {
                        if b < 0 {
                            best = Some(score);
                        }
                    } else {
                        if b < 0 && score < b {
                            best = Some(score);
                        }
                    }
                }
            }
        }

        let result = best.unwrap_or(0);
        self.cache.insert(key, result);
        result
    }
}

fn ply_distance_to_dataset_score(ply_distance: i32, stones_played: u32) -> i32 {
    if ply_distance == 0 {
        return 0;
    }
    let plies = ply_distance.unsigned_abs();
    let total_at_win = stones_played + plies;
    let stones_for_winner = ((total_at_win + 1) / 2) as i32; // ceil(total_at_win / 2)
    let score = 22 - stones_for_winner;
    if ply_distance > 0 { score } else { -score }
}

fn parse_line(line: &str) -> Option<(Board, i32)> {
    if line.trim().is_empty() {
        return None;
    }
    let mut parts = line.split_whitespace();
    let moves_str = parts.next()?;
    let label: i32 = parts.next()?.parse().ok()?;

    let mut board = board::empty();
    for c in moves_str.chars() {
        let col = c.to_digit(10)? as u64;
        if col == 0 || col > 7 {
            return None;
        }
        let moves = board.moves();
        let move_ = moves.for_column(col - 1);
        if move_.empty() {
            // Invalid position; skip.
            return None;
        }
        board = board.do_move(move_);
    }
    Some((board, label))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "test-data/middle-medium".to_string());

    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let mut total = 0usize;
    let mut correct = 0usize;
    let mut total_nodes: u64 = 0;
    let mut total_time: f64 = 0.0;

    // Max remaining moves in `end-easy` is 13 (42 - 29).
    let mut solver = Solver::new(13);

    for line in reader.lines() {
        let line = line?;
        let (board, label) = match parse_line(&line) {
            Some(v) => v,
            None => continue,
        };
        total += 1;

        let stones_played = board.total_stones();
        let remaining = 42u8.saturating_sub(stones_played as u8);

        let nodes_before = solver.nodes;
        let start = Instant::now();
        let ply_distance = solver.solve(board, remaining.min(solver.max_depth));
        let predicted = ply_distance_to_dataset_score(ply_distance, stones_played);
        let elapsed = start.elapsed().as_secs_f64();
        let nodes_for_pos = solver.nodes - nodes_before;

        if predicted == label {
            correct += 1;
        }
        total_nodes += nodes_for_pos;
        total_time += elapsed;
    }

    println!("File: {}", path);
    let accuracy_pct = (correct as f64) * 100.0 / (total as f64);
    let mean_time_ms = (total_time / total as f64) * 1000.0;
    let mean_nodes = total_nodes as f64 / total as f64;
    let kpos_per_s = (total_nodes as f64 / total_time) / 1000.0;

    println!("--- Benchmark Results ---");
    println!("Accuracy: {} / {} ({:.2}%)", correct, total, accuracy_pct);
    println!("Mean time per position: {:.6}ms", mean_time_ms);
    println!("Mean nodes explored: {:.0}", mean_nodes);
    println!("Solver speed: {:.2} kpos/s", kpos_per_s);

    Ok(())
}
