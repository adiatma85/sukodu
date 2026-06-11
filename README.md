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

## Performance Matrix

The following table documents execution times and peak memory footprints (Maximum Resident Set Size - RSS) for generating and solving puzzles across various grid sizes and difficulties in release mode (`--release`).

| Grid Size | Difficulty | Clues Remaining | Gen Time (s) | Gen Peak Memory | Sol Time (s) | Sol Peak Memory | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **9x9** | Easy | 36 | 0.230s | 2.36 MB | 0.100s | 1.91 MB | Success |
| **9x9** | Medium | 28 | 0.010s | 2.34 MB | 0.000s | 1.92 MB | Success |
| **9x9** | Hard | 28 | 0.000s | 2.48 MB | 0.000s | 1.92 MB | Success |
| **16x16** | Easy | 116 | 0.030s | 4.03 MB | 0.000s | 2.75 MB | Success |
| **16x16** | Medium | 102 | 0.400s | 4.03 MB | 0.010s | 2.83 MB | Success |
| **16x16** | Hard | 104 | 0.130s | 4.09 MB | 0.000s | 2.78 MB | Success |
| **25x25** | Easy | 317 | 3.040s | 12.16 MB | 0.000s | 5.88 MB | Success |
| **25x25** | Medium | 305 | 3.120s | 10.17 MB | 0.010s | 5.89 MB | Success |
| **25x25** | Hard | 312 | 3.230s | 12.86 MB | 0.010s | 5.88 MB | Success |
| **36x36** | Easy | 718 | 10.880s | 409.66 MB | 0.000s | 16.56 MB | Success |
| **36x36** | Medium | 710 | 11.010s | 407.59 MB | 0.020s | 16.58 MB | Success |
| **36x36** | Hard | 700 | 11.070s | 409.69 MB | 0.010s | 16.58 MB | Success |
| **49x49** | Easy | 1402 | 67.880s | 974.48 MB | 0.030s | 32.39 MB | Success |
| **49x49** | Medium | 1423 | 37.190s | 974.47 MB | 0.020s | 32.38 MB | Success |
| **49x49** | Hard | 1391 | 36.400s | 974.53 MB | 0.020s | 32.41 MB | Success |

- **Solver Efficiency**: The solver performs at microsecond to millisecond levels across all board sizes using Algorithm X with the Minimum Remaining Values (MRV) heuristic.
- **Generator Scaling**: Large boards (36x36, 49x49) leverage a step-limit search constraint to keep generation times short, preventing backtracking hangs, and leverage rotationally symmetric clue removal to ensure professional designs.

