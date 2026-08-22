import { describe, expect, it, vi } from 'vitest';
import { describeDuration, formatDuration, payoffDate } from './duration';

// The period-to-time conversion is mortgage-core's, tested in
// crates/mortgage-core/src/frequency.rs. What is left here is wording and
// date formatting, which need the reader's catalog and locale.

const t = (key, params) =>
  ({
    'duration.years': `${params?.years} yr`,
    'duration.months': `${params?.months} mo`,
    'duration.yearsMonths': `${params?.years} yr ${params?.months} mo`,
  })[key];

const duration = (years, months) => ({
  years,
  months,
  total_months: years * 12 + months,
  years_exact: years + months / 12,
  periods_per_year: 12,
});

describe('formatDuration', () => {
  it('drops the empty half rather than reading "20 yr 0 mo"', () => {
    expect(formatDuration(duration(30, 0), t)).toBe('30 yr');
    expect(formatDuration(duration(0, 7), t)).toBe('7 mo');
    expect(formatDuration(duration(10, 7), t)).toBe('10 yr 7 mo');
  });

  it('treats a missing duration as zero rather than crashing', () => {
    expect(formatDuration(undefined, t)).toBe('0 mo');
    expect(formatDuration(null, t)).toBe('0 mo');
  });
});

describe('payoffDate', () => {
  it('names the payoff month', () => {
    const from = new Date(2026, 0, 15);
    expect(payoffDate(duration(19, 5), 'en', from)).toBe('Jun 2045');
  });

  it('is month-precision, since the completion date is never asked for', () => {
    const from = new Date(2026, 0, 31);
    // Same month regardless of the day it is read on.
    expect(payoffDate(duration(0, 1), 'en', from)).toBe('Feb 2026');
  });

  it('falls back to today when there is no duration', () => {
    const from = new Date(2026, 5, 1);
    expect(payoffDate(null, 'en', from)).toBe('Jun 2026');
  });
});

describe('describeDuration', () => {
  it('hands the period count and cadence to the module', () => {
    const describe_duration = vi.fn(() => duration(10, 7));
    const result = describeDuration({ describe_duration }, 127, 'biweekly');

    expect(describe_duration).toHaveBeenCalledWith({ periods: 127, frequency: 'biweekly' });
    expect(result.years).toBe(10);
  });

  it('never asks about a negative or fractional number of payments', () => {
    const describe_duration = vi.fn(() => duration(0, 0));
    describeDuration({ describe_duration }, -5, 'monthly');
    describeDuration({ describe_duration }, 12.7, 'monthly');

    expect(describe_duration).toHaveBeenNthCalledWith(1, { periods: 0, frequency: 'monthly' });
    expect(describe_duration).toHaveBeenNthCalledWith(2, { periods: 13, frequency: 'monthly' });
  });

  it('describes no time at all when there is no module to ask', () => {
    expect(describeDuration(null, 360, 'monthly').total_months).toBe(0);
  });
});
