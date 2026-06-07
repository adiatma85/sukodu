use std::env;
use std::process;
use sudoku::dlx::ExactCover;
use sudoku::{generate, make_lines_cols, parse, print};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage to solve:   {} <size> < puzzle_file.txt", args[0]);
        eprintln!("Usage to generate: {} <size> <difficulty>", args[0]);
        eprintln!("Example:           {} 9 hard", args[0]);
        process::exit(1);
    }

    let size = match args[1].parse::<usize>() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error: Invalid size argument. It must be a positive integer.");
            process::exit(1);
        }
    };

    // Verify size is a perfect square
    let sqrt = (size as f64).sqrt().floor() as usize;
    if sqrt * sqrt != size || size == 0 {
        eprintln!("Error: Sudoku size must be a non-zero perfect square (e.g., 4, 9, 16, 25).");
        process::exit(1);
    }

    if args.len() > 2 {
        // Generate mode
        let difficulty = args[2].to_lowercase();
        if difficulty != "easy" && difficulty != "medium" && difficulty != "hard" {
            eprintln!("Error: Invalid difficulty. Choose 'easy', 'medium', or 'hard'.");
            process::exit(1);
        }
        generate(size, &difficulty);
    } else {
        // Solve mode
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
}
