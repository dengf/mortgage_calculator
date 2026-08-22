import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

// English that never reaches `t()` renders untranslated in the middle of an
// otherwise fully translated page. Three shipped that way and were found by
// eye rather than by test: "20.0% of price" under the deposit field, "Custom
// scenario" as a row name, and "Scenario label" as a placeholder.
//
// The catalog tests next door check that the catalogs agree with each other.
// Nothing checked for text that never enters a catalog at all.

const COMPONENTS = path.join(import.meta.dirname, 'components');

function sources() {
  return fs
    .readdirSync(COMPONENTS)
    .filter((f) => f.endsWith('.jsx') && !f.endsWith('.test.jsx'))
    .map((file) => ({
      file,
      // Comments are prose by design and must not be flagged.
      source: fs
        .readFileSync(path.join(COMPONENTS, file), 'utf8')
        .replace(/\/\*[\s\S]*?\*\//g, '')
        .replace(/^\s*\/\/.*$/gm, ''),
    }));
}

/** User-visible English that is not coming from a catalog. */
function hardcodedEnglish(source) {
  const found = [];
  source.split('\n').forEach((line, index) => {
    // Text sitting directly between JSX tags, e.g. `<span>Loan amount</span>`.
    for (const m of line.matchAll(/>\s*([A-Za-z][A-Za-z ,.'%-]{4,})\s*</g)) {
      found.push({ line: index + 1, text: m[1].trim() });
    }
    // Props whose value the user reads.
    for (const m of line.matchAll(
      /\b(?:label|placeholder|title|aria-label)\s*[=:]\s*['"]([^'"]{3,})['"]/g,
    )) {
      found.push({ line: index + 1, text: m[1] });
    }
  });
  return found;
}

describe('components put no English on screen that a catalog cannot translate', () => {
  it('finds sources to check', () => {
    expect(sources().length).toBeGreaterThan(10);
  });

  it.each(sources())('$file', ({ file, source }) => {
    expect({ file, hardcoded: hardcodedEnglish(source) }).toEqual({ file, hardcoded: [] });
  });
});
