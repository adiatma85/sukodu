// Pure, framework-free Sudoku helpers. Kept out of App.jsx so they can be
// unit-tested without a DOM or the WASM worker.

export const SIZES = [4, 9, 16, 25];

// Map a single character to a Sudoku value (0 = empty). 'A' -> 10, 'B' -> 11, ...
// Returns null for characters that aren't valid for the given board size.
export function charToValue(ch, size) {
  if (ch === '' || ch === '0' || ch === '.') return 0;
  if (ch >= '1' && ch <= '9') {
    const n = parseInt(ch, 10);
    return n <= size ? n : null;
  }
  const upper = ch.toUpperCase();
  if (upper >= 'A' && upper <= 'Z') {
    const v = upper.charCodeAt(0) - 65 + 10; // 'A' -> 10
    return v <= size ? v : null;
  }
  return null;
}

// 0 -> '', 5 -> '5', 10 -> 'A'
export function formatCellValue(val) {
  if (val === 0) return '';
  if (val >= 1 && val <= 9) return val.toString();
  if (val >= 10 && val <= 35) return String.fromCharCode(65 + (val - 10)); // 'A' -> 65
  return '?';
}

// Return the set of indices that violate row/column/block uniqueness.
export function findConflicts(board, size) {
  const sq = Math.floor(Math.sqrt(size));
  const conflicts = new Set();

  const checkList = (indices) => {
    const seen = new Map();
    for (const idx of indices) {
      const val = board[idx];
      if (val > 0) {
        if (seen.has(val)) {
          conflicts.add(idx);
          conflicts.add(seen.get(val));
        } else {
          seen.set(val, idx);
        }
      }
    }
  };

  // Rows
  for (let r = 0; r < size; r++) {
    const indices = [];
    for (let c = 0; c < size; c++) indices.push(r * size + c);
    checkList(indices);
  }

  // Columns
  for (let c = 0; c < size; c++) {
    const indices = [];
    for (let r = 0; r < size; r++) indices.push(r * size + c);
    checkList(indices);
  }

  // Blocks
  for (let b = 0; b < size; b++) {
    const indices = [];
    const blockRow = Math.floor(b / sq) * sq;
    const blockCol = (b % sq) * sq;
    for (let dr = 0; dr < sq; dr++) {
      for (let dc = 0; dc < sq; dc++) {
        indices.push((blockRow + dr) * size + (blockCol + dc));
      }
    }
    checkList(indices);
  }

  return conflicts;
}

// Parse a whitespace-separated puzzle (the same format the CLI accepts: numbers
// with 0 for blanks; single letters A-P are also accepted for large boards).
// Returns { values, size } on success or { error } with a human-readable reason.
export function parsePuzzleText(text, allowedSizes = SIZES) {
  const tokens = text.trim().split(/\s+/).filter(Boolean);
  if (tokens.length === 0) {
    return { error: 'no values found' };
  }

  const count = tokens.length;
  const size = Math.round(Math.sqrt(count));
  if (size * size !== count) {
    return { error: `${count} values do not form a square grid` };
  }
  if (!allowedSizes.includes(size)) {
    return { error: `${size}x${size} boards aren't supported (use 4, 9, 16, 25)` };
  }

  const values = [];
  for (const tok of tokens) {
    let v;
    if (/^\d+$/.test(tok)) v = parseInt(tok, 10);
    else if (tok.length === 1) v = charToValue(tok, size);
    else v = null;

    if (v === null || v === undefined || v < 0 || v > size) {
      return { error: `invalid value "${tok}" for a ${size}x${size} board` };
    }
    values.push(v);
  }

  return { values, size };
}
