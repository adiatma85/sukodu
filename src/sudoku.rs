use crate::dlx::ExactCover;
use std::io::{self, BufRead};

/// Generates the list of column indices covered by each line.
/// There are `size * size * size` lines and `size * size * 4` columns.
pub fn make_lines_cols(size: usize) -> Vec<Vec<usize>> {
    let sqrt = (size as f64).sqrt().floor() as usize;
    let mut lines_cols = Vec::with_capacity(size * size * size);
    for i in 0..size {
        for j in 0..size {
            let block = (i / sqrt) * sqrt + (j / sqrt);
            for k in 0..size {
                let mut cols = vec![0; 4];
                cols[0] = i * size + j;
                cols[1] = i * size + k + size * size;
                cols[2] = j * size + k + size * size * 2;
                cols[3] = block * size + k + size * size * 3;
                lines_cols.push(cols);
            }
        }
    }
    lines_cols
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        let mut rng = SimpleRng { state: seed };
        if rng.state == 0 {
            rng.state = 1;
        }
        rng
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        let range = (max - min) as u64;
        min + (self.next() % range) as usize
    }
}

fn shuffle<T>(slice: &mut [T], rng: &mut SimpleRng) {
    let n = slice.len();
    for i in (1..n).rev() {
        let j = rng.gen_range(0, i + 1);
        slice.swap(i, j);
    }
}

/// Generates a new Sudoku puzzle of the specified size and difficulty.
pub fn generate(size: usize, difficulty: &str) {
    let total_cells = size * size;
    let target_clues = match difficulty {
        "easy" => (total_cells * 45) / 100,
        "medium" => (total_cells * 35) / 100,
        "hard" => (total_cells * 23) / 100,
        _ => (total_cells * 35) / 100,
    };

    let seed = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as u64,
        Err(_) => 42,
    };
    let mut rng = SimpleRng::new(seed);

    // 1. Generate a solved board by randomizing the first row
    let mut first_row: Vec<usize> = (1..=size).collect();
    shuffle(&mut first_row, &mut rng);

    let num_cols = size * size * 4;
    let lines_cols = make_lines_cols(size);
    let mut problem = ExactCover::new(num_cols, lines_cols.clone());

    for j in 0..size {
        let val = first_row[j];
        let line_idx = j * size + val - 1;
        problem.select(line_idx);
    }

    if !problem.solve() {
        eprintln!("Failed to generate base board");
        return;
    }

    let mut solved_board = vec![0; total_cells];
    for i in 0..size {
        for j in 0..size {
            for k in 0..size {
                let line_idx = (i * size + j) * size + k;
                if problem.is_selected(line_idx) {
                    solved_board[i * size + j] = k + 1;
                    break;
                }
            }
        }
    }

    // 2. Remove clues while maintaining unique solution
    let mut puzzle = solved_board.clone();
    let mut cell_indices: Vec<usize> = (0..total_cells).collect();
    shuffle(&mut cell_indices, &mut rng);

    let mut current_clues = total_cells;
    for &idx in &cell_indices {
        if current_clues <= target_clues {
            break;
        }

        let temp = puzzle[idx];
        puzzle[idx] = 0;

        if has_unique_solution(&puzzle, size, &lines_cols) {
            current_clues -= 1;
        } else {
            puzzle[idx] = temp;
        }
    }

    // Print puzzle to stdout
    for i in 0..size {
        for j in 0..size {
            print!("{}", puzzle[i * size + j]);
            if j < size - 1 {
                print!(" ");
            }
        }
        println!();
    }
}

fn has_unique_solution(board: &[usize], size: usize, lines_cols: &Vec<Vec<usize>>) -> bool {
    let num_cols = size * size * 4;
    let mut problem = ExactCover::new(num_cols, lines_cols.clone());
    for (idx, &val) in board.iter().enumerate() {
        if val > 0 {
            let line_idx = idx * size + val - 1;
            problem.select(line_idx);
        }
    }
    problem.count_solutions(2) == 1
}

/// Parses the Sudoku puzzle from standard input and selects the pre-filled cells.
pub fn parse(problem: &mut ExactCover, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = String::new();
    let mut count = 0;
    let target = size * size;

    while count < target {
        buffer.clear();
        let bytes_read = handle.read_line(&mut buffer)?;
        if bytes_read == 0 {
            break; // EOF reached
        }
        for token in buffer.split_whitespace() {
            if count >= target {
                break;
            }
            let k: usize = token.parse()?;
            if k >= 1 && k <= size {
                let line_idx = count * size + k - 1;
                problem.select(line_idx);
            }
            count += 1;
        }
    }
    Ok(())
}

/// Prints the solved Sudoku puzzle to standard output.
pub fn print(problem: &ExactCover, size: usize) {
    for i in 0..size {
        for j in 0..size {
            for k in 0..size {
                let line_idx = (i * size + j) * size + k;
                if problem.is_selected(line_idx) {
                    print!("{}", k + 1);
                    break;
                }
            }
            if j < size - 1 {
                print!(" ");
            }
        }
        println!();
    }
}
