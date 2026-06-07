# Sudoku Solver & Generator

A Sudoku solver and puzzle generator written in Rust, built on **Algorithm X (Dancing Links)** for exact cover.

Both binaries share a single library crate (`sudoku-core`) with the DLX, heap, and Sudoku helpers.

## Project Structure

```
sudoku/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs           # Library entry — re-exports shared modules
│   ├── dlx.rs           # Exact cover (Dancing Links + Algorithm X)
│   ├── heap.rs          # Binary min-heap for column selection
│   ├── sudoku.rs        # Sudoku helpers + puzzle generation
│   ├── bin/
│   │   ├── solver.rs    # Solver binary
│   │   └── generator.rs # Generator binary
```

## Usage

### Build

```bash
cargo build --release
```

### Solver

Reads a Sudoku grid from standard input and prints the solution.

```bash
./target/release/solver < puzzle.txt
```

Optional size argument (default 9):

```bash
./target/release/solver 16 < puzzle_16.txt
```

### Generator

Generates a new Sudoku puzzle and prints it to standard output.

```bash
./target/release/generator 9 easy
./target/release/generator 16 medium
./target/release/generator 25 hard
```

The generator can also solve puzzles (same interface as the solver):

```bash
./target/release/generator 9 < puzzle.txt
```

### Difficulty

| Difficulty | Clues (% of cells) |
|-----------|-------------------|
| easy      | ~45%              |
| medium    | ~35%              |
| hard      | ~23%              |

### Input Format

`n * n` whitespace-separated numbers, with empty cells as `0`:

```text
0 6 0 0 5 0 0 0 0
0 0 0 0 0 0 8 4 0
0 5 3 0 0 0 0 0 0
1 0 0 9 0 0 0 0 6
0 0 6 3 0 8 0 0 7
8 0 0 6 0 0 0 0 4
0 7 1 0 0 0 0 0 0
0 0 0 0 0 0 3 9 0
0 8 0 0 4 0 0 0 0
```

## How It Works

Sudoku is reducible to the **exact cover problem**: every cell, row, column, and block constraint maps to a column in a binary matrix. **Algorithm X** with **Dancing Links** solves this efficiently by covering and uncovering columns (and their intersecting rows) through circular doubly-linked lists.

Index-based references in flat `Vec<Cell>` structures avoid `unsafe` code while keeping the cache-friendly performance of the original C++ implementation.
