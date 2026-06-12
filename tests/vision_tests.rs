//! Image-scanning tests.
//!
//! These exercise the full vision pipeline: render a synthetic puzzle image, locate and
//! warp the grid, OCR the digits with Tesseract, then solve the recognized board.
//!
//! The OCR stage needs Tesseract installed (`brew install tesseract leptonica`, or
//! `apt-get install tesseract-ocr libtesseract-dev libleptonica-dev`). When it is missing,
//! `scan_board` returns an error and these tests **skip** instead of failing, so the rest of
//! the suite still runs on a machine without Tesseract.

use sukodu::vision::{
    detect_grid_size, find_grid_corners, generate_synthetic_image, load_image, scan_board,
    warp_grid,
};
use sukodu::{format_board, has_unique_solution, make_lines_cols, solve_board};

/// Builds a valid, fully-solved 16x16 board via the canonical box-shifting construction
/// (box size 4): rows, columns, and 4x4 blocks each contain every value 1..=16.
fn solved_16x16() -> Vec<usize> {
    let n = 16usize;
    let b = 4usize; // sqrt(16)
    let mut grid = vec![0usize; n * n];
    for r in 0..n {
        for c in 0..n {
            grid[r * n + c] = (b * (r % b) + r / b + c) % n + 1;
        }
    }
    grid
}

/// Asserts that `board` is a complete, valid Sudoku of the given size.
fn assert_valid_solution(board: &[usize], size: usize) {
    let b = (size as f64).sqrt() as usize;
    let full: u64 = (1..=size).map(|v| 1u64 << v).sum();
    let seen = |mask: u64, label: &str, idx: usize| {
        assert_eq!(mask, full, "{} {} is not a complete 1..={} set", label, idx, size);
    };
    for i in 0..size {
        let mut row = 0u64;
        let mut col = 0u64;
        for j in 0..size {
            row |= 1 << board[i * size + j];
            col |= 1 << board[j * size + i];
        }
        seen(row, "row", i);
        seen(col, "column", i);
    }
    for br in 0..b {
        for bc in 0..b {
            let mut box_mask = 0u64;
            for r in 0..b {
                for c in 0..b {
                    box_mask |= 1 << board[(br * b + r) * size + (bc * b + c)];
                }
            }
            seen(box_mask, "box", br * b + bc);
        }
    }
}

/// A fixed, fully-solved 9x9 board. Using a deterministic puzzle (rather than the
/// time-seeded generator) keeps the OCR accuracy assertion reproducible run-to-run.
const SOLVED_9X9: [usize; 81] = [
    5, 3, 4, 6, 7, 8, 9, 1, 2,
    6, 7, 2, 1, 9, 5, 3, 4, 8,
    1, 9, 8, 3, 4, 2, 5, 6, 7,
    8, 5, 9, 7, 6, 1, 4, 2, 3,
    4, 2, 6, 8, 5, 3, 7, 9, 1,
    7, 1, 3, 9, 2, 4, 8, 5, 6,
    9, 6, 1, 5, 3, 7, 2, 8, 4,
    2, 8, 7, 4, 1, 9, 6, 3, 5,
    3, 4, 5, 2, 8, 6, 1, 7, 9,
];

/// The same board with about half the cells blanked out — a solvable puzzle whose clues we
/// expect the scanner to recover.
const PUZZLE_9X9: [usize; 81] = [
    5, 3, 0, 0, 7, 0, 0, 0, 0,
    6, 0, 0, 1, 9, 5, 0, 0, 0,
    0, 9, 8, 0, 0, 0, 0, 6, 0,
    8, 0, 0, 0, 6, 0, 0, 0, 3,
    4, 0, 0, 8, 0, 3, 0, 0, 1,
    7, 0, 0, 0, 2, 0, 0, 0, 6,
    0, 6, 0, 0, 0, 0, 2, 8, 0,
    0, 0, 0, 4, 1, 9, 0, 0, 5,
    0, 0, 0, 0, 8, 0, 0, 7, 9,
];

/// Returns the recognized board, or `None` if Tesseract is unavailable (test should skip).
fn scan_image(path: &str, size: usize) -> Option<Vec<usize>> {
    let gray = load_image(path).unwrap();
    let binary = imageproc::contrast::adaptive_threshold(&gray, 15, 10);
    let corners = find_grid_corners(&binary).unwrap();
    let warped = warp_grid(&gray, corners).unwrap();

    let detected = detect_grid_size(&warped);
    assert_eq!(detected, size, "auto-detected grid size should be {}", size);

    match scan_board(&warped, size) {
        Ok(board) => Some(board),
        Err(e) => {
            eprintln!("Skipping image test (Tesseract unavailable): {}", e);
            None
        }
    }
}

/// Accuracy of recognized clues vs. the ground-truth puzzle.
fn clue_accuracy(recognized: &[usize], truth: &[usize]) -> (usize, usize, usize) {
    let (mut clues, mut matches, mut empty_mismatches) = (0, 0, 0);
    for i in 0..truth.len() {
        if truth[i] > 0 {
            clues += 1;
            if recognized[i] == truth[i] {
                matches += 1;
            }
        } else if recognized[i] > 0 {
            empty_mismatches += 1;
        }
    }
    (clues, matches, empty_mismatches)
}

