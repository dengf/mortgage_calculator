import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// Read as text rather than imported: the point is what the sheet *says*,
// and jsdom parses no media queries at all.
const css = readFileSync(join(dirname(fileURLToPath(import.meta.url)), 'styles/main.css'), 'utf8');

/** Every `@media <query> {` opener in the sheet, with the line it sits on. */
const mediaQueries = () =>
  css
    .split('\n')
    .map((line, i) => ({ line: i + 1, text: line.trim() }))
    .filter(({ text }) => text.startsWith('@media'));

describe('the report stylesheet on paper', () => {
  // The document exists to be printed. A screen rule that leaks into print
  // is invisible until someone makes a PDF and sends it to a client, which
  // is the one moment nobody is looking at this file.

  it('scopes every width breakpoint after the print block to screen', () => {
    // An A4 page at the 12mm margins this sheet sets is about 703 CSS px,
    // so a bare `@media (max-width: 720px)` matches when printing. The
    // report's responsive rules sit *after* `@media print` and would win on
    // equal specificity -- putting phone margins, a stacked header and
    // tightened cells on every PDF anyone makes.
    const printAt = css.indexOf('@media print');
    expect(printAt).toBeGreaterThan(0);

    const after = mediaQueries().filter(
      ({ line }) => css.split('\n').slice(0, line).join('\n').length > printAt,
    );
    expect(after.length).toBeGreaterThan(0);

    const unscoped = after.filter(({ text }) => /max-width/.test(text) && !/\bscreen\b/.test(text));
    expect(unscoped).toEqual([]);
  });

  it('undoes the scroll container on paper', () => {
    // A wide table scrolls inside its own box on screen. On paper there is
    // nothing to scroll, and an overflow container can clip at a page
    // boundary -- losing the tail of the yearly schedule from the PDF.
    const print = css.slice(css.indexOf('@media print'));
    expect(print).toMatch(/\.report-table-wrap\s*\{[^}]*overflow:\s*visible/);
    expect(print).toMatch(/\.report-table-wrap\s*\{[^}]*background:\s*none/);
  });
});
