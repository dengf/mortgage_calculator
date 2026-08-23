import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import ReportDocument from './ReportDocument';
import { I18nProvider } from '../i18n';
import { DEFAULT_SCENARIO } from '../scenario';

// A report for the default Singapore package, in the shape build_report
// returns. The figures themselves are tested in mortgage-calc; what is
// under test here is what the document says about them.
const REPORT = {
  principal: 400000,
  term_years: 25,
  initial_rate_percent: 1.42,
  initial_payment: 1584.75,
  final_rate_percent: 1.72,
  payment_after_reversion: 1637.12,
  lock_in_years: 2,
  total_paid: 489880.62,
  total_interest: 89880.62,
  interest_share_percent: 18.347,
  bands: [
    { from_year: 1, to_year: 2, annual_rate_percent: 1.42, payment: 1584.75 },
    { from_year: 3, to_year: 25, annual_rate_percent: 1.72, payment: 1637.12 },
  ],
  rate_rise: [
    { increase_percent: 0.5, annual_rate_percent: 2.22, payment: 1726.75, payment_increase: 89.63 },
    { increase_percent: 1, annual_rate_percent: 2.72, payment: 1819.26, payment_increase: 182.14 },
    { increase_percent: 2, annual_rate_percent: 3.72, payment: 2012.81, payment_increase: 375.69 },
  ],
  yearly: [{ year: 1, paid: 19017, principal: 13425, interest: 5592, remaining_balance: 386575 }],
  references: [
    { code: 'ref.MasNotice632a', url: 'https://www.mas.gov.sg/regulation/notices/notice-632a' },
    { code: 'ref.MasNotice645', url: 'https://www.mas.gov.sg/regulation/notices/notice-645' },
  ],
  error: null,
};

const show = (props = {}, locale = 'en') =>
  render(
    <I18nProvider initialLocale={locale}>
      <ReportDocument
        report={REPORT}
        region="SG"
        scenario={DEFAULT_SCENARIO}
        sourceUrl="https://dengf.github.io/mortgage_calculator/"
        {...props}
      />
    </I18nProvider>,
  );

describe('what the document must always say', () => {
  // The page borrows its shape from disclosures a *lender* issues about an
  // offer it is making -- the CFPB's Loan Estimate, MAS's Residential
  // Property Loan Fact Sheet. The closer it gets to those, the more it has
  // to say what it is not. These are not decoration and not style.

  it('carries the watermark', () => {
    show();
    expect(document.querySelector('.report-watermark')).toHaveTextContent('For reference only');
  });

  it('says in the footer that it is not an offer from anyone', () => {
    show();
    const foot = document.querySelector('.report-foot');
    expect(foot).toHaveTextContent(/not a loan offer/i);
    expect(foot).toHaveTextContent(/no bank has seen these figures/i);
  });

  it('cites every authority the core handed it, with somewhere to check', () => {
    show();
    for (const reference of REPORT.references) {
      expect(screen.getByText(reference.url)).toBeInTheDocument();
    }
    expect(screen.getByText(/MAS Notice 645/)).toBeInTheDocument();
  });

  it('names the real disclosure it is not, so the two are told apart', () => {
    show();
    expect(screen.getByText(/Fact Sheet a bank must issue/i)).toBeInTheDocument();
  });

  it('links back to where the working can be inspected', () => {
    show();
    expect(screen.getByText('https://dengf.github.io/mortgage_calculator/')).toBeInTheDocument();
  });

  it.each(['zh-Hans', 'zh-Hant'])('says all of it in %s too', (locale) => {
    // A reader of the Chinese page gets a Chinese document, watermark and
    // disclaimer included -- these are the parts it would be worst to leave
    // in English.
    show({}, locale);
    expect(document.querySelector('.report-watermark').textContent).toMatch(/僅供參考|仅供参考/);
    expect(document.querySelector('.report-foot').textContent).toMatch(/並非貸款要約|并非贷款要约/);
  });
});

describe('what the document states about the loan', () => {
  it('answers "can this change?" for the rate and the instalment', () => {
    // The CFPB column. On a package it is the only honest way to state
    // either one, and it is the question a borrower does not know to ask.
    show();
    expect(screen.getByText(/steps up after 2 yr, to 1.720%/)).toBeInTheDocument();
    expect(screen.getByText(/rises after 2 yr, to S\$1,637.12/)).toBeInTheDocument();
  });

  it('describes the loan over time rather than as one figure', () => {
    show();
    expect(screen.getByText('Years 1–2')).toBeInTheDocument();
    expect(screen.getByText('Years 3–25')).toBeInTheDocument();
  });

  it('shows what a rise costs per month', () => {
    show();
    expect(screen.getByText('+2.00%')).toBeInTheDocument();
    expect(screen.getByText('S$375.69')).toBeInTheDocument();
  });

  it('marks a flat loan as not changing rather than hiding the question', () => {
    show({
      report: {
        ...REPORT,
        payment_after_reversion: null,
        lock_in_years: null,
        bands: [{ from_year: 1, to_year: 30, annual_rate_percent: 6.5, payment: 2528.27 }],
      },
    });
    expect(screen.getAllByText('No').length).toBe(5);
  });

  it('renders nothing rather than a blank document when the loan is invalid', () => {
    const { container } = show({ report: { error: 'nope', bands: [] } });
    expect(container.querySelector('.report')).toBeNull();
  });
});
