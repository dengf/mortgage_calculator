import React from 'react';
import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import About from './About';
import { I18nProvider } from '../i18n';

const show = (Component, region, locale = 'en') =>
  render(
    <I18nProvider initialLocale={locale}>
      <Component region={region} />
    </I18nProvider>,
  );

describe('region-scoped explanatory content', () => {
  it('never mentions Singapore rules to a US buyer', () => {
    const { container } = show(About, 'US');
    // Reading about TDSR and CPF while buying in Ohio makes a reader doubt
    // whether the figures above apply to them either.
    expect(container.textContent).not.toMatch(/TDSR|MSR|CPF|stamp duty|HDB/i);
    expect(container.textContent).toMatch(/PMI/);
    expect(container.textContent).toMatch(/ZIP code/);
  });

  it('never mentions US rules to a Singapore buyer', () => {
    const { container } = show(About, 'SG');
    expect(container.textContent).not.toMatch(/PMI|ZIP code|jumbo|Fannie Mae/i);
    expect(container.textContent).toMatch(/TDSR/);
    expect(container.textContent).toMatch(/CPF/);
  });

  it('keeps refinance break-even in both, since it is region-neutral', () => {
    const us = show(About, 'US').container.textContent;
    expect(us).toMatch(/break-even/);
    const sg = show(About, 'SG').container.textContent;
    expect(sg).toMatch(/break-even/);
  });

  it('scopes the disclaimer too', () => {
    expect(show(About, 'US').container.textContent).toMatch(/state averages/);
    expect(show(About, 'SG').container.textContent).toMatch(/IRAS/);
  });

  it('falls back to US for an unknown region rather than rendering nothing', () => {
    const { container } = show(About, 'XX');
    expect(container.querySelectorAll('.about-item')).toHaveLength(4);
  });

  it('translates the region content rather than falling back to English', () => {
    const { container } = show(About, 'SG', 'zh-Hans');
    expect(container.textContent).toMatch(/公积金/);
    expect(container.textContent).not.toMatch(/Ordinary Account/);
  });
});