#[test]
fn test_scan_9x9_accuracy() {
    let path = "tests/sudoku_synthetic_9x9.png";
    generate_synthetic_image(path, 9, &PUZZLE_9X9).unwrap();

    let recognized = match scan_image(path, 9) {
        Some(r) => r,
        None => return,
    };

    let (clues, matches, empty_mismatches) = clue_accuracy(&recognized, &PUZZLE_9X9);
    let accuracy = matches as f32 / clues as f32;
    println!(
        "9x9 OCR — clues: {}, matches: {}, empty mismatches: {}, accuracy: {:.1}%",
        clues, matches, empty_mismatches, accuracy * 100.0
    );
    // The key safety property is zero false positives (no empty cell read as a digit); a
    // dropped clue is recoverable, a wrong clue is not. ~93% is the synthetic-font ceiling;
    // real printed puzzles typically score higher.
    assert!(
        accuracy >= 0.90 && empty_mismatches == 0,
        "expected >=90% clue accuracy and no false positives, got {:.1}% with {} empty mismatches",
        accuracy * 100.0,
        empty_mismatches
    );
}

#[test]
fn test_scan_and_solve_9x9_end_to_end() {
    // This is the core "see the image and solve it" check.
    let path = "tests/sudoku_synthetic_9x9.png";
    generate_synthetic_image(path, 9, &PUZZLE_9X9).unwrap();

    let recognized = match scan_image(path, 9) {
        Some(r) => r,
        None => return,
    };

    // No recognized digit may be wrong (it may be dropped, but never misread): every
    // non-empty recognized cell must agree with the true solution.
    for i in 0..81 {
        if recognized[i] != 0 {
            assert_eq!(
                recognized[i], SOLVED_9X9[i],
                "cell {} misrecognized as {} (true value {})",
                i, recognized[i], SOLVED_9X9[i]
            );
        }
    }

    let solved = solve_board(&recognized, 9)
        .expect("the recognized board should be solvable");

    // The solver's output must be the known unique solution.
    assert_eq!(
        solved,
        SOLVED_9X9.to_vec(),
        "scanned-and-solved board did not match the known solution:\n{}",
        format_board(&solved, 9)
    );
}

#[test]
fn test_scan_and_solve_16x16_end_to_end() {
    // 16x16 uses A..G for values 10..16 and has much smaller cells, so recognition is harder
    // than 9x9. This renders a *valid* 16x16 puzzle, scans it, and solves it — a correct
    // solve confirms the scan recovered the board.
    let size = 16;
    let solution = solved_16x16();
    assert_valid_solution(&solution, size); // sanity-check the constructed board

    // The canonical construction is highly symmetric, so blank only a sparse, scattered set of
    // cells. This keeps the puzzle uniquely solvable and heavily over-constrained, so the few
    // clues OCR may drop can never change the solution.
    let mut puzzle = solution.clone();
    for i in 0..(size * size) {
        if i % 17 == 0 {
            puzzle[i] = 0;
        }
    }
    let lines_cols = make_lines_cols(size);
    assert!(
        has_unique_solution(&puzzle, size, &lines_cols),
        "test puzzle must have a unique solution"
    );

    let path = "tests/sudoku_synthetic_16x16.png";
    generate_synthetic_image(path, size, &puzzle).unwrap();

    let recognized = match scan_image(path, size) {
        Some(r) => r,
        None => return,
    };

    // Categorise any OCR errors (drops vs. misreads) for diagnosis.
    let (mut drops, mut misreads) = (0, 0);
    for i in 0..(size * size) {
        if puzzle[i] > 0 && recognized[i] != puzzle[i] {
            if recognized[i] == 0 {
                drops += 1;
            } else {
                misreads += 1;
                println!("  misread at cell {}: truth={} got={}", i, puzzle[i], recognized[i]);
            }
        }
    }
    let false_positives = (0..size * size)
        .filter(|&i| puzzle[i] == 0 && recognized[i] != 0)
        .count();
    println!(
        "16x16 OCR — drops: {}, misreads: {}, false positives: {}",
        drops, misreads, false_positives
    );

    // A misread or a false positive feeds the solver a wrong clue; a drop is recoverable.
    assert_eq!(misreads, 0, "OCR misread a clue (wrong digit) — see output above");
    assert_eq!(false_positives, 0, "OCR read an empty cell as a digit");

    let solved = solve_board(&recognized, size).expect("the recognized board should be solvable");
    assert_valid_solution(&solved, size);
    assert_eq!(
        solved, solution,
        "scanned-and-solved 16x16 board did not match the known solution"
    );
}

#[test]
fn test_real_world_9x9_if_present() {
    // Optional: only runs if a real photo is supplied. Asserts the grid is located and sized
    // correctly; recognition accuracy on arbitrary photos is not asserted.
    let path = "tests/sudoku_9x9.png";
    if !std::path::Path::new(path).exists() {
        println!("Skipping real-world 9x9 test: {} not found", path);
        return;
    }

    let gray = load_image(path).unwrap();
    let binary = imageproc::contrast::adaptive_threshold(&gray, 15, 10);
    let corners = find_grid_corners(&binary).unwrap();
    let warped = warp_grid(&gray, corners).unwrap();
    assert_eq!(detect_grid_size(&warped), 9, "real-world image should detect as 9x9");

    match scan_board(&warped, 9) {
        Ok(recognized) => {
            println!("Real-world recognized board:\n{}", format_board(&recognized, 9));
        }
        Err(e) => println!("Skipping real-world OCR (Tesseract unavailable): {}", e),
    }
}
