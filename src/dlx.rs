use crate::heap::Heap;

pub struct Cell {
    pub previous: usize,
    pub next: usize,
    pub line: usize,
    pub col: usize,
}

pub struct Line {
    pub state: i8, // 0 = active/undecided, 1 = selected, -1 = removed
    pub cells: Vec<usize>,
}

pub struct Column {
    pub size: i32,
    pub head: usize,
}

pub struct ExactCover {
    cells: Vec<Cell>,
    lines: Vec<Line>,
    cols: Vec<Column>,
    heap: Heap,
}

impl ExactCover {
    /// Creates a new ExactCover instance.
    /// - `num_cols`: the number of columns in the exact cover matrix
    /// - `lines_cols`: a vector of lines, where each line is represented by the list of column indices it covers.
    pub fn new(num_cols: usize, lines_cols: Vec<Vec<usize>>) -> Self {
        let mut cells = Vec::new();
        let mut cols = Vec::with_capacity(num_cols);

        // 1. Create a dummy head cell for each column
        for col_idx in 0..num_cols {
            cells.push(Cell {
                previous: col_idx,
                next: col_idx,
                line: usize::MAX,
                col: col_idx,
            });
            cols.push(Column {
                size: 0,
                head: col_idx,
            });
        }

        let mut lines = Vec::with_capacity(lines_cols.len());

        // 2. Create cells for each line and link them to their columns
        for (line_idx, selected_cols) in lines_cols.into_iter().enumerate() {
            let mut line_cell_indices = Vec::new();
            for col_idx in selected_cols {
                let cell_idx = cells.len();
                cells.push(Cell {
                    previous: cell_idx,
                    next: cell_idx,
                    line: line_idx,
                    col: col_idx,
                });
                line_cell_indices.push(cell_idx);

                // Append the cell to the column's circular doubly linked list.
                let head_idx = cols[col_idx].head;
                let next_idx = cells[head_idx].next;

                cells[cell_idx].previous = head_idx;
                cells[cell_idx].next = next_idx;
                cells[next_idx].previous = cell_idx;
                cells[head_idx].next = cell_idx;

                cols[col_idx].size += 1;
            }
            lines.push(Line {
                state: 0,
                cells: line_cell_indices,
            });
        }

        // 3. Populate and build the heap
        let mut heap = Heap::new(num_cols);
        for (col_idx, col) in cols.iter().enumerate().take(num_cols) {
            heap.append(col_idx, col.size);
        }
        heap.heapify();

        ExactCover {
            cells,
            lines,
            cols,
            heap,
        }
    }

    /// Returns whether the line at `line_idx` is selected in the solution.
    pub fn is_selected(&self, line_idx: usize) -> bool {
        self.lines[line_idx].state == 1
    }

    /// Selects the line at `line_idx` as part of the solution (used to set up pre-filled cells).
    pub fn select(&mut self, line_idx: usize) {
        self.line_select(line_idx);
    }

    /// Selects a line and removes the corresponding columns/cells from the matrix.
    fn line_select(&mut self, line_idx: usize) -> Vec<Vec<usize>> {
        let mut stacks = Vec::new();
        if self.lines[line_idx].state == 0 {
            self.lines[line_idx].state = 1;
            let cell_indices = self.lines[line_idx].cells.clone();
            for cell_idx in cell_indices {
                let col_idx = self.cells[cell_idx].col;
                stacks.push(self.column_remove(col_idx));
            }
        }
        stacks
    }

    /// Unselects a line and restores the corresponding columns/cells back to the matrix.
    fn line_unselect(&mut self, line_idx: usize, stacks: &[Vec<usize>]) {
        if self.lines[line_idx].state == 1 {
            self.lines[line_idx].state = 0;
            let cell_indices = self.lines[line_idx].cells.clone();
            for i in (0..stacks.len()).rev() {
                let cell_idx = cell_indices[i];
                let col_idx = self.cells[cell_idx].col;
                self.column_restore(col_idx, &stacks[i]);
            }
        }
    }

    /// Removes a column from the cover matrix.
    fn column_remove(&mut self, col_idx: usize) -> Vec<usize> {
        self.heap.remove(col_idx);
        let mut stack = Vec::new();
        let head_idx = self.cols[col_idx].head;
        let mut cell_idx = self.cells[head_idx].next;
        while cell_idx != head_idx {
            let line_idx = self.cells[cell_idx].line;
            if self.lines[line_idx].state == 0 {
                self.line_remove(line_idx);
                stack.push(line_idx);
            }
            cell_idx = self.cells[cell_idx].next;
        }
        stack
    }

    /// Restores a column back to the cover matrix.
    fn column_restore(&mut self, col_idx: usize, stack: &[usize]) {
        for &line_idx in stack.iter().rev() {
            self.line_restore(line_idx);
        }
        self.heap.restore(col_idx);
    }

