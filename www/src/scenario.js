import { useMemo } from 'react';

/**
 * The one loan the Payment, Amortization and Compare tabs are all describing.
 *
 * These three tabs used to hold their own copies of loan amount, rate and
 * term, so dialling in a scenario on one and switching to another silently
 * reverted to defaults — five tabs that behaved like five unrelated
 * calculators sharing a header.
 *
 * Affordability deliberately stays out of this: it answers the opposite
 * question (what loan could I get?) and derives a price rather than taking
 * one, so sharing inputs there would mean writing the user's answer back
 * over their question.
 */
export const DEFAULT_SCENARIO = {
  // Price and deposit lead, and the loan is derived from them. Nobody
  // shopping for a home knows their loan amount; they know the asking price
  // and what they can put down.
  homePrice: 500000,
  downPayment: 100000,
  rate: 6.5,
  termYears: 30,
  frequency: 'monthly',
};

// What an empty form summarizes to, and what is shown when there is no wasm
// module to ask. Deriving these here instead would be a second answer to
// "what is the loan amount" -- see mortgage-calc/src/scenario.rs.
const EMPTY_SUMMARY = { principal: 0, downPaymentPercent: null };

/** Coerces a form field, which may be an empty string mid-typing. */
const num = (value) => Number(value) || 0;

/**
 * Principal and deposit percentage, computed by the Rust core.
 *
 * `downPaymentPercent` is `null` rather than `0` when there is no price to
 * divide by: a buyer who hasn't entered a price has no deposit percentage,
 * and "0.0% of price" would state something the inputs don't.
 */
export function summarizeScenario(wasmModule, scenario) {
  if (!wasmModule?.summarize_scenario) return EMPTY_SUMMARY;
  const result = wasmModule.summarize_scenario({
    home_price: num(scenario.homePrice),
    down_payment: num(scenario.downPayment),
  });
  return {
    principal: result?.principal ?? 0,
    downPaymentPercent: result?.down_payment_percent ?? null,
  };
}

/** The deposit a percentage of the price comes to, in whole cents. */
export function downPaymentForPercent(wasmModule, homePrice, percent) {
  if (!wasmModule?.down_payment_for_percent) return null;
  const result = wasmModule.down_payment_for_percent({
    home_price: num(homePrice),
    percent: num(percent),
  });
  return result?.down_payment ?? null;
}

/** Memoized so the boundary is crossed once per change, not once per render. */
export function useScenarioSummary(wasmModule, scenario) {
  const { homePrice, downPayment } = scenario;
  return useMemo(
    () => summarizeScenario(wasmModule, { homePrice, downPayment }),
    [wasmModule, homePrice, downPayment],
  );
}
