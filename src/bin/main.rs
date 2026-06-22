use std::env;
use std::fs;
use std::path::Path;
use std::process;
use sukodu::dlx::ExactCover;
use sukodu::vision;
use sukodu::{format_board, generate, make_lines_cols, parse, parse_grid, print, solve_board};

fn print_usage(program_name: &str) {
    eprintln!("sukodu - Sudoku Solver & Generator");
    eprintln!();
    eprintln!("Usage:");
    eprintln!(
        "  {} generate [size] [difficulty]      - Generate a new puzzle (default: 9 medium)",
        program_name
    );
    eprintln!(
        "  {} solve [size]                      - Solve a text puzzle from stdin (default size: 9)",
        program_name
    );
    eprintln!(
        "  {} solve --image <path> [--size n] [--output-image <path>]",
        program_name
    );
    eprintln!("                                       - Solve a puzzle from an image (PNG/JPG)");
    eprintln!(
        "  {} solve --size <n> --input-file <in> --output-file <out>",
        program_name
    );
    eprintln!(
        "                                       - Solve a text puzzle from a file into a file"
    );
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} generate 9 easy", program_name);
    eprintln!("  {} solve < puzzle.txt", program_name);
    eprintln!("  {} solve --image ./sudoku.png", program_name);
    eprintln!(
        "  {} solve --image ./sudoku.png --output-image ./solved.png",
        program_name
    );
    eprintln!(
        "  {} solve --size 9 --input-file puzzle.txt --output-file solution.txt",
        program_name
    );
}

/// Exits with an error unless `size` is a non-zero perfect square.
fn validate_perfect_square(size: usize) {
    let sqrt = (size as f64).sqrt().floor() as usize;
    if size == 0 || sqrt * sqrt != size {
        eprintln!("Error: Sudoku size must be a non-zero perfect square (e.g. 4, 9, 16, 25).");
        process::exit(1);
    }
}