    /// Removes a line from the cover matrix, which removes its cells from their respective columns.
    fn line_remove(&mut self, line_idx: usize) {
        if self.lines[line_idx].state == 0 {
            self.lines[line_idx].state = -1;
            let cell_indices = self.lines[line_idx].cells.clone();
            for cell_idx in cell_indices {
                let prev_idx = self.cells[cell_idx].previous;
                let next_idx = self.cells[cell_idx].next;
                self.cells[prev_idx].next = next_idx;
                self.cells[next_idx].previous = prev_idx;

                let col_idx = self.cells[cell_idx].col;
                self.cols[col_idx].size -= 1;
                self.heap.decrease(col_idx, self.cols[col_idx].size);
            }
        }
    }

    /// Restores a line back to the cover matrix, re-inserting its cells into their respective columns.
    fn line_restore(&mut self, line_idx: usize) {
        if self.lines[line_idx].state == -1 {
            self.lines[line_idx].state = 0;
            let cell_indices = self.lines[line_idx].cells.clone();
            for cell_idx in cell_indices.into_iter().rev() {
                let prev_idx = self.cells[cell_idx].previous;
                let next_idx = self.cells[cell_idx].next;
                self.cells[prev_idx].next = cell_idx;
                self.cells[next_idx].previous = cell_idx;

                let col_idx = self.cells[cell_idx].col;
                self.cols[col_idx].size += 1;
                self.heap.increase(col_idx, self.cols[col_idx].size);
            }
        }
    }

    /// Solves the exact cover problem using Algorithm X (Dancing Links).
    /// Returns true if a solution is found, false otherwise.
    pub fn solve(&mut self) -> bool {
        if self.heap.is_empty() {
            return true;
        }
        let col_idx = self.heap.take_min();
        let n = self.cols[col_idx].size;
        if n == 0 {
            self.heap.restore(col_idx);
            return false;
        }
        let head_idx = self.cols[col_idx].head;
        let mut cell_idx = self.cells[head_idx].next;
        while cell_idx != head_idx {
            let line_idx = self.cells[cell_idx].line;
            let stacks = self.line_select(line_idx);
            if self.solve() {
                return true;
            }
            self.line_unselect(line_idx, &stacks);
            cell_idx = self.cells[cell_idx].next;
        }
        self.heap.restore(col_idx);
        false
    }

    /// Counts the number of solutions to the exact cover problem up to `limit`.
    pub fn count_solutions(&mut self, limit: usize) -> usize {
        let mut count = 0;
        self.solve_count(&mut count, limit);
        count
    }

    fn solve_count(&mut self, count: &mut usize, limit: usize) {
        if *count >= limit {
            return;
        }
        if self.heap.is_empty() {
            *count += 1;
            return;
        }
        let col_idx = self.heap.take_min();
        let n = self.cols[col_idx].size;
        if n == 0 {
            self.heap.restore(col_idx);
            return;
        }
        let head_idx = self.cols[col_idx].head;
        let mut cell_idx = self.cells[head_idx].next;
        while cell_idx != head_idx {
            let line_idx = self.cells[cell_idx].line;
            let stacks = self.line_select(line_idx);
            self.solve_count(count, limit);
            self.line_unselect(line_idx, &stacks);
            if *count >= limit {
                self.heap.restore(col_idx);
                return;
            }
            cell_idx = self.cells[cell_idx].next;
        }
        self.heap.restore(col_idx);
    }

    /// Counts the number of solutions up to `limit`, aborting if `max_steps` operations are performed.
    /// Returns `(solutions_count, completed)`.
    pub fn count_solutions_limited(&mut self, limit: usize, max_steps: usize) -> (usize, bool) {
        let mut count = 0;
        let mut steps = 0;
        let completed = self.solve_count_limited(&mut count, limit, &mut steps, max_steps);
        (count, completed)
    }

    fn solve_count_limited(
        &mut self,
        count: &mut usize,
        limit: usize,
        steps: &mut usize,
        max_steps: usize,
    ) -> bool {
        *steps += 1;
        if *steps > max_steps {
            return false;
        }
        if *count >= limit {
            return true;
        }
        if self.heap.is_empty() {
            *count += 1;
            return true;
        }
        let col_idx = self.heap.take_min();
        let n = self.cols[col_idx].size;
        if n == 0 {
            self.heap.restore(col_idx);
            return true;
        }
        let head_idx = self.cols[col_idx].head;
        let mut cell_idx = self.cells[head_idx].next;
        while cell_idx != head_idx {
            let line_idx = self.cells[cell_idx].line;
            let stacks = self.line_select(line_idx);
            let ok = self.solve_count_limited(count, limit, steps, max_steps);
            self.line_unselect(line_idx, &stacks);
            if !ok {
                self.heap.restore(col_idx);
                return false;
            }
            if *count >= limit {
                self.heap.restore(col_idx);
                return true;
            }
            cell_idx = self.cells[cell_idx].next;
        }
        self.heap.restore(col_idx);
        true
    }
}
