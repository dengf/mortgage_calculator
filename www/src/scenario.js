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

/** The amount actually borrowed. Never negative, however the fields are set. */
export function principalOf(scenario) {
  const price = Number(scenario.homePrice) || 0;
  const down = Number(scenario.downPayment) || 0;
  return Math.max(0, price - down);
}

/** Deposit as a share of price, or `null` when there's no price to divide by. */
export function downPaymentPercent(scenario) {
  const price = Number(scenario.homePrice) || 0;
  if (price <= 0) return null;
  return ((Number(scenario.downPayment) || 0) / price) * 100;
}
