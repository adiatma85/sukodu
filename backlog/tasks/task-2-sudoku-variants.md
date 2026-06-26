---
id: task-2
title: "Support for Sudoku Variants (e.g., Sudoku-X, Hyper-Sudoku)"
status: "To Do"
assignee: []
created_date: "2026-06-22"
updated_date: "2026-06-22"
labels:
  - "algorithm"
  - "solver"
dependencies: []
priority: "medium"
---

## Description
Extend the Exact Cover (`ExactCover`) solver and board generator to support common Sudoku variations such as Diagonal Sudoku (Sudoku-X) and Hyper-Sudoku (Windoku) by adding extra constraint columns/cells to the cover matrix.

## Acceptance Criteria
- [ ] Add support in `make_lines_cols` for diagonal constraints (Sudoku-X)
- [ ] Add support in `make_lines_cols` for the four extra 3x3 window regions (Hyper-Sudoku)
- [ ] Update `generate_board` and `solve_board` to accept a variant parameter
- [ ] Add unit tests verifying solutions and uniqueness under different variant constraints
- [ ] Update the CLI (`main.rs`) to expose a `--variant <type>` parameter for generate and solve subcommands

## Definition of Done
- [ ] Code compiles without warnings
- [ ] Unit and integration tests pass
- [ ] Clippy and cargo fmt checks pass
- [ ] Documentation updated
