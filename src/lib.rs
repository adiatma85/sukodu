pub mod dlx;
pub mod heap;
pub mod sudoku;

#[cfg(not(target_arch = "wasm32"))]
pub mod vision;

pub use sudoku::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate_sudoku_wasm(
    size: usize,
    difficulty: &str,
    seed: u64,
) -> Result<Vec<usize>, String> {
    let sqrt = (size as f64).sqrt().floor() as usize;
    if size == 0 || sqrt * sqrt != size {
        return Err("Size must be a non-zero perfect square (e.g. 4, 9, 16, 25).".to_string());
    }
    let difficulty_lc = difficulty.to_lowercase();
    if difficulty_lc != "easy" && difficulty_lc != "medium" && difficulty_lc != "hard" {
        return Err("Difficulty must be 'easy', 'medium', or 'hard'.".to_string());
    }
    Ok(sudoku::generate_board_seeded(size, &difficulty_lc, seed))
}

#[wasm_bindgen]
pub fn solve_sudoku_wasm(board: Vec<usize>, size: usize) -> Result<Vec<usize>, String> {
    let expected_len = size * size;
    if board.len() != expected_len {
        return Err(format!(
            "Invalid board length: expected {}, got {}.",
            expected_len,
            board.len()
        ));
    }
    match sudoku::solve_board(&board, size) {
        Some(solved) => Ok(solved),
        None => Err("No solution found for this Sudoku puzzle.".to_string()),
    }
}
