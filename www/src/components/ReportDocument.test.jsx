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
  frequency: 'monthly',
  schedule: [
    {
      period: 1,
      year: 1,
      paid: 1584.75,
      principal: 1111.42,
      interest: 473.33,
      remaining_balance: 398888.58,
    },
    {
      period: 13,
      year: 2,
      paid: 1584.75,
      principal: 1112.73,
      interest: 472.02,
      remaining_balance: 397775.85,
    },
  ],
  yearly: [{ year: 1, paid: 19017, principal: 13425, interest: 5592, remaining_balance: 386575 }],
  // The package is quoted over 3M SORA, so every figure above is exact
  // given a number nobody in this app controls.
  floating_base_percent: 1.12,
  rate_note: {
    code: 'note.floatingBase',
    params: { base: '1.12' },
    text: 'These figures assume the base rate stays at 1.12%.',
  },
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

  it('says it again right after the header, not only in the footer', () => {
    // The footer sits after a schedule that can run to hundreds of rows --
    // many printed pages away. A reader who only keeps the first page must
    // still see what this document is not.
    show();
    const lede = document.querySelector('.report-lede-disclaimer');
    expect(lede).toHaveTextContent(/not a loan offer/i);
    expect(lede.compareDocumentPosition(document.querySelector('.report-foot'))).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
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

describe('what the document says was assumed', () => {
  // The tables read as a quotation: exact instalments, an exact total, a
  // year-by-year schedule. They are exact *given* a benchmark that was held
  // still, and a reader who is not told that has been shown a projection
  // wearing a quotation's clothes. MAS makes a bank admit the same thing on
  // a Notice 632A fact sheet.

  it('names the figure that was held still, beside the terms it produced', () => {
    show();
    const note = document.querySelector('.report-assumption');
    expect(note).toHaveTextContent('1.12%');
    // In the terms section, not filed at the bottom with the citations.
    expect(note.closest('.report-section')).toContainElement(
      screen.getByRole('heading', { name: 'Loan terms' }),
    );
  });

  it('answers the rate column with the benchmark as well as the schedule', () => {
    show();
    // "Steps up after 2 yr, to 1.720%" is true and incomplete: the 1.720%
    // moves too.
    expect(document.querySelector('.report-can-change')).toHaveTextContent(
      'whenever the benchmark moves',
    );
  });

  it('answers the payment column the same way', () => {
    show();
    // The instalment moves for the same reason the rate does. Saying it of
    // the rate alone reads as though the payment settles at the thereafter
    // figure and stays there for twenty-three years.
    const rows = [...document.querySelectorAll('.report-terms tbody tr')];
    const payment = rows.find((r) => /1,584/.test(r.textContent));
    expect(payment.querySelector('.report-can-change')).toHaveTextContent(
      'whenever the benchmark moves',
    );
  });

  it('says nothing when the rates are contractual', () => {
    show({ report: { ...REPORT, floating_base_percent: null, rate_note: null } });
    expect(document.querySelector('.report-assumption')).toBeNull();
    expect(document.querySelector('.report-can-change')).toBeNull();
  });

  it('states the assumption in the language the document is printed in', () => {
    show({}, 'zh-Hans');
    expect(document.querySelector('.report-assumption').textContent).toContain('基准利率');
  });
});

describe('the schedule and the cadence it is paid on', () => {
  it('prints a row per payment, numbered by payment', () => {
    show();
    const rows = [...document.querySelectorAll('.report-schedule tbody tr')];
    expect(rows).toHaveLength(2);
    expect(rows[0].querySelector('th')).toHaveTextContent('1');
    expect(rows[1].querySelector('th')).toHaveTextContent('13');
  });

  it('tells each payment which year it falls in', () => {
    // Payment 13 of a monthly loan is year 2, and nobody divides while
    // reading -- on a bi-weekly loan the divisor is 26 and nobody guesses.
    show();
    const rows = [...document.querySelectorAll('.report-schedule tbody tr')];
    expect(rows[0].cells[1]).toHaveTextContent('1');
    expect(rows[1].cells[1]).toHaveTextContent('2');
  });

  it('reads the year from the report rather than dividing', () => {
    // The cadence decides it, and the cadence lives in Rust. A component
    // doing `period / 12` would be right for one of the three frequencies.
    show({
      report: {
        ...REPORT,
        frequency: 'biweekly',
        schedule: [
          { period: 27, year: 2, paid: 731, principal: 512, interest: 219, remaining_balance: 1 },
        ],
      },
    });
    expect(document.querySelector('.report-schedule tbody tr').cells[1]).toHaveTextContent('2');
  });

  it('prints the yearly roll-up instead when that is what was asked for', () => {
    show({ granularity: 'year' });
    const rows = [...document.querySelectorAll('.report-schedule tbody tr')];
    expect(rows).toHaveLength(1);
    expect(screen.getByRole('heading', { name: 'Yearly schedule' })).toBeInTheDocument();
    // No payment-number column to leave a stray header behind.
    expect(document.querySelectorAll('.report-schedule thead th')).toHaveLength(5);
  });

  it('heads the payment view with six columns, the yearly with five', () => {
    show();
    expect(document.querySelectorAll('.report-schedule thead th')).toHaveLength(6);
    expect(screen.getByRole('heading', { name: 'Payment schedule' })).toBeInTheDocument();
  });

  it('names the instalment column after the cadence, not "monthly"', () => {
    // The figure is a fortnightly payment. Calling it a monthly one is not
    // a wording slip -- it is a wrong number on a document a banker hands a
    // client.
    show({ report: { ...REPORT, frequency: 'biweekly' } });
    expect(screen.getAllByText('Bi-weekly instalment').length).toBeGreaterThan(0);
    expect(screen.queryByText('Monthly instalment')).toBeNull();
  });

  it('says monthly when it is monthly', () => {
    show();
    expect(screen.getAllByText('Monthly instalment').length).toBeGreaterThan(0);
  });

  it('falls back to monthly rather than printing a blank column head', () => {
    // An older `pkg/` built before the cadence crossed the boundary.
    show({ report: { ...REPORT, frequency: undefined } });
    expect(screen.getAllByText('Monthly instalment').length).toBeGreaterThan(0);
  });

  it('states the rise per payment without claiming a period', () => {
    show({ report: { ...REPORT, frequency: 'weekly' } });
    expect(screen.getByText('More per payment')).toBeInTheDocument();
  });
});
