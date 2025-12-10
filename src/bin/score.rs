#![feature(test)]
#![feature(portable_simd)]

use std::env;
use std::io::{self, BufRead};

use connect4_rust::solver::{self, ScoreCache};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = ScoreCache::new();

    // If positions are passed as CLI args, use them; otherwise read from stdin.
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            handle_position(line, &mut cache)?;
        }
    } else {
        for arg in args {
            handle_position(arg, &mut cache)?;
        }
    }

    Ok(())
}

fn handle_position(line: String, cache: &mut ScoreCache) -> Result<(), Box<dyn std::error::Error>> {
    let position = line.split_whitespace().next().unwrap_or("").to_string();
    if position.is_empty() {
        return Ok(());
    }
    match solver::solve_moves(&position, cache) {
        Ok((score, nodes, micros)) => {
            let outcome = if score > 0 {
                "win"
            } else if score < 0 {
                "loss"
            } else {
                "draw"
            };
            println!(
                "{} {} {} nodes={} time_us={}",
                position, score, outcome, nodes, micros
            );
        }
        Err(err) => {
            eprintln!("Error on '{}': {}", position, err);
        }
    }
    Ok(())
}
