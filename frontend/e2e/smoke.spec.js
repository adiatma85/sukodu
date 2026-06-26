import { test, expect } from '@playwright/test';

const countEmptyCells = (page) =>
  page.locator('.sudoku-cell').evaluateAll((els) => els.filter((e) => e.value === '').length);

test.describe('SUKODU web app', () => {
  test('loads the WASM engine and auto-generates a 9x9 puzzle', async ({ page }) => {
    await page.goto('/');
    // The status line settles on "Generated ..." only after the worker + WASM load.
    await expect(page.locator('.status-box')).toContainText('Generated 9x9', { timeout: 20_000 });
    await expect(page.locator('.sudoku-cell')).toHaveCount(81);
    await expect(page.locator('.sudoku-cell.clue').first()).toBeVisible();
  });

  test('solves the current board completely', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.status-box')).toContainText('Generated 9x9', { timeout: 20_000 });

    expect(await countEmptyCells(page)).toBeGreaterThan(0);
    await page.getByRole('button', { name: 'Solve Instantly' }).click();

    await expect(page.locator('.status-box')).toContainText('solved successfully', { timeout: 20_000 });
    expect(await countEmptyCells(page)).toBe(0);
  });

  test('reveals exactly one cell on Hint', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.status-box')).toContainText('Generated 9x9', { timeout: 20_000 });

    const before = await countEmptyCells(page);
    await page.getByRole('button', { name: 'Hint' }).click();
    await expect(page.locator('.status-box')).toContainText('Revealed one cell', { timeout: 20_000 });
    expect(await countEmptyCells(page)).toBe(before - 1);
  });

  test('imports a puzzle from text and switches board size', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.status-box')).toContainText('Generated 9x9', { timeout: 20_000 });

    await page.getByRole('button', { name: 'Import from Text' }).click();
    // 16 tokens -> a 4x4 board.
    await page.locator('.import-textarea').fill('1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4');
    await page.getByRole('button', { name: 'Load Puzzle' }).click();

    await expect(page.locator('.status-box')).toContainText('Imported 4x4');
    await expect(page.locator('.sudoku-cell')).toHaveCount(16);
  });
});
