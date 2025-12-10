# Connect Four Solver

This is a program that solves the game of Connect Four. In other words, it determines the outcome when both players play optimally (the first moving player can always win).

The program is written in Rust. Optimizations:

- Bit hacks
- SIMD
- Caching
- Cache key state reduction (mirroring and ignoring checkers that can't influence the outcome)
- Move ordering

A blog post with more information is available [here](https://jorrid.com/posts/the-wondrous-world-of-connect-four-bit-boards/).

# Running

To time how long it takes to solve, run:

```
time rustup run nightly cargo run --release
```

## Additional binaries

- `cargo run --release --bin full_solve` — identical to the original `cargo run --release`, solves the empty board.
- `cargo run --release --bin score -- 4455` — evaluates a specific move sequence and prints the exact game-theoretic score (positive = player to move wins, negative = player to move loses, 0 = draw). If no argument is passed, positions are read from stdin (one per line).

## Test data check

To verify the solver against the provided datasets (defaults to `test-data/middle-medium`):

```
./scripts/check_test_data.sh
```

# Benchmarks

Some operations have multiple implementations (mainly SIMD or not SIMD). There are some benchmarks that help to decide what is faster.

```
rustup run nightly cargo bench
```
