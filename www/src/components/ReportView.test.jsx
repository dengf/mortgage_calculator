import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import ReportView from './ReportView';
import { I18nProvider } from '../i18n';
import { scenarioBindings } from '../test/wasm';

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
  bands: [{ from_year: 1, to_year: 2, annual_rate_percent: 1.42, payment: 1584.75 }],
  rate_rise: [
    { increase_percent: 1, annual_rate_percent: 2.72, payment: 1819.26, payment_increase: 182.14 },
  ],
  schedule: [],
  yearly: [],
  frequency: 'monthly',
  references: [
    { code: 'ref.MasNotice645', url: 'https://www.mas.gov.sg/regulation/notices/notice-645' },
  ],
  error: null,
};

function mockWasm(overrides = {}) {
  return { ...scenarioBindings(), build_report: vi.fn(() => REPORT), ...overrides };
}

const show = (wasmModule = mockWasm()) =>
  render(
    <I18nProvider initialLocale="en">
      <ReportView wasmModule={wasmModule} region="SG" onScenarioChange={() => {}} />
    </I18nProvider>,
  );

let assigned;

beforeEach(() => {
  // `email()` navigates to a mailto: URL. jsdom would warn and do nothing;
  // capturing it is how the composed link gets inspected.
  assigned = [];
  delete window.location;
  window.location = {
    origin: 'https://example.test',
    pathname: '/calc/',
    set href(value) {
      assigned.push(value);
    },
  };
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe('sending the report', () => {
  it('does not open a mail client on the first click', async () => {
    // The one action on this page that points outward. A single click is
    // how a half-edited recipient list gets sent.
    show();
    await userEvent.type(screen.getByLabelText(/Email to/), 'jane@example.com');
    await userEvent.click(screen.getByRole('button', { name: /Open in mail app/ }));

    expect(assigned).toEqual([]);
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('spells out every recipient before it will go anywhere', async () => {
    // Not "2 recipients" — the count is exactly the phrasing that lets a
    // stale address through unnoticed.
    show();
    await userEvent.type(screen.getByLabelText(/Email to/), 'jane@example.com, bob@example.com');
    await userEvent.click(screen.getByRole('button', { name: /Open in mail app/ }));

    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveTextContent('jane@example.com');
    expect(dialog).toHaveTextContent('bob@example.com');
  });

  it('opens the mail client only after the confirmation is accepted', async () => {
    show();
    await userEvent.type(screen.getByLabelText(/Email to/), 'jane@example.com, bob@example.com');
    await userEvent.click(screen.getByRole('button', { name: /Open in mail app/ }));
    await userEvent.click(screen.getByRole('button', { name: /Yes, open my mail app/ }));

    await waitFor(() => expect(assigned).toHaveLength(1));
    expect(assigned[0]).toContain('mailto:jane%40example.com,bob%40example.com');
    expect(assigned[0]).toContain('subject=Mortgage%20illustration');
  });

  it('backs out without sending', async () => {
    show();
    await userEvent.type(screen.getByLabelText(/Email to/), 'jane@example.com');
    await userEvent.click(screen.getByRole('button', { name: /Open in mail app/ }));
    await userEvent.click(screen.getByRole('button', { name: /Not yet/ }));

    expect(assigned).toEqual([]);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('says in the body that the attachment has to be added by hand', async () => {
    // A mailto: link cannot carry one. Leaving that unsaid is how a client
    // gets an email promising a document that is not there.
    show();
    await userEvent.type(screen.getByLabelText(/Email to/), 'jane@example.com');
    await userEvent.click(screen.getByRole('button', { name: /Open in mail app/ }));
    await userEvent.click(screen.getByRole('button', { name: /Yes, open my mail app/ }));

    await waitFor(() => expect(assigned).toHaveLength(1));
    expect(decodeURIComponent(assigned[0])).toContain('attached as a PDF');
    expect(decodeURIComponent(assigned[0])).toContain('For reference only');
  });

  it('refuses to offer sending with nobody addressed', () => {
    show();
    expect(screen.getByRole('button', { name: /Open in mail app/ })).toBeDisabled();
  });
});

describe('downloading the report as CSV', () => {
  beforeEach(() => {
    global.URL.createObjectURL = vi.fn(() => 'blob:mock');
    global.URL.revokeObjectURL = vi.fn();
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});
  });

  it('carries every section of the printed document, not just the schedule', async () => {
    show();
    await userEvent.click(await screen.findByRole('button', { name: 'Download as CSV' }));

    expect(global.URL.createObjectURL).toHaveBeenCalled();
    const blob = global.URL.createObjectURL.mock.calls[0][0];
    const csv = await blob.text();

    expect(csv).toContain('Market,Singapore');

    // The terms table, including the "can this change" column the printed
    // document leads with -- a stepped package's rate is not the whole
    // answer without it.
    expect(csv).toContain('Home price,500000.00');
    expect(csv).toContain('Home loan amount,400000.00');
    expect(csv).toContain('Loan term,25 yr');
    expect(csv).toContain('Interest rate,1.420%,"Yes — steps up after 2 yr, to 1.720%"');
    expect(csv).toContain('Payment,1584.75,"Yes — rises after 2 yr, to 1637.12"');

    // The year-by-year rate bands and their totals.
    expect(csv).toContain('Period,Rate,Monthly instalment');
    expect(csv).toContain('Years 1–2,1.420%,1584.75');
    expect(csv).toContain('Total paid over the term,489880.62');
    expect(csv).toContain('Total interest,89880.62');
    expect(csv).toContain('Interest as a share of everything paid,18.347%');

    // The rate-rise stress test.
    expect(csv).toContain('Rise,Rate,Monthly instalment,More per payment');
    expect(csv).toContain('+1.00%,2.720%,1819.26,182.14');

    // The schedule itself, in whichever view is currently selected.
    expect(csv).toContain('Payment schedule');
    expect(csv).toContain('Payment,Year,Paid,Principal,Interest,Balance');

    // The disclaimer and its sources, same wording as the printed page's
    // footer, and appearing after the data the way the footer does.
    const scheduleIndex = csv.indexOf('Payment schedule');
    const disclaimerIndex = csv.indexOf('For reference only. This is an illustration');
    expect(disclaimerIndex).toBeGreaterThan(scheduleIndex);
    expect(csv).toContain('MAS and IRAS rules reflect published figures');
    expect(csv).toContain('Where the rules come from');
    expect(csv).toContain('MAS Notice 645');
    expect(csv).toContain('https://www.mas.gov.sg/regulation/notices/notice-645');
  });

  it('downloads the yearly view once that is what the panel shows', async () => {
    const wasmModule = mockWasm({
      build_report: vi.fn(() => ({
        ...REPORT,
        yearly: [
          {
            year: 1,
            paid: 19017,
            principal: 13424.16,
            interest: 5592.84,
            remaining_balance: 386575.84,
          },
        ],
      })),
    });
    show(wasmModule);

    await userEvent.click(screen.getByRole('button', { name: 'By year' }));
    await userEvent.click(await screen.findByRole('button', { name: 'Download as CSV' }));

    const blob = global.URL.createObjectURL.mock.calls[0][0];
    const csv = await blob.text();
    expect(csv).toContain('Yearly schedule');
    expect(csv).toContain('Year,Paid,Principal,Interest,Balance');
    expect(csv).toContain('1,19017.00,13424.16,5592.84,386575.84');
    // Not the payment-by-payment header, which the toggle just moved away from.
    expect(csv).not.toContain('Payment,Year,Paid,Principal,Interest,Balance');
  });
});
