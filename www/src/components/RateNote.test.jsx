import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import RateNote from './RateNote';
import { I18nProvider } from '../i18n';
import { DEFAULT_RATE } from '../rate';

const SORA_NOTE = {
  code: 'note.floatingBase',
  params: { base: '1.12' },
  text: 'These figures assume the base rate stays at 1.12%.',
};

const show = (wasmModule, rate = DEFAULT_RATE, locale = 'en') =>
  render(
    <I18nProvider initialLocale={locale}>
      <RateNote wasmModule={wasmModule} rate={rate} />
    </I18nProvider>,
  );

describe('what a rate assumed', () => {
  it('prints the note the core returns', () => {
    show({ rate_note: () => SORA_NOTE });
    // The figure that was held still, not the rate charged: a reader can
    // already see the rate.
    expect(screen.getByRole('note')).toHaveTextContent('stays at 1.12%');
  });

  it('says nothing when nothing was assumed', () => {
    // A caveat under a contractual rate teaches the reader to skip the ones
    // that mean something.
    show({ rate_note: () => null });
    expect(screen.queryByRole('note')).toBeNull();
  });

  it('asks the core rather than reading the shape itself', () => {
    // Whether a step-up rests on a benchmark is a fact about the quote, and
    // it is decided in Rust. A component that inferred it from `kind` would
    // be a second, quieter copy of that rule.
    const rate_note = vi.fn(() => SORA_NOTE);
    show({ rate_note }, { ...DEFAULT_RATE, kind: 'reverting', baseFloats: true });
    expect(rate_note).toHaveBeenCalledWith(expect.objectContaining({ base_floats: true }));
  });

  it('carries the toggle across as false when the base is agreed', () => {
    const rate_note = vi.fn(() => null);
    show({ rate_note }, { ...DEFAULT_RATE, kind: 'reverting', baseFloats: false });
    expect(rate_note).toHaveBeenCalledWith(expect.objectContaining({ base_floats: false }));
  });

  it('leaves the figures standing if the note cannot be built', () => {
    // The note is a courtesy on top of a working tab. Blanking the page
    // over a missing caveat is the worse failure.
    expect(() =>
      show({
        rate_note: () => {
          throw new Error('boundary');
        },
      }),
    ).not.toThrow();
    expect(screen.queryByRole('note')).toBeNull();
  });

  it('reads the note in the language the page is in', () => {
    show({ rate_note: () => SORA_NOTE }, DEFAULT_RATE, 'zh-Hans');
    // Composed from the code and its values, not the English text the core
    // ships as a fallback.
    expect(screen.getByRole('note')).toHaveTextContent('1.12%');
    expect(screen.getByRole('note').textContent).toContain('基准利率');
  });
});
