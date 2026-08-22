import { describe, expect, it } from 'vitest';
import { formatDuration, payoffDate, periodsToYearsMonths } from './duration';
import { translate } from './i18n';

const t = (key, params) => translate('en', key, params);

describe('duration', () => {
  it('converts payment periods to years and months', () => {
    expect(periodsToYearsMonths(233, 12)).toEqual({ years: 19, months: 5 });
    expect(periodsToYearsMonths(360, 12)).toEqual({ years: 30, months: 0 });
  });

  it('handles non-monthly cadences', () => {
    // 26 fortnightly payments is a year.
    expect(periodsToYearsMonths(26, 26)).toEqual({ years: 1, months: 0 });
    expect(periodsToYearsMonths(52, 52)).toEqual({ years: 1, months: 0 });
  });

  it('drops the empty half rather than reading "20 yr 0 mo"', () => {
    expect(formatDuration(360, 12, t)).toBe('30 yr');
    expect(formatDuration(7, 12, t)).toBe('7 mo');
    expect(formatDuration(127, 12, t)).toBe('10 yr 7 mo');
  });

  it('names the payoff month', () => {
    const from = new Date(2026, 0, 1); // Jan 2026
    expect(payoffDate(233, 12, 'en', from)).toBe('Jun 2045');
  });

  it('treats a missing count as zero rather than NaN', () => {
    expect(formatDuration(undefined, 12, t)).toBe('0 mo');
  });
});
