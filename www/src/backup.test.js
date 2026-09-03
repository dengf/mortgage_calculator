import { describe, expect, it } from 'vitest';
import { EXPORT_FORMAT, readBackup } from './backup';

const scenario = (over = {}) => ({
  id: 's1',
  calculator: 'payment',
  name: '30yr fixed',
  created_at: 1700000000000,
  inputs_json: '{}',
  ...over,
});

const valid = (over = {}) => ({
  format: EXPORT_FORMAT,
  exported_at: '2026-08-29T00:00:00.000Z',
  scenarios: [scenario()],
  ...over,
});

describe('readBackup', () => {
  it('accepts a file this app wrote', () => {
    const result = readBackup(valid());
    expect(result.ok).toBe(true);
    expect(result.scenarios).toHaveLength(1);
    expect(result.count).toBe(1);
  });

  it('rejects anything without our format marker', () => {
    // The import replaces existing data, so a file we only half-recognize
    // must be refused outright rather than partially applied -- the
    // person's only copy may be the thing the import is about to delete.
    expect(readBackup({ ...valid(), format: undefined }).ok).toBe(false);
    expect(readBackup({ ...valid(), format: 'something.else' }).ok).toBe(false);
    expect(readBackup(null).ok).toBe(false);
    expect(readBackup('nope').ok).toBe(false);
  });

  it('names an i18n key rather than prose when it refuses', () => {
    expect(readBackup(null).reason).toBe('err.badImportFile');
  });

  it('rejects a missing scenarios field', () => {
    const { scenarios, ...withoutScenarios } = valid();
    expect(readBackup(withoutScenarios).ok).toBe(false);
  });

  it('rejects a scenarios value that is not an array', () => {
    expect(readBackup(valid({ scenarios: 'not an array' })).ok).toBe(false);
  });

  it('rejects a scenario entry missing a required field', () => {
    expect(readBackup(valid({ scenarios: [{ ...scenario(), id: undefined }] })).ok).toBe(false);
    expect(readBackup(valid({ scenarios: [{ ...scenario(), created_at: '2026' }] })).ok).toBe(false);
  });

  it('accepts scenarios spanning more than one calculator kind', () => {
    const result = readBackup(
      valid({ scenarios: [scenario({ id: 'a' }), scenario({ id: 'b', calculator: 'refinance' })] }),
    );
    expect(result.ok).toBe(true);
    expect(result.count).toBe(2);
  });

  it('accepts an empty scenario list', () => {
    const result = readBackup(valid({ scenarios: [] }));
    expect(result.ok).toBe(true);
    expect(result.count).toBe(0);
  });

  it('rejects a scenario whose inputs_json is not actually JSON', () => {
    // Every reader of a saved scenario (SavedScenarios, currentInputs) trusts
    // this string enough to hand it straight to JSON.parse. Catching a
    // corrupt or adversarial payload here means the failure is "this file
    // isn't a backup" at import time, before `clear_all_scenarios` has
    // already wiped whatever was really there -- not an uncaught exception
    // on whatever screen loads the poisoned record later.
    const result = readBackup(
      valid({ scenarios: [scenario({ inputs_json: '{not valid json' })] }),
    );
    expect(result.ok).toBe(false);
  });

  it('rejects an unreasonably large number of scenarios', () => {
    const scenarios = Array.from({ length: 10_001 }, (_, i) => scenario({ id: `s${i}` }));
    expect(readBackup(valid({ scenarios })).ok).toBe(false);
  });

  it('rejects a scenario with an unreasonably long name or inputs_json', () => {
    expect(readBackup(valid({ scenarios: [scenario({ name: 'x'.repeat(501) })] })).ok).toBe(
      false,
    );
    expect(
      readBackup(
        valid({ scenarios: [scenario({ inputs_json: JSON.stringify('x'.repeat(100_001)) })] }),
      ).ok,
    ).toBe(false);
  });
});
