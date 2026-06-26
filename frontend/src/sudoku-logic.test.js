import { describe, it, expect } from 'vitest';
import {
  charToValue,
  formatCellValue,
  findConflicts,
  parsePuzzleText,
} from './sudoku-logic.js';

// A valid, conflict-free 4x4 solution used as a baseline.
const SOLVED_4 = [
  1, 2, 3, 4,
  3, 4, 1, 2,
  2, 1, 4, 3,
  4, 3, 2, 1,
];

const sorted = (set) => [...set].sort((a, b) => a - b);

describe('charToValue', () => {
  it('treats blanks as 0', () => {
    expect(charToValue('', 9)).toBe(0);
    expect(charToValue('0', 9)).toBe(0);
    expect(charToValue('.', 9)).toBe(0);
  });

  it('maps digits within range', () => {
    expect(charToValue('1', 9)).toBe(1);
    expect(charToValue('9', 9)).toBe(9);
  });

  it('rejects digits above the board size', () => {
    expect(charToValue('5', 4)).toBeNull();
  });

  it('maps letters (case-insensitive) starting at A=10', () => {
    expect(charToValue('A', 16)).toBe(10);
    expect(charToValue('g', 16)).toBe(16);
    expect(charToValue('P', 25)).toBe(25);
  });

  it('rejects letters above the board size', () => {
    expect(charToValue('A', 9)).toBeNull(); // 10 > 9
    expect(charToValue('H', 16)).toBeNull(); // 17 > 16
  });

  it('rejects junk characters', () => {
    expect(charToValue('!', 9)).toBeNull();
    expect(charToValue('-', 9)).toBeNull();
  });
});

describe('formatCellValue', () => {
  it('renders blanks and digits', () => {
    expect(formatCellValue(0)).toBe('');
    expect(formatCellValue(7)).toBe('7');
  });

  it('renders values above 9 as letters', () => {
    expect(formatCellValue(10)).toBe('A');
    expect(formatCellValue(16)).toBe('G');
    expect(formatCellValue(25)).toBe('P');
  });

  it('round-trips with charToValue', () => {
    for (let v = 1; v <= 25; v++) {
      expect(charToValue(formatCellValue(v), 25)).toBe(v);
    }
  });

  it('renders out-of-range as ?', () => {
    expect(formatCellValue(99)).toBe('?');
  });
});

describe('findConflicts', () => {
  it('reports no conflicts for a valid solution', () => {
    expect(findConflicts(SOLVED_4, 4).size).toBe(0);
  });

  it('reports no conflicts for an empty board', () => {
    expect(findConflicts(Array(16).fill(0), 4).size).toBe(0);
  });

  it('detects a duplicate in a row', () => {
    const board = Array(16).fill(0);
    board[0] = 1;
    board[1] = 1;
    expect(sorted(findConflicts(board, 4))).toEqual([0, 1]);
  });

  it('detects a duplicate in a column', () => {
    const board = Array(16).fill(0);
    board[0] = 1;
    board[4] = 1; // same column, next row
    expect(sorted(findConflicts(board, 4))).toEqual([0, 4]);
  });

  it('detects a duplicate within a 2x2 block', () => {
    const board = Array(16).fill(0);
    board[0] = 2; // block 0: indices 0,1,4,5
    board[5] = 2;
    expect(sorted(findConflicts(board, 4))).toEqual([0, 5]);
  });

  it('ignores blanks (0) when checking', () => {
    const board = Array(16).fill(0);
    expect(findConflicts(board, 4).size).toBe(0);
  });
});

describe('parsePuzzleText', () => {
  it('parses a 4x4 board of blanks and detects its size', () => {
    const { values, size, error } = parsePuzzleText(Array(16).fill('0').join(' '));
    expect(error).toBeUndefined();
    expect(size).toBe(4);
    expect(values).toHaveLength(16);
    expect(values.every((v) => v === 0)).toBe(true);
  });

  it('parses a full 9x9 board', () => {
    const { values, size } = parsePuzzleText(Array(81).fill('0').join('\n'));
    expect(size).toBe(9);
    expect(values).toHaveLength(81);
  });

  it('accepts letters and . blanks for large boards', () => {
    const tokens = Array(256).fill('0');
    tokens[0] = 'A'; // 10
    tokens[1] = 'g'; // 16, lowercase
    tokens[2] = '.'; // blank
    const { values, size, error } = parsePuzzleText(tokens.join(' '));
    expect(error).toBeUndefined();
    expect(size).toBe(16);
    expect(values[0]).toBe(10);
    expect(values[1]).toBe(16);
    expect(values[2]).toBe(0);
  });

  it('rejects empty input', () => {
    expect(parsePuzzleText('   ').error).toBe('no values found');
  });

  it('rejects a non-square count', () => {
    expect(parsePuzzleText('1 2 3 4 5').error).toMatch(/square grid/);
  });

  it('rejects an unsupported (but square) size', () => {
    // 36 tokens -> 6x6, which is square but not in the supported set.
    expect(parsePuzzleText(Array(36).fill('0').join(' ')).error).toMatch(/aren't supported/);
  });

  it('rejects an out-of-range value for the detected size', () => {
    const tokens = Array(16).fill('0');
    tokens[3] = '9'; // 9 > 4 on a 4x4 board
    expect(parsePuzzleText(tokens.join(' ')).error).toMatch(/invalid value "9"/);
  });
});
