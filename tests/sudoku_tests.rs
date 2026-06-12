use sukodu::{format_board, generate_board, has_unique_solution, make_lines_cols, parse_grid, solve_board};

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

// ---- parse_grid / format_board (text I/O) ----

#[test]
fn test_parse_grid_valid() {
    let input = "0 6 0 0 5 0 0 0 0\n\
                 0 0 0 0 0 0 8 4 0\n\
                 0 5 3 0 0 0 0 0 0\n\
                 1 0 0 9 0 0 0 0 6\n\
                 0 0 6 3 0 8 0 0 7\n\
                 8 0 0 6 0 0 0 0 4\n\
                 0 7 1 0 0 0 0 0 0\n\
                 0 0 0 0 0 0 3 9 0\n\
                 0 8 0 0 4 0 0 0 0\n";
    let board = parse_grid(input, 9).expect("valid 9x9 input should parse");
    assert_eq!(board.len(), 81);
    assert_eq!(board[1], 6);
    assert_eq!(board[0], 0);
}

#[test]
fn test_parse_grid_wrong_count() {
    let err = parse_grid("1 2 3", 9).unwrap_err();
    assert!(err.contains("expected exactly 81"), "got: {}", err);
}

#[test]
fn test_parse_grid_out_of_range() {
    // value 10 is invalid for a 9x9 grid
    let mut input = "10 ".repeat(81);
    input.pop();
    let err = parse_grid(&input, 9).unwrap_err();
    assert!(err.contains("out of range"), "got: {}", err);
}

#[test]
fn test_parse_grid_non_numeric() {
    let input = "x ".repeat(81);
    let err = parse_grid(&input, 9).unwrap_err();
    assert!(err.contains("invalid token"), "got: {}", err);
}

#[test]
fn test_format_board_round_trips_through_parse_grid() {
    let board = generate_board(9, "easy");
    let text = format_board(&board, 9);
    let reparsed = parse_grid(&text, 9).expect("formatted board should re-parse");
    assert_eq!(reparsed, board);
}

#[test]
fn test_solve_from_parsed_grid() {
    // End-to-end at the library level: generate -> format -> parse -> solve.
    let board = generate_board(9, "easy");
    let text = format_board(&board, 9);
    let parsed = parse_grid(&text, 9).unwrap();
    let solved = solve_board(&parsed, 9).expect("parsed puzzle should solve");
    for i in 0..81 {
        if board[i] > 0 {
            assert_eq!(solved[i], board[i]);
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
