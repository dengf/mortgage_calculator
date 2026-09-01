// What the app becomes when the calculator engine cannot be loaded.
//
// This used to be a full second implementation of the mortgage math in
// JavaScript -- payment, amortization, affordability, refinance, comparison
// -- so a missing wasm build produced numbers rather than an error. Numbers
// that could disagree with the Rust core, silently, in production: a browser
// that fails to instantiate wasm got a plausible answer computed by different
// code than every test in this repo covers.
//
// Now it reports the failure and computes nothing. Every panel already knows
// how to render a result that carries an error, and says it in the reader's
// language.
//
// Scenario storage is kept, in memory: persistence is not a mortgage rule,
// and losing the save/load UI as well would make a page that is already
// degraded harder to diagnose.
export function createUnavailableModule() {
  console.warn('Calculator engine unavailable - run `npm run build:wasm`.');

  // Both forms, because panels prefer the code and fall back to the sentence.
  const unavailable = () => ({
    error: 'The calculator engine could not be loaded.',
    error_message: { code: 'err.engineUnavailable', params: {} },
  });

  return {
    calculate_payment: unavailable,
    calculate_amortization_schedule: unavailable,
    calculate_extra_payment_impact: unavailable,
    calculate_affordability: unavailable,
    calculate_refinance: unavailable,
    calculate_comparison: unavailable,
    calculate_singapore: unavailable,
    calculate_united_states: unavailable,
    calculate_sg_affordability: unavailable,

    // Deliberately absent: detect_region, summarize_scenario,
    // down_payment_for_percent and describe_duration. Their callers already
    // handle a module that cannot answer, and adding stubs here would be
    // re-creating in JavaScript exactly what was just moved out of it.
    get_common_rate_presets: () => [],

    _scenarios: [],
    _seq: 0,
    init_storage: async function () {},
    save_scenario: async function (params) {
      const id = params.id ?? `local-${this._seq++}`;
      this._scenarios = this._scenarios.filter((s) => s.id !== id);
      this._scenarios.push({
        id,
        calculator: params.calculator,
        name: params.name,
        created_at: Date.now(),
        inputs_json: params.inputs_json,
      });
      return { id, error: null };
    },
    list_scenarios: async function (calculator) {
      return {
        scenarios: this._scenarios
          .filter((s) => !calculator || s.calculator === calculator)
          .sort((a, b) => b.created_at - a.created_at),
        error: null,
      };
    },
    load_scenario: async function (id) {
      const scenario = this._scenarios.find((s) => s.id === id);
      if (!scenario) return { scenario: null, error: `scenario not found: ${id}` };
      return { scenario, error: null };
    },
    delete_scenario: async function (id) {
      this._scenarios = this._scenarios.filter((s) => s.id !== id);
      return { success: true, error: null };
    },
    clear_all_scenarios: async function () {
      this._scenarios = [];
      return { success: true, error: null };
    },

    _currentInputs: {},
    save_current_inputs: async function (key, inputsJson) {
      this._currentInputs[key] = inputsJson;
      return { success: true, error: null };
    },
    load_current_inputs: async function (key) {
      return { inputs_json: this._currentInputs[key] ?? null, error: null };
    },
    clear_current_inputs: async function () {
      this._currentInputs = {};
      return { success: true, error: null };
    },
  };
}
