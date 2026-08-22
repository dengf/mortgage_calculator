/**
 * Time in the units people actually think in.
 *
 * The extra-payment summary reported "127 payments saved" and "233 payments"
 * — engineer-native units. Nobody thinks in payments. The emotional payload
 * of an extra-payment calculator is "you finish ten and a half years early",
 * and a payment count throws that away.
 */

/** Converts a count of payment periods into whole years and months. */
export function periodsToYearsMonths(periods, periodsPerYear) {
  const totalMonths = Math.round((Number(periods) || 0) * (12 / periodsPerYear));
  return { years: Math.floor(totalMonths / 12), months: totalMonths % 12 };
}

/**
 * "10 yr 7 mo", dropping whichever half is zero so exact spans read cleanly
 * rather than as "20 yr 0 mo".
 */
export function formatDuration(periods, periodsPerYear, t) {
  const { years, months } = periodsToYearsMonths(periods, periodsPerYear);
  if (years && months) return t('duration.yearsMonths', { years, months });
  if (years) return t('duration.years', { years });
  return t('duration.months', { months });
}

/**
 * The month a loan running `periods` from now is paid off.
 *
 * Deliberately month-precision: the day depends on a completion date this
 * calculator never asks for, and rendering one would imply knowledge it
 * doesn't have.
 */
export function payoffDate(periods, periodsPerYear, locale, from = new Date()) {
  const totalMonths = Math.round((Number(periods) || 0) * (12 / periodsPerYear));
  const date = new Date(from.getFullYear(), from.getMonth() + totalMonths, 1);
  return new Intl.DateTimeFormat(locale, { year: 'numeric', month: 'short' }).format(date);
}
