# SUKODU — Web Interface

An interactive Sudoku player, generator, and solver that runs the Rust core
([Dancing Links / Algorithm X](../README.md)) entirely in the browser via
WebAssembly. Built with Vite + React 19.

The solver/generator run in a **Web Worker**, so even large boards (16×16, 25×25)
never freeze the UI while computing.

## Prerequisites

- **Node.js** 18+ and a package manager (`npm`, `pnpm`, or `bun`).
- **Rust** toolchain with the WASM target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- **wasm-pack** to compile the crate to WASM + JS bindings:
  ```bash
  cargo install wasm-pack
  ```

## Build the WASM module

The compiled WASM lives in `src/pkg/` and is **git-ignored** — you must generate
it before the app can run. From this `frontend/` directory:

```bash
npm run build:wasm
```

This runs, from the crate root:

```bash
wasm-pack build --target web --out-dir frontend/src/pkg
```

and produces `src/pkg/sukodu.js`, `src/pkg/sukodu_bg.wasm`, and the type
definitions that `src/sudoku.worker.js` imports. Re-run it whenever you change
the Rust solver, generator, or the `*_wasm` bindings in `src/lib.rs`.

> Only the library is compiled to WASM. The image/OCR pipeline (`vision.rs`) is
> excluded from the `wasm32` target, so Tesseract is **not** required to build
> the web app.

## Develop

```bash
npm install
npm run build:wasm   # once (and after any Rust change)
npm run dev          # Vite dev server with HMR
```

## Production build

```bash
npm run build        # runs build:wasm, then vite build
npm run preview      # serve the production bundle locally
```

`npm run build` always rebuilds the WASM first so a fresh checkout produces a
working bundle.

## Using the app

- **Board size / difficulty** — switch between 4×4, 9×9, 16×16, 25×25 and
  easy / medium / hard. A new puzzle generates automatically.
- **Entering values** — click a cell and type (`1`–`9`, and `A`–`P` for boards
  larger than 9×9), or use the on-screen number palette (works on touch
  devices). Arrow keys move between cells; `Backspace`/`Delete` clears.
- **Solve Instantly** — solves the current board with the DLX solver.
- **Hint** — reveals one correct cell.
- **Reset to Clues / Clear Grid** — restore the generated clues, or empty the
  board to enter your own puzzle.
- **Import from Text** — paste a whitespace-separated puzzle (the same format
  the CLI accepts: numbers, `0` for blanks). The grid size is auto-detected
  from the value count.

## Notes

- Fonts resolve from the system stack (no external CDN request). See the comment
  in `src/index.css` to self-host the exact typefaces via `@fontsource`.
