//! End-to-end tests for the `sukodu` binary's file-based solve mode
//! (`solve --size <n> --input-file <in> --output-file <out>`).
//!
//! These drive the actual compiled binary via `CARGO_BIN_EXE_sukodu`, exercising the real
//! argument parsing, file I/O, validation, and exit codes.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use sukodu::parse_grid;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_sukodu")
}

/// A unique temp path for this test run (no external temp-file crate needed).
fn tmp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("sukodu_test_{}_{}", std::process::id(), name));
    p
}

const PUZZLE_9X9: &str = "\
5 3 0 0 7 0 0 0 0
6 0 0 1 9 5 0 0 0
0 9 8 0 0 0 0 6 0
8 0 0 0 6 0 0 0 3
4 0 0 8 0 3 0 0 1
7 0 0 0 2 0 0 0 6
0 6 0 0 0 0 2 8 0
0 0 0 4 1 9 0 0 5
0 0 0 0 8 0 0 7 9
";

const SOLUTION_9X9: [usize; 81] = [
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

#[test]
fn solve_valid_file_writes_correct_solution() {
    let input = tmp_path("valid_in.txt");
    let output = tmp_path("valid_out.txt");
    fs::write(&input, PUZZLE_9X9).unwrap();

    let status = Command::new(bin())
        .args([
            "solve",
            "--size", "9",
            "--input-file", input.to_str().unwrap(),
            "--output-file", output.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "solve should exit 0 on a valid puzzle");

    let written = fs::read_to_string(&output).unwrap();
    let solved = parse_grid(&written, 9).expect("output file should be a valid grid");
    assert_eq!(solved, SOLUTION_9X9.to_vec(), "solution written to file is wrong");

    let _ = fs::remove_file(&input);
    let _ = fs::remove_file(&output);
}

#[test]
fn solve_malformed_file_fails() {
    let input = tmp_path("bad_in.txt");
    let output = tmp_path("bad_out.txt");
    fs::write(&input, "1 2 3\n").unwrap(); // wrong number of cells

    let out = Command::new(bin())
        .args([
            "solve",
            "--size", "9",
            "--input-file", input.to_str().unwrap(),
            "--output-file", output.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!out.status.success(), "malformed input should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("invalid puzzle format"), "stderr was: {}", stderr);
    assert!(!output.exists(), "no output file should be written on failure");

    let _ = fs::remove_file(&input);
}

#[test]
fn solve_missing_output_flag_fails() {
    let input = tmp_path("noout_in.txt");
    fs::write(&input, PUZZLE_9X9).unwrap();

    let out = Command::new(bin())
        .args([
            "solve",
            "--size", "9",
            "--input-file", input.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!out.status.success(), "missing --output-file should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--output-file is required"), "stderr was: {}", stderr);

    let _ = fs::remove_file(&input);
}

#[test]
fn solve_missing_size_flag_fails() {
    let input = tmp_path("nosize_in.txt");
    let output = tmp_path("nosize_out.txt");
    fs::write(&input, PUZZLE_9X9).unwrap();

    let out = Command::new(bin())
        .args([
            "solve",
            "--input-file", input.to_str().unwrap(),
            "--output-file", output.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!out.status.success(), "missing --size should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--size"), "stderr was: {}", stderr);

    let _ = fs::remove_file(&input);
}

#[test]
fn generate_then_solve_via_stdin_still_works() {
    // Regression guard: the original generate + stdin-solve paths must keep working.
    let gen_out = Command::new(bin())
        .args(["generate", "9", "easy"])
        .output()
        .unwrap();
    assert!(gen_out.status.success());
    let puzzle = gen_out.stdout;
    let parsed_puzzle = parse_grid(&String::from_utf8_lossy(&puzzle), 9)
        .expect("generated puzzle should be a valid grid");

    // Pipe the generated puzzle into `solve` via stdin.
    let mut child = Command::new(bin())
        .arg("solve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&puzzle).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "stdin solve should succeed");

    let solution = parse_grid(&String::from_utf8_lossy(&out.stdout), 9)
        .expect("stdin solve output should be a valid grid");
    // Solution must be complete and consistent with the original clues.
    assert!(solution.iter().all(|&v| (1..=9).contains(&v)), "solution must be fully filled");
    for i in 0..81 {
        if parsed_puzzle[i] > 0 {
            assert_eq!(solution[i], parsed_puzzle[i], "solution must respect clue at {}", i);
        }
    }
}
