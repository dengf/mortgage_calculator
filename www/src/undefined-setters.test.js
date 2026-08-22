import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

// Two tabs shipped the same crash: Compare and Amortization both called
// setters that stopped existing when their state moved into the shared
// scenario. Nothing failed, because a ReferenceError inside an event handler
// only fires when a user clicks the thing -- and neither path had a test.
//
// This is a source-level guard rather than a rendering test, so it covers
// every handler in every component including ones nobody has written a test
// for yet. It is not a type checker: it asks only whether a name that is
// called like a setter is ever *bound* in the file it is called from.

const SRC = path.join(import.meta.dirname, 'components');

function jsxSources() {
  return fs
    .readdirSync(SRC)
    .filter((f) => f.endsWith('.jsx') && !f.endsWith('.test.jsx'))
    .map((f) => ({ file: f, source: fs.readFileSync(path.join(SRC, f), 'utf8') }));
}

/**
 * Names called as `setSomething(...)` that never appear anywhere else in the
 * file.
 *
 * A setter that is bound -- by `useState`, by destructuring a hook's return,
 * or by a prop -- appears at least once without a following `(`. One that is
 * only ever called is a name with nothing behind it.
 */
function unboundSetters(source) {
  const called = new Set([...source.matchAll(/\b(set[A-Z]\w*)\s*\(/g)].map((m) => m[1]));
  return [...called].filter((name) => {
    const everyMention = source.match(new RegExp(`\\b${name}\\b(?!\\s*\\()`, 'g'));
    return !everyMention;
  });
}

describe('components call no setter they do not have', () => {
  it('finds sources to check', () => {
    expect(jsxSources().length).toBeGreaterThan(5);
  });

  it.each(jsxSources())('$file', ({ file, source }) => {
    expect({ file, unbound: unboundSetters(source) }).toEqual({ file, unbound: [] });
  });
});
