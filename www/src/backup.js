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

// Nobody exporting their own data produces a file anywhere near these sizes.
// They exist so a corrupted or adversarial file -- forwarded as a "shared
// scenario", say -- can't pass the shape check and then, after
// `clear_all_scenarios` has already wiped what was really there, either
// stall the import loop on an unbounded number of records or hand a later
// reader (`SavedScenarios`, `currentInputs`) a string too large to be a real
// scenario's saved inputs.
const MAX_SCENARIOS = 10_000;
const MAX_ID_LENGTH = 200;
const MAX_NAME_LENGTH = 500;
const MAX_INPUTS_JSON_LENGTH = 100_000;

function isScenarioArray(value) {
  return (
    Array.isArray(value) &&
    value.length <= MAX_SCENARIOS &&
    value.every(
      (s) =>
        s &&
        typeof s === 'object' &&
        typeof s.id === 'string' &&
        s.id.length <= MAX_ID_LENGTH &&
        typeof s.calculator === 'string' &&
        s.calculator.length <= MAX_ID_LENGTH &&
        typeof s.name === 'string' &&
        s.name.length <= MAX_NAME_LENGTH &&
        typeof s.created_at === 'number' &&
        typeof s.inputs_json === 'string' &&
        s.inputs_json.length <= MAX_INPUTS_JSON_LENGTH &&
        parsesAsJson(s.inputs_json),
    )
  );
}

// A scenario's `inputs_json` is read back with a bare `JSON.parse` wherever
// a saved scenario is loaded (`SavedScenarios`, `currentInputs`) -- rejecting
// a file that doesn't even parse here means the failure surfaces as "this
// file isn't a backup" at import time, not as an uncaught exception on
// whatever later screen tries to load the poisoned record.
function parsesAsJson(text) {
  try {
    JSON.parse(text);
    return true;
  } catch {
    return false;
  }
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
