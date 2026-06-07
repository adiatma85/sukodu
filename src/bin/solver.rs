use std::env;
use std::process;
use sudoku::dlx::ExactCover;
use sudoku::{make_lines_cols, parse, print};

fn main() {
    let args: Vec<String> = env::args().collect();
    let size = if args.len() > 1 {
        match args[1].parse::<usize>() {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Invalid size argument");
                process::exit(1);
            }
        }
    } else {
        9
    };

    let num_cols = size * size * 4;
    let lines_cols = make_lines_cols(size);
    let mut problem = ExactCover::new(num_cols, lines_cols);

    if let Err(e) = parse(&mut problem, size) {
        eprintln!("Error parsing sudoku: {}", e);
        process::exit(1);
    }

    if !problem.solve() {
        eprintln!("No solution");
        process::exit(1);
    }

    print(&problem, size);
}
