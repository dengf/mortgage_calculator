/**
 * Scenario bindings for component tests.
 *
 * These are stubs standing in for the wasm module, not a second
 * implementation: they define the input a test feeds a component so the
 * component's own wiring can be asserted. The rules themselves are tested
 * in crates/mortgage-calc/src/scenario.rs, where they live.
 */
export function scenarioBindings() {
  return {
    summarize_scenario: ({ home_price, down_payment }) => ({
      principal: Math.max(0, home_price - down_payment),
      down_payment_percent: home_price > 0 ? (down_payment / home_price) * 100 : null,
    }),
    down_payment_for_percent: ({ home_price, percent }) => ({
      down_payment: Math.round(home_price * percent) / 100,
    }),
  };
}

/** Spreads the scenario stubs under whatever else a test needs to mock. */
export function withScenario(module = {}) {
  return { ...scenarioBindings(), ...module };
}
