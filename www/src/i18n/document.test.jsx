import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { I18nProvider, canonicalFor, detectLocale, useI18n } from './index';

function Probe() {
  const { locale, setLocale } = useI18n();
  return (
    <div>
      <span data-testid="locale">{locale}</span>
      <button type="button" onClick={() => setLocale('zh-Hans')}>
        switch
      </button>
    </div>
  );
}

const meta = (sel) => document.head.querySelector(sel)?.getAttribute('content');

describe('document language metadata', () => {
  beforeEach(() => {
    document.documentElement.lang = 'en';
    document.head.innerHTML = '';
    window.history.replaceState(null, '', '/');
    window.localStorage?.clear();
  });

  it('sets lang and title on first paint, not only on change', async () => {
    // The bug this pins: a returning visitor with a stored Chinese preference
    // saw Chinese text under lang="en", so screen readers used an English
    // voice and crawlers saw an English page.
    render(
      <I18nProvider initialLocale="zh-Hans">
        <Probe />
      </I18nProvider>,
    );

    await waitFor(() => expect(document.documentElement.lang).toBe('zh-Hans'));
    expect(document.title).toMatch(/房贷计算器/);
    expect(meta('meta[name="description"]')).toMatch(/浏览器/);
  });

  it('retitles the document when the locale changes', async () => {
    render(
      <I18nProvider initialLocale="en">
        <Probe />
      </I18nProvider>,
    );
    await waitFor(() => expect(document.title).toMatch(/Mortgage Calculator/));

    await userEvent.click(screen.getByRole('button', { name: 'switch' }));

    await waitFor(() => expect(document.title).toMatch(/房贷计算器/));
    expect(document.documentElement.lang).toBe('zh-Hans');
  });

  it('gives each locale an addressable URL and canonical', async () => {
    render(
      <I18nProvider initialLocale="en">
        <Probe />
      </I18nProvider>,
    );
    await waitFor(() =>
      expect(document.head.querySelector('link[rel="canonical"]')?.getAttribute('href')).toBe(
        canonicalFor('en'),
      ),
    );
    expect(window.location.search).not.toMatch(/lang=/);

    await userEvent.click(screen.getByRole('button', { name: 'switch' }));

    await waitFor(() => expect(window.location.search).toBe('?lang=zh-Hans'));
    expect(document.head.querySelector('link[rel="canonical"]')?.getAttribute('href')).toBe(
      canonicalFor('zh-Hans'),
    );
  });

  it('honours ?lang= over a stored preference, so shared links land right', () => {
    window.localStorage?.setItem('mc:locale', 'en');
    window.history.replaceState(null, '', '/?lang=zh-Hant');
    expect(detectLocale()).toBe('zh-Hant');
  });

  it('ignores a ?lang= value it has no catalog for', () => {
    window.history.replaceState(null, '', '/?lang=klingon');
    expect(detectLocale()).toBe('en');
  });
});
