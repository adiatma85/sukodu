# Sudoku Solver & Generator

A Sudoku solver and puzzle generator written in Rust, built on **Algorithm X (Dancing Links)** for exact cover. It ships as a single `sukodu` binary with `generate` and `solve` subcommands, including the ability to **solve a puzzle from a photo** via Tesseract OCR.

## Project Structure

```
sukodu/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs           # Library entry — re-exports shared modules
│   ├── dlx.rs           # Exact cover (Dancing Links + Algorithm X)
│   ├── heap.rs          # Binary min-heap for column selection
│   ├── sudoku.rs        # Sudoku helpers, generation, parse_grid/format_board
│   ├── vision.rs        # Image pipeline: grid detection, warp, Tesseract OCR
│   └── bin/
│       └── main.rs      # CLI: `sukodu generate` / `sukodu solve`
└── tests/
    ├── sudoku_tests.rs  # solve / generate / parse_grid
    ├── cli_tests.rs     # binary file-based solve
    └── vision_tests.rs  # image scan + solve (needs Tesseract)
```

## Usage

### Build

```bash
cargo build --release
```

The binary is then at `./target/release/sukodu` (examples below use `cargo run --` for convenience).

### Generate

Generate a new puzzle and print it to standard output:

```bash
sukodu generate                 # 9x9, medium (defaults)
sukodu generate 9 easy
sukodu generate 16 medium
sukodu generate 25 hard
```

Size must be a non-zero perfect square (4, 9, 16, 25, …); difficulty is `easy`, `medium`, or `hard`.

### Solve from stdin

Read a puzzle from standard input and print the solution:

```bash
sukodu solve < puzzle.txt        # 9x9 (default)
sukodu solve 16 < puzzle_16.txt  # optional positional size
```

### Solve from a file (`--input-file` / `--output-file`)

Read a puzzle from a text file, validate its format, solve it, and write the solution to a file. The `--size`, `--input-file`, and `--output-file` flags are all required:

```bash
sukodu solve --size 9 --input-file ./puzzle.txt --output-file ./solution.txt
```

The input is validated before solving — a clear error is printed (and the process exits non-zero) if the file has the wrong number of cells, a non-numeric token, or an out-of-range value. The solution file uses the same numeric format as the input, so it round-trips back through the solver.

### Solve from an image

Solve a puzzle from a photo or screenshot (see [Image scanning](#image-scanning) for the prerequisite):

```bash
sukodu solve --image ./sudoku.png            # auto-detects 9x9 vs 16x16
sukodu solve --image ./sudoku.png --size 9   # skip auto-detection
```

To save the solved puzzle as an image, displaying the solution overlayed on the grid, use the `--output-image` flag:

```bash
sukodu solve --image ./sudoku.png --output-image ./solved.png
```

This warps and threshold-corrects the puzzle grid, solves it, and draws the solved digits onto the empty cells using a distinct accent color. Only PNG, JPG, and JPEG formats are supported.


### Difficulty

| Difficulty | Clues (% of cells) |
|-----------|-------------------|
| easy      | ~45%              |
| medium    | ~35%              |
| hard      | ~23%              |

### Input Format

`n * n` whitespace-separated numbers, with empty cells as `0` (for sizes above 9, values 10–16 are written as the numbers `10`–`16`):

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

## Image scanning

`sukodu solve --image` turns a picture of a puzzle into a board and solves it. The pipeline is:

1. **Load & threshold** — load the image as grayscale and apply an adaptive threshold.
2. **Locate the grid** — find the largest square contour and its four corners.
3. **Perspective-correct** — warp the grid to a flat 576×576 square (handles camera angle).
4. **Detect size** — auto-detect 9×9 vs 16×16 from grid-line spacing (override with `--size`).
5. **Recognize digits** — crop each cell, clean it up, and read it with **Tesseract** in single-character mode, restricted to the valid Sudoku alphabet.
6. **Solve** — feed the recognized board to the DLX solver.

### Prerequisite: Tesseract

Digit recognition uses Tesseract via the `leptess` crate, which links against the system Tesseract and Leptonica libraries. Install them once:

```bash
# macOS
brew install tesseract leptonica

# Debian / Ubuntu
sudo apt-get install tesseract-ocr libtesseract-dev libleptonica-dev
```

If Tesseract is not installed, `solve --image` prints a clear error telling you to install it; all other commands work without it.

> **Note:** the scanner targets clean, printed grids photographed roughly straight-on. It does not handle handwriting, and accuracy degrades on skewed or low-contrast photos. Always sanity-check the "RECOGNIZED BOARD" it prints before trusting the solution.

## How It Works

Sudoku is reducible to the **exact cover problem**: every cell, row, column, and block constraint maps to a column in a binary matrix. **Algorithm X** with **Dancing Links** solves this efficiently by covering and uncovering columns (and their intersecting rows) through circular doubly-linked lists.

Index-based references in flat `Vec<Cell>` structures avoid `unsafe` code while keeping the cache-friendly performance of the original C++ implementation.

## Testing

```bash
cargo test                 # fast suite: solve/generate (9x9), parse_grid, CLI, image scan
cargo test -- --ignored    # large boards (16x16 … 49x49) — slower
```

What the suite covers:

- **`sudoku_tests.rs`** — generate → unique-solution → solve across sizes, plus `parse_grid` validation and `format_board` round-tripping.
- **`cli_tests.rs`** — drives the compiled binary end-to-end for the file-based solve flow (valid puzzle, malformed input, missing flags) and the stdin path.
- **`vision_tests.rs`** — renders synthetic puzzles, runs the full image pipeline, and solves the recognized board. These require Tesseract; if it is not installed they **skip** rather than fail, so the rest of the suite still runs.

### Continuous Integration (CI)

A GitHub Actions workflow is configured to automatically run on every push and pull request targeting the `main` branch. It performs the following checks:
1. **Formatting**: Runs `cargo fmt --check` to ensure style consistency.
2. **Linting**: Runs `cargo clippy --all-targets -- -D warnings` to catch code smells and idiomatic issues.
3. **Testing**: Installs system prerequisites (`tesseract-ocr`, `libtesseract-dev`, `libleptonica-dev`) and runs `cargo test --all-targets` to verify correctness.

### Git Hooks

A shareable pre-commit hook is provided in the `.githooks/` directory to run formatting, Clippy, and tests locally before each commit.

To enable the pre-commit hook:

```bash
git config core.hooksPath .githooks
```



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

