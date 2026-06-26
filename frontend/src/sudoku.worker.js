// Web Worker: runs the Rust/WASM solver & generator off the main thread so the
// UI never freezes — important for large boards (25x25 generation can take seconds).
import init, { generate_sudoku_wasm, solve_sudoku_wasm } from './pkg/sukodu.js';

// Initialize the WASM module once; every request awaits this promise.
const ready = init().then(
  () => self.postMessage({ type: 'ready' }),
  (err) => self.postMessage({ type: 'error', error: String(err) }),
);

self.onmessage = async (e) => {
  const { id, type, payload } = e.data;
  try {
    await ready;
    let result;
    if (type === 'generate') {
      // The Rust generator needs a u64 seed (wasm32 has no SystemTime). Supply
      // one from the JS host so each puzzle differs.
      const seed = (BigInt(Date.now()) << 20n) ^ BigInt(Math.floor(Math.random() * (1 << 20)));
      result = Array.from(generate_sudoku_wasm(payload.size, payload.difficulty, seed));
    } else if (type === 'solve') {
      result = Array.from(solve_sudoku_wasm(new Uint32Array(payload.board), payload.size));
    } else {
      throw new Error(`Unknown request type: ${type}`);
    }
    self.postMessage({ id, ok: true, result });
  } catch (err) {
    self.postMessage({ id, ok: false, error: err?.toString?.() ?? String(err) });
  }
};
