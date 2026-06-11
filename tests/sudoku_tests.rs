use sudoku::{generate_board, solve_board, make_lines_cols, has_unique_solution};

fn run_scenario(size: usize, difficulty: &str) {
    let board = generate_board(size, difficulty);
    assert_eq!(board.len(), size * size);
    
    let clues_count = board.iter().filter(|&&val| val > 0).count();
    println!("Size {}x{} ({}) generated with {} clues", size, size, difficulty, clues_count);
    
    let lines_cols = make_lines_cols(size);
    assert!(
        has_unique_solution(&board, size, &lines_cols),
        "Generated board must have a unique solution"
    );
    
    let solution_opt = solve_board(&board, size);
    assert!(solution_opt.is_some(), "Solver failed to find a solution");
    let solution = solution_opt.unwrap();
    assert_eq!(solution.len(), size * size);
    
    for i in 0..(size * size) {
        if board[i] > 0 {
            assert_eq!(solution[i], board[i]);
        }
    }
}

// 9x9 Tests
#[test]
fn test_9x9_easy() { run_scenario(9, "easy"); }
#[test]
fn test_9x9_medium() { run_scenario(9, "medium"); }
#[test]
fn test_9x9_hard() { run_scenario(9, "hard"); }

// 16x16 Tests (Ignored by default for fast cargo test)
#[test]
#[ignore]
fn test_16x16_easy() { run_scenario(16, "easy"); }
#[test]
#[ignore]
fn test_16x16_medium() { run_scenario(16, "medium"); }
#[test]
#[ignore]
fn test_16x16_hard() { run_scenario(16, "hard"); }

// 25x25 Tests (Ignored by default for fast cargo test)
#[test]
#[ignore]
fn test_25x25_easy() { run_scenario(25, "easy"); }
#[test]
#[ignore]
fn test_25x25_medium() { run_scenario(25, "medium"); }
#[test]
#[ignore]
fn test_25x25_hard() { run_scenario(25, "hard"); }

// 36x36 Tests (Ignored by default for fast cargo test)
#[test]
#[ignore]
fn test_36x36_easy() { run_scenario(36, "easy"); }
#[test]
#[ignore]
fn test_36x36_medium() { run_scenario(36, "medium"); }
#[test]
#[ignore]
fn test_36x36_hard() { run_scenario(36, "hard"); }

// 49x49 Tests (Ignored by default for fast cargo test)
#[test]
#[ignore]
fn test_49x49_easy() { run_scenario(49, "easy"); }
#[test]
#[ignore]
fn test_49x49_medium() { run_scenario(49, "medium"); }
#[test]
#[ignore]
fn test_49x49_hard() { run_scenario(49, "hard"); }
