---
name: document-feature-changes
description: Guidelines for ensuring new features, CLI flags, or system changes are properly documented in the project README.md and other documentation files.
---

# Document Feature Changes

This skill ensures that whenever new features, CLI commands, subcommands, flags, or configuration options are introduced or modified in the Sudoku Solver & Generator (`sukodu`), they are comprehensively and accurately documented.

## Guidelines

1. **Identify Documentation Impact**:
   - Check if the task adds or modifies any CLI subcommands (e.g., `generate`, `solve`).
   - Check if any new CLI flags (e.g., `--output-image`, `--input-file`, `--size`) have been added or their behavior modified.
   - Check if prerequisites (like system packages or libraries) have changed.

2. **Update README.md**:
   - Locate the relevant sections in [README.md](file:///Users/ramdanikurnia/Desktop/Backup/Programming/personal-project/sudoku-solver/sudoku/README.md) (e.g., "Usage", "Generate", "Solve from stdin", "Solve from an image", "Solve from a file").
   - Add clear shell command examples showing how to use the new feature or flag.
   - Provide a brief description of the input format, output format, limitations, and supported options.
   - Mention any new dependencies or system requirements under the "Prerequisite" or setup sections if applicable.

3. **Verify Documentation Accuracy**:
   - Verify that the CLI usage examples match the exact syntax and options defined in [src/bin/main.rs](file:///Users/ramdanikurnia/Desktop/Backup/Programming/personal-project/sudoku-solver/sudoku/src/bin/main.rs).
   - Ensure the description is precise and clearly explains what the new capability does.
