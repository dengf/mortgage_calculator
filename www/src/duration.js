/**
 * Time in the units people actually think in.
 *
 * The extra-payment summary reported "127 payments saved" and "233 payments"
 * — engineer-native units. Nobody thinks in payments. The emotional payload
 * of an extra-payment calculator is "you finish ten and a half years early",
 * and a payment count throws that away.
 *
 * The conversion is `mortgage_core::PaymentFrequency`'s — it depends on the
 * payment cadence, and a second table of periods-per-year is a second thing
 * to get wrong. What is left here is wording and date formatting, which need
 * the reader's catalog and locale.
 */

const NO_DURATION = { years: 0, months: 0, total_months: 0, years_exact: 0, periods_per_year: 12 };

/** Years, months and cadence for a count of payment periods. */
export function describeDuration(wasmModule, periods, frequency) {
  if (!wasmModule?.describe_duration) return NO_DURATION;
  return (
    wasmModule.describe_duration({
      periods: Math.max(0, Math.round(Number(periods) || 0)),
      frequency,
    }) ?? NO_DURATION
  );
}

/**
 * The inverse of `describeDuration`: how many payment periods a term
 * entered in years comes to at `frequency`. `PaymentFrequency` is the one
 * table of periods-per-year in this app; multiplying years by 12 here would
 * be a second, monthly-only copy of it.
 */
export function periodsInYears(wasmModule, years, frequency) {
  if (!wasmModule?.periods_in_years) return 0;
  return (
    wasmModule.periods_in_years({ years: Number(years) || 0, frequency })?.periods ?? 0
  );
}

/**
 * "10 yr 7 mo", dropping whichever half is zero so exact spans read cleanly
 * rather than as "20 yr 0 mo".
 */
export function formatDuration(duration, t) {
  const { years, months } = duration ?? NO_DURATION;
  if (years && months) return t('duration.yearsMonths', { years, months });
  if (years) return t('duration.years', { years });
  return t('duration.months', { months });
}

/**
 * The month a loan running `duration` from now is paid off.
 *
 * Deliberately month-precision: the day depends on a completion date this
 * calculator never asks for, and rendering one would imply knowledge it
 * doesn't have.
 */
export function payoffDate(duration, locale, from = new Date()) {
  const totalMonths = duration?.total_months ?? 0;
  const date = new Date(from.getFullYear(), from.getMonth() + totalMonths, 1);
  return new Intl.DateTimeFormat(locale, { year: 'numeric', month: 'short' }).format(date);
}