/// Returns the value following a flag at index `i`, or exits with a usage error if absent.
fn flag_value<'a>(args: &'a [String], i: usize, flag: &str, program_name: &str) -> &'a str {
    if i + 1 >= args.len() {
        eprintln!("Error: missing value for {}.", flag);
        print_usage(program_name);
        process::exit(1);
    }
    args[i + 1].as_str()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = if !args.is_empty() { &args[0] } else { "sukodu" };

    if args.len() < 2 {
        print_usage(program_name);
        process::exit(1);
    }

    let command = args[1].as_str();
    match command {
        "generate" => {
            let size = if args.len() > 2 {
                match args[2].parse::<usize>() {
                    Ok(s) => s,
                    Err(_) => {
                        eprintln!("Error: Invalid size. Must be a positive integer.");
                        process::exit(1);
                    }
                }
            } else {
                9
            };

            let difficulty = if args.len() > 3 {
                args[3].to_lowercase()
            } else {
                "medium".to_string()
            };

            if difficulty != "easy" && difficulty != "medium" && difficulty != "hard" {
                eprintln!("Error: Invalid difficulty. Choose 'easy', 'medium', or 'hard'.");
                process::exit(1);
            }

            validate_perfect_square(size);

            generate(size, &difficulty);
        }

        "solve" => {
            // Scan the flags that follow `solve`. Supported:
            //   --image <path>        solve from an image
            //   --input-file <path>   solve from a text file (requires --output-file + --size)
            //   --output-file <path>  write the solution to a text file
            //   --size <n>            grid size (overrides image auto-detection)
            //   <n>                   bare positional size (stdin mode, backward compatible)
            let solve_args = &args[2..];
            let mut image_path: Option<&str> = None;
            let mut output_image_path: Option<&str> = None;
            let mut input_file: Option<&str> = None;
            let mut output_file: Option<&str> = None;
            let mut size_flag: Option<usize> = None;
            let mut positional_size: Option<usize> = None;

            let mut i = 0;
            while i < solve_args.len() {
                match solve_args[i].as_str() {
                    "--image" => {
                        image_path = Some(flag_value(solve_args, i, "--image", program_name));
                        i += 2;
                    }
                    "--output-image" => {
                        output_image_path =
                            Some(flag_value(solve_args, i, "--output-image", program_name));
                        i += 2;
                    }
                    "--input-file" => {
                        input_file = Some(flag_value(solve_args, i, "--input-file", program_name));
                        i += 2;
                    }
                    "--output-file" => {
                        output_file =
                            Some(flag_value(solve_args, i, "--output-file", program_name));
                        i += 2;
                    }
                    "--size" => {
                        let raw = flag_value(solve_args, i, "--size", program_name);
                        match raw.parse::<usize>() {
                            Ok(s) => size_flag = Some(s),
                            Err(_) => {
                                eprintln!(
                                    "Error: Invalid --size value '{}'. Must be a positive integer.",
                                    raw
                                );
                                process::exit(1);
                            }
                        }
                        i += 2;
                    }
                    other => {
                        match other.parse::<usize>() {
                            Ok(s) => positional_size = Some(s),
                            Err(_) => {
                                eprintln!("Error: unknown argument '{}'.", other);
                                print_usage(program_name);
                                process::exit(1);
                            }
                        }
                        i += 1;
                    }
                }
            }

            if image_path.is_some() && (input_file.is_some() || output_file.is_some()) {
                eprintln!("Error: --image cannot be combined with --input-file/--output-file.");
                process::exit(1);
            }

            if output_image_path.is_some() && image_path.is_none() {
                eprintln!("Error: --output-image can only be used with --image.");
                process::exit(1);
            }

            if input_file.is_some() || output_file.is_some() {
                // ---- File-based solve ----
                let input_file = match input_file {
                    Some(p) => p,
                    None => {
                        eprintln!("Error: --input-file is required when using --output-file.");
                        process::exit(1);
                    }
                };
                let output_file = match output_file {
                    Some(p) => p,
                    None => {
                        eprintln!("Error: --output-file is required when using --input-file.");
                        process::exit(1);
                    }
                };
                let size = match size_flag {
                    Some(s) => s,
                    None => {
                        eprintln!("Error: --size <n> is required when solving from a file.");
                        process::exit(1);
                    }
                };
                validate_perfect_square(size);

                // 1. Read the file.
                let content = match fs::read_to_string(input_file) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error: cannot read input file '{}': {}", input_file, e);
                        process::exit(1);
                    }
                };

                // 2. Validate the format first.
                let board = match parse_grid(&content, size) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Error: invalid puzzle format in '{}': {}", input_file, e);
                        process::exit(1);
                    }
                };

                // 3. Solve and write the solution.
                match solve_board(&board, size) {
                    Some(solved) => {
                        if let Err(e) = fs::write(output_file, format_board(&solved, size)) {
                            eprintln!("Error: cannot write output file '{}': {}", output_file, e);
                            process::exit(1);
                        }
                        println!(
                            "Solved {}x{} puzzle: wrote solution to {}",
                            size, size, output_file
                        );
                    }
                    None => {
                        eprintln!("Error: the puzzle has no valid solution.");
                        process::exit(1);
                    }
                }
            } else if let Some(image_path) = image_path {
                // ---- Image-based solve ----
                let path = Path::new(image_path);

                // Verify file format is png or jpg/jpeg
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if ext != "png" && ext != "jpg" && ext != "jpeg" {
                    eprintln!("Error: Only PNG and JPG/JPEG formats are supported.");
                    process::exit(1);
                }

                if !path.exists() {
                    eprintln!("Error: File does not exist at {}", image_path);
                    process::exit(1);
                }

                if let Some(size) = size_flag {
                    validate_perfect_square(size);
                }

                println!("Processing Sudoku image: {} ...", image_path);

                let run_pipeline = || -> Result<(), Box<dyn std::error::Error>> {
                    let gray = vision::load_image(path)?;
                    let binary = imageproc::contrast::adaptive_threshold(&gray, 15, 10);

                    // Locate grid
                    let corners = vision::find_grid_corners(&binary)?;
                    let warped = vision::warp_grid(&gray, corners)?;

                    // Use the explicit --size if given, otherwise auto-detect.
                    let size = match size_flag {
                        Some(s) => s,
                        None => {
                            let detected = vision::detect_grid_size(&warped);
                            println!("Auto-detected grid size: {}x{}", detected, detected);
                            detected
                        }
                    };

                    // OCR board
                    println!("Running OCR character recognition...");
                    let recognized = vision::scan_board(&warped, size)?;

                    // Print recognized board
                    println!("\n--- RECOGNIZED BOARD ---");
                    print_flat_board(&recognized, size);

                    // Solve
                    println!("\nSolving board using DLX Algorithm X...");
                    if let Some(solved) = solve_board(&recognized, size) {
                        println!("\n--- SOLVED BOARD ---");
                        print_flat_board(&solved, size);

                        if let Some(out_img_path) = output_image_path {
                            let out_path = Path::new(out_img_path);
                            let out_ext = out_path
                                .extension()
                                .and_then(|s| s.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            if out_ext != "png" && out_ext != "jpg" && out_ext != "jpeg" {
                                return Err(
                                    "Only PNG and JPG/JPEG formats are supported for output image."
                                        .into(),
                                );
                            }

                            println!("Rendering solved puzzle to {} ...", out_img_path);
                            let solved_img =
                                vision::draw_solution(&warped, &recognized, &solved, size)?;
                            solved_img.save(out_path)?;
                            println!("Solved image saved successfully.");
                        }
                    } else {
                        eprintln!(
                            "\nError: The scanned puzzle has no valid solution. Please check the recognized grid above."
                        );
                    }
                    Ok(())
                };

                if let Err(e) = run_pipeline() {
                    eprintln!("Error solving image: {}", e);
                    process::exit(1);
                }
            } else {
                // ---- Solve from stdin (standard helper) ----
                let size = size_flag.or(positional_size).unwrap_or(9);
                validate_perfect_square(size);

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

        _ => {
            print_usage(program_name);
            process::exit(1);
        }
    }
}

fn print_flat_board(board: &[usize], size: usize) {
    for row in 0..size {
        for col in 0..size {
            let val = board[row * size + col];
            if val == 0 {
                print!(".");
            } else if (1..=9).contains(&val) {
                print!("{}", val);
            } else if (10..=16).contains(&val) {
                let c = (b'A' + (val - 10) as u8) as char;
                print!("{}", c);
            } else {
                print!("?");
            }
            if col < size - 1 {
                print!(" ");
            }
        }
        println!();
    }
}
