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
});
