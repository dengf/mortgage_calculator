// The shape of the file "Export all data" writes and "Import" reads.
//
// Host-layer, not a domain decision: this is our own file envelope, and
// the records inside are the same `ScenarioDto` shape `list_scenarios`
// already returns, round-tripped unchanged. What this module owns is
// refusing to touch anything until the file looks like ours -- an import
// that half-applies a wrong file, having already cleared what was there
// via Clear all, is unrecoverable for someone whose only copy was the
// thing they just deleted.

export const EXPORT_FORMAT = 'meifio.mortgage_calculator.v1';

function isScenarioArray(value) {
  return (
    Array.isArray(value) &&
    value.every(
      (s) =>
        s &&
        typeof s === 'object' &&
        typeof s.id === 'string' &&
        typeof s.calculator === 'string' &&
        typeof s.name === 'string' &&
        typeof s.created_at === 'number' &&
        typeof s.inputs_json === 'string',
    )
  );
}

/**
 * Validates a parsed export file.
 *
 * Returns `{ ok: true, scenarios, count }` or `{ ok: false, reason }`
 * where `reason` is an i18n key. Deliberately strict: a file we only
 * half-recognize is rejected rather than merged, because the import
 * replaces existing data.
 */
export function readBackup(payload) {
  if (!payload || typeof payload !== 'object') return { ok: false, reason: 'err.badImportFile' };
  if (payload.format !== EXPORT_FORMAT) return { ok: false, reason: 'err.badImportFile' };
  if (!isScenarioArray(payload.scenarios)) return { ok: false, reason: 'err.badImportFile' };

  return { ok: true, scenarios: payload.scenarios, count: payload.scenarios.length };
}
