---
id: task-3
title: "Step-by-Step Logical Solver & Explainer"
status: "To Do"
assignee: []
created_date: "2026-06-22"
updated_date: "2026-06-22"
labels:
  - "algorithm"
  - "cli"
dependencies: []
priority: "medium"
---

## Description
Develop a logic-based Sudoku solver that solves puzzles using human-like techniques (e.g., Naked/Hidden Singles, Pointing Pairs, Box-Line Reduction, X-Wing) instead of relying solely on the DLX backtracking search. Provide a CLI option to output step-by-step logical explanations for solving.

## Acceptance Criteria
- [ ] Implement a logical solver module with a hierarchy of strategies (Naked Singles, Hidden Singles, Pointing Pairs, etc.)
- [ ] Generate structured explanations for each step/deduction
- [ ] Allow falling back to Dancing Links if human strategies cannot progress further
- [ ] Rate puzzle difficulty precisely based on the most advanced technique required to solve it
- [ ] Add a CLI subcommand `sukodu explain < puzzle.txt` to print human-readable explanations

## Definition of Done
- [ ] Code compiles without warnings
- [ ] Unit and integration tests pass
- [ ] Clippy and cargo fmt checks pass
- [ ] Documentation updated
