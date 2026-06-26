import { useState, useEffect, useRef } from 'react';
import {
  SIZES,
  charToValue,
  formatCellValue,
  findConflicts,
  parsePuzzleText,
} from './sudoku-logic.js';
import './App.css';

const DIFFICULTIES = ['easy', 'medium', 'hard'];

// Pick a random element. Kept at module scope (outside the component) so it's
// not treated as impure render logic.
function pickRandom(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function App() {
  const [wasmLoaded, setWasmLoaded] = useState(false);
  const [isBusy, setIsBusy] = useState(false);
  const [size, setSize] = useState(9);
  const [difficulty, setDifficulty] = useState('medium');

  // Board states (flat arrays of length size * size)
  const [board, setBoard] = useState([]);
  const [initialBoard, setInitialBoard] = useState([]);

  const [selectedCell, setSelectedCell] = useState(null);
  const [highlightedValue, setHighlightedValue] = useState(0);
  const [conflicts, setConflicts] = useState(new Set());
  const [hintIndices, setHintIndices] = useState(new Set());

  const [statusMsg, setStatusMsg] = useState('Initializing WASM engine...');
  const [isError, setIsError] = useState(false);
  const [isSolvedState, setIsSolvedState] = useState(false);
  const [waveIndices, setWaveIndices] = useState(new Set());

  const [importText, setImportText] = useState('');
  const [showImport, setShowImport] = useState(false);

  const workerRef = useRef(null);
  const pendingRef = useRef(new Map());
  const reqIdRef = useRef(0);
  const pendingImportRef = useRef(null);

  // Compute block size
  const sqrt = Math.floor(Math.sqrt(size));

  // Spin up the WASM worker once. All solve/generate work runs there so the
  // main thread (and the UI) stays responsive.
  useEffect(() => {
    const worker = new Worker(new URL('./sudoku.worker.js', import.meta.url), {
      type: 'module',
    });
    workerRef.current = worker;
    const pending = pendingRef.current;

    worker.onmessage = (e) => {
      const { id, type, ok, result, error } = e.data;
      if (type === 'ready') {
        setWasmLoaded(true);
        setStatusMsg('WASM engine loaded successfully.');
        return;
      }
      if (type === 'error') {
        setIsError(true);
        setStatusMsg('Failed to load WASM engine: ' + error);
        return;
      }
      const pending = pendingRef.current.get(id);
      if (!pending) return;
      pendingRef.current.delete(id);
      if (ok) pending.resolve(result);
      else pending.reject(new Error(error));
    };

    worker.onerror = (err) => {
      setIsError(true);
      setStatusMsg('Worker error: ' + (err.message || 'unknown'));
    };

    return () => {
      worker.terminate();
      pending.clear();
    };
  }, []);

  // Send a request to the worker and resolve when it replies.
  const callWorker = (type, payload) =>
    new Promise((resolve, reject) => {
      const worker = workerRef.current;
      if (!worker) {
        reject(new Error('Worker not initialized'));
        return;
      }
      const id = ++reqIdRef.current;
      pendingRef.current.set(id, { resolve, reject });
      worker.postMessage({ id, type, payload });
    });

  const generateNewPuzzle = async (currentSize, currentDiff) => {
    setStatusMsg('Generating puzzle...');
    setIsError(false);
    setIsSolvedState(false);
    setWaveIndices(new Set());
    setHintIndices(new Set());
    setIsBusy(true);

    try {
      const result = await callWorker('generate', {
        size: currentSize,
        difficulty: currentDiff,
      });
      setBoard(result);
      setInitialBoard(result);
      setConflicts(new Set());
      setStatusMsg(`Generated ${currentSize}x${currentSize} ${currentDiff} puzzle.`);
    } catch (e) {
      setIsError(true);
      setStatusMsg('Generation failed: ' + e.message);
    } finally {
      setIsBusy(false);
    }
  };

  const solvePuzzle = async () => {
    if (!wasmLoaded || board.length === 0) return;
    setStatusMsg('Solving via Dancing Links + Algorithm X...');
    setIsError(false);
    setIsBusy(true);

    try {
      const solved = await callWorker('solve', { board, size });
      setBoard(solved);
      setConflicts(new Set());
      setHintIndices(new Set());
      setIsSolvedState(true);
      setStatusMsg('Sudoku solved successfully!');
      triggerSolveWave(size);
    } catch (e) {
      setIsError(true);
      setStatusMsg(e.message);
    } finally {
      setIsBusy(false);
    }
  };

  // Reveal one correct value in a random empty cell, computed from the solver.
  const giveHint = async () => {
    if (!wasmLoaded || board.length === 0 || isBusy) return;

    const empties = [];
    for (let i = 0; i < board.length; i++) {
      if (board[i] === 0) empties.push(i);
    }
    if (empties.length === 0) {
      setStatusMsg('No empty cells left to hint.');
      return;
    }

    setStatusMsg('Computing hint...');
    setIsError(false);
    setIsBusy(true);
    try {
      const solved = await callWorker('solve', { board, size });
      const idx = pickRandom(empties);
      const next = [...board];
      next[idx] = solved[idx];
      setBoard(next);
      setHintIndices((prev) => new Set(prev).add(idx));
      runConflictCheck(next, size);
      setHighlightedValue(solved[idx]);
      setStatusMsg('Revealed one cell.');
    } catch {
      setIsError(true);
      setStatusMsg('No hint available — current entries make the puzzle unsolvable.');
    } finally {
      setIsBusy(false);
    }
  };

  // Radial solve wave animation starting from top-left
  const triggerSolveWave = (currentSize) => {
    const maxDist = currentSize * 2;
    setWaveIndices(new Set());

    for (let dist = 0; dist <= maxDist; dist++) {
      const indicesAtDist = [];
      for (let r = 0; r < currentSize; r++) {
        for (let c = 0; c < currentSize; c++) {
          if (r + c === dist) {
            indicesAtDist.push(r * currentSize + c);
          }
        }
      }

      setTimeout(() => {
        setWaveIndices((prev) => {
          const next = new Set(prev);
          indicesAtDist.forEach((idx) => next.add(idx));
          return next;
        });
      }, dist * 35);
    }
  };

  // Check conflicts in real-time
  const runConflictCheck = (currentBoard, sizeArg = size) => {
    setConflicts(findConflicts(currentBoard, sizeArg));
  };

  // Update specific cell
  const updateCell = (row, col, value) => {
    const idx = row * size + col;
    if (initialBoard[idx] !== 0) return; // Clue protection

    const nextBoard = [...board];
    nextBoard[idx] = value;
    setBoard(nextBoard);

    // A manually edited cell is no longer a "hint" cell.
    if (hintIndices.has(idx)) {
      setHintIndices((prev) => {
        const next = new Set(prev);
        next.delete(idx);
        return next;
      });
    }

    runConflictCheck(nextBoard, size);

    if (value > 0) {
      setHighlightedValue(value);
    }
  };

  // Enter a value into the currently selected cell (used by the number palette).
  const enterValue = (value) => {
    if (!selectedCell) {
      setStatusMsg('Select a cell first, then tap a number.');
      return;
    }
    updateCell(selectedCell.row, selectedCell.col, value);
    const el = document.getElementById(`cell-${selectedCell.row}-${selectedCell.col}`);
    if (el) el.focus();
  };

  // Typed input (keyboard + mobile soft keyboards) flows through onChange.
  // We read the most recently typed character so a single-cell input works
  // regardless of caret position or whether the cell was already filled.
  const handleChange = (row, col, e) => {
    const raw = e.target.value;
    const ch = raw.slice(-1);
    const val = charToValue(ch, size);
    if (val === null) return; // ignore characters outside this board's alphabet
    updateCell(row, col, val);
  };

  // Arrow-key navigation + explicit delete. Value entry is handled by onChange.
  const handleKeyDown = (row, col, e) => {
    if (e.key === 'Backspace' || e.key === 'Delete') {
      updateCell(row, col, 0);
      return;
    }

    let nextRow = row;
    let nextCol = col;
    if (e.key === 'ArrowUp') nextRow = (row - 1 + size) % size;
    else if (e.key === 'ArrowDown') nextRow = (row + 1) % size;
    else if (e.key === 'ArrowLeft') nextCol = (col - 1 + size) % size;
    else if (e.key === 'ArrowRight') nextCol = (col + 1) % size;
    else return;

    e.preventDefault();
    const el = document.getElementById(`cell-${nextRow}-${nextCol}`);
    if (el) el.focus();
  };

  const getCellSizeStyle = () => {
    if (size === 4) return { width: '70px', height: '70px', fontSize: '2rem' };
    if (size === 9) return { width: '54px', height: '54px', fontSize: '1.5rem' };
    if (size === 16) return { width: '36px', height: '36px', fontSize: '1rem' };
    return { width: '25px', height: '25px', fontSize: '0.75rem' }; // 25x25
  };

  const resetBoard = () => {
    setBoard([...initialBoard]);
    setConflicts(new Set());
    setWaveIndices(new Set());
    setHintIndices(new Set());
    setIsSolvedState(false);
    setStatusMsg('Reset board to clues.');
    setIsError(false);
  };

  const clearBoard = () => {
    const empty = Array(size * size).fill(0);
    setBoard(empty);
    setInitialBoard(empty);
    setConflicts(new Set());
    setWaveIndices(new Set());
    setHintIndices(new Set());
    setIsSolvedState(false);
    setStatusMsg('Board cleared. Enter your clues manually!');
    setIsError(false);
  };

  // Apply a parsed board imported from text.
  const applyImport = (values, importSize) => {
    setBoard([...values]);
    setInitialBoard([...values]);
    setConflicts(new Set());
    setHintIndices(new Set());
    setWaveIndices(new Set());
    setIsSolvedState(false);
    setIsError(false);
    runConflictCheck(values, importSize);
    setStatusMsg(`Imported ${importSize}x${importSize} puzzle.`);
  };

  // Parse the textarea contents and load the puzzle (auto-detecting board size).
  const handleImport = () => {
    const { values, size: detected, error } = parsePuzzleText(importText);
    if (error) {
      setIsError(true);
      setStatusMsg(`Import failed: ${error}.`);
      return;
    }

    setIsError(false);
    setShowImport(false);
    setImportText('');

    pendingImportRef.current = values;
    if (detected !== size) {
      setSize(detected); // generate effect will consume the pending import
    } else {
      pendingImportRef.current = null;
      applyImport(values, detected);
    }
  };

  // Generate a fresh puzzle (or apply a pending import) whenever size /
  // difficulty changes and once the engine is ready.
  useEffect(() => {
    if (!wasmLoaded) return;
    if (pendingImportRef.current) {
      const values = pendingImportRef.current;
      pendingImportRef.current = null;
      applyImport(values, size);
      return;
    }
    generateNewPuzzle(size, difficulty);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [size, difficulty, wasmLoaded]);

  return (
    <div className="app-container">
      <header className="header">
        <h1>SUKODU</h1>
        <p>Rust-WASM Sudoku Solver & Generator</p>
      </header>

      <main className="dashboard">
        {/* Sudoku Board Wrapper */}
        <section className="board-wrapper">
          <div
            className="sudoku-board"
            style={{
              gridTemplateColumns: `repeat(${size}, 1fr)`,
            }}
          >
            {board.map((val, idx) => {
              const r = Math.floor(idx / size);
              const c = idx % size;
              const isClue = initialBoard[idx] !== 0;
              const isSolved = isSolvedState && !isClue;
              const isUserFilled = !isSolvedState && !isClue && val !== 0;
              const isConflict = conflicts.has(idx);
              const isHint = hintIndices.has(idx);

              // Grid Borders classes
              let classNames = ['sudoku-cell'];
              if (isClue) classNames.push('clue');
              if (isSolved) classNames.push('solved');
              if (isUserFilled) classNames.push('user-filled');
              if (isHint) classNames.push('hint');
              if (isConflict) classNames.push('conflict');
              if (r % sqrt === 0 && r > 0) classNames.push('block-top');
              if (c % sqrt === 0 && c > 0) classNames.push('block-left');

              // Highlighting helpers
              if (selectedCell && (selectedCell.row === r || selectedCell.col === c)) {
                classNames.push('highlight-axis');
              }
              if (highlightedValue > 0 && val === highlightedValue) {
                classNames.push('highlight-match');
              }
              if (waveIndices.has(idx)) {
                classNames.push('solved-animate');
              }

              return (
                <input
                  key={idx}
                  id={`cell-${r}-${c}`}
                  className={classNames.join(' ')}
                  style={getCellSizeStyle()}
                  type="text"
                  inputMode={size <= 9 ? 'numeric' : 'text'}
                  value={formatCellValue(val)}
                  readOnly={isClue || !wasmLoaded}
                  onFocus={(e) => {
                    setSelectedCell({ row: r, col: c });
                    if (val > 0) setHighlightedValue(val);
                    e.target.select();
                  }}
                  onBlur={() => {
                    setSelectedCell(null);
                    setHighlightedValue(0);
                  }}
                  onChange={(e) => handleChange(r, c, e)}
                  onKeyDown={(e) => handleKeyDown(r, c, e)}
                  autoComplete="off"
                />
              );
            })}
          </div>

          {/* Number palette — primary input on touch devices */}
          {board.length > 0 && (
            <div className="palette">
              {Array.from({ length: size }, (_, i) => i + 1).map((v) => (
                <button
                  key={v}
                  className="palette-btn"
                  onMouseDown={(e) => e.preventDefault()} // keep cell focus
                  onClick={() => enterValue(v)}
                  disabled={!wasmLoaded}
                >
                  {formatCellValue(v)}
                </button>
              ))}
              <button
                className="palette-btn erase"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => enterValue(0)}
                disabled={!wasmLoaded}
                aria-label="Erase cell"
              >
                ⌫
              </button>
            </div>
          )}
        </section>

        {/* Controls Panel */}
        <section className="panel">
          {/* Size Selector */}
          <div>
            <div className="section-title">Board Size</div>
            <div className="selector-group">
              {SIZES.map((s) => (
                <button
                  key={s}
                  className={`selector-btn ${size === s ? 'active' : ''}`}
                  onClick={() => setSize(s)}
                  disabled={!wasmLoaded || isBusy}
                >
                  {s}x{s}
                </button>
              ))}
            </div>
          </div>

          {/* Difficulty Selector */}
          <div>
            <div className="section-title">Difficulty</div>
            <div className="selector-group diff-group">
              {DIFFICULTIES.map((d) => (
                <button
                  key={d}
                  className={`selector-btn ${difficulty === d ? 'active' : ''}`}
                  onClick={() => setDifficulty(d)}
                  disabled={!wasmLoaded || isBusy}
                >
                  {d}
                </button>
              ))}
            </div>
          </div>

          {/* Action Buttons */}
          <div className="action-buttons">
            <button
              className="primary-btn"
              onClick={() => generateNewPuzzle(size, difficulty)}
              disabled={!wasmLoaded || isBusy}
            >
              Generate New
            </button>
            <button
              className="secondary-btn accent-btn"
              onClick={solvePuzzle}
              disabled={!wasmLoaded || isBusy || board.length === 0}
            >
              Solve Instantly
            </button>
            <button
              className="secondary-btn"
              onClick={giveHint}
              disabled={!wasmLoaded || isBusy || board.length === 0}
            >
              Hint
            </button>
            <button
              className="secondary-btn"
              onClick={resetBoard}
              disabled={!wasmLoaded || isBusy || board.length === 0}
            >
              Reset to Clues
            </button>
            <button
              className="secondary-btn"
              onClick={clearBoard}
              disabled={!wasmLoaded || isBusy || board.length === 0}
            >
              Clear Grid
            </button>
          </div>

          {/* Import Puzzle */}
          <div>
            <div className="section-title">Import Puzzle</div>
            <button
              className="secondary-btn"
              onClick={() => setShowImport((s) => !s)}
              disabled={!wasmLoaded || isBusy}
            >
              {showImport ? 'Hide Import' : 'Import from Text'}
            </button>
            {showImport && (
              <div className="import-box">
                <textarea
                  className="import-textarea"
                  placeholder={'Paste a puzzle — whitespace-separated numbers, 0 for blanks.\nGrid size is detected from the value count (4, 9, 16, 25).'}
                  value={importText}
                  onChange={(e) => setImportText(e.target.value)}
                  rows={6}
                  spellCheck={false}
                />
                <button
                  className="primary-btn"
                  onClick={handleImport}
                  disabled={!wasmLoaded || isBusy}
                >
                  Load Puzzle
                </button>
              </div>
            )}
          </div>

          {/* Status Bar */}
          <div className={`status-box ${isError ? 'error' : ''}`}>
            <span
              className={`status-dot ${wasmLoaded && !isError ? 'active' : ''} ${isError ? 'error' : ''} ${isBusy ? 'busy' : ''}`}
            />
            <span>{statusMsg}</span>
          </div>
        </section>
      </main>

      <footer className="footer">
        <p>
          Built with Rust & Dancing Links. View on{' '}
          <a
            href="https://github.com/adiatma85/sukodu"
            target="_blank"
            rel="noreferrer"
          >
            GitHub
          </a>
        </p>
      </footer>
    </div>
  );
}

export default App;
