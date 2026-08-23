import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import PaymentCalculator from './PaymentCalculator';
import { renderControlled } from '../test/controlled';
import { scenarioBindings } from '../test/wasm';
import { DEFAULT_SCENARIO } from '../scenario';

// PaymentCalculator receives wasmModule as a prop rather than importing the
// wasm-pack build directly, so it can be exercised here with a plain mock —
// no actual wasm compilation needed. SavedScenarios (rendered inside) also
// needs list_scenarios, since it calls it on mount.
function mockWasmModule(overrides = {}) {
  return {
    ...scenarioBindings(),
    calculate_payment: vi.fn(() => ({
      payment: 2528.27,
      total_periods: 360,
      total_paid: 910177.2,
      total_interest: 510177.2,
      error: null,
    })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    ...overrides,
  };
}

describe('PaymentCalculator', () => {
  it('renders the payment result computed by the wasm module', async () => {
    render(<PaymentCalculator wasmModule={mockWasmModule()} />);

    expect(await screen.findByText('$2,528.27')).toBeInTheDocument();
    expect(screen.getByText('$910,177.20')).toBeInTheDocument();
    expect(screen.getByText('$510,177.20')).toBeInTheDocument();
  });

  it('does not call into wasm while a required field is empty', async () => {
    // Clearing the loan amount is the first thing a visitor does — they have
    // to remove the default before typing their own. That used to reach
    // serde as `''` where an f64 was expected, and the resulting parse error
    // printed a wasm stack trace onto the page.
    const wasmModule = mockWasmModule();
    renderControlled(PaymentCalculator, { wasmModule });
    await waitFor(() => expect(wasmModule.calculate_payment).toHaveBeenCalled());

    const callsBefore = wasmModule.calculate_payment.mock.calls.length;
    await userEvent.clear(screen.getByDisplayValue('500,000'));

    expect(wasmModule.calculate_payment.mock.calls.length).toBe(callsBefore);
  });

  it('holds the last result rather than erroring when a field is cleared', async () => {
    renderControlled(PaymentCalculator, { wasmModule: mockWasmModule() });
    expect(await screen.findByText('$2,528.27')).toBeInTheDocument();

    await userEvent.clear(screen.getByDisplayValue('500,000'));

    // Figures stay on screen (dimmed) instead of vanishing or being replaced
    // by an error.
    expect(screen.getByText('$2,528.27')).toBeInTheDocument();
    expect(document.querySelector('.panel-results.stale')).not.toBeNull();
  });

  it('translates a parse failure instead of printing the raw error', async () => {
    const wasmModule = mockWasmModule({
      calculate_payment: vi.fn(() => ({
        error: "Some values are missing or aren't valid numbers. Check the fields above.",
        error_message: { code: 'err.badRequest', params: {} },
      })),
    });
    render(<PaymentCalculator wasmModule={wasmModule} />);

    const shown = await screen.findByText(/values are missing/i);
    expect(shown.textContent).not.toMatch(/wasm-function|JsValue|\.js:/);
  });

  it('passes the current field values to calculate_payment', async () => {
    const wasmModule = mockWasmModule();
    render(<PaymentCalculator wasmModule={wasmModule} />);

    await waitFor(() => expect(wasmModule.calculate_payment).toHaveBeenCalled());
    // The rate crosses as a shape, so a package that steps up can too. A
    // bare `annual_rate_percent` could only ever describe a flat loan.
    expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith({
      principal: 400000,
      rate: { kind: 'fixed', rate_percent: 6.5 },
      term_years: 30,
      frequency: 'monthly',
    });
  });

  it('recomputes when an input changes', async () => {
    const wasmModule = mockWasmModule();
    renderControlled(PaymentCalculator, { wasmModule });
    await waitFor(() => expect(wasmModule.calculate_payment).toHaveBeenCalled());

    // NumberField's label wraps both the label text and the suffix span
    // ("%"), so the accessible name is "Rate %", not an exact match of the
    // label text alone. "Interest rate" now names the group of shapes above
    // it -- fixed, steps up, floating -- rather than a single input.
    const rateInput = screen.getByLabelText(/^Rate/);
    await userEvent.clear(rateInput);
    await userEvent.type(rateInput, '7');

    await waitFor(() =>
      expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith(
        expect.objectContaining({ rate: { kind: 'fixed', rate_percent: 7 } }),
      ),
    );
  });

  it('shows the error message instead of stale results when the wasm module reports one', async () => {
    const wasmModule = mockWasmModule({
      calculate_payment: vi.fn(() => ({
        error: 'loan term is unreasonably long, got 5200001 payment periods',
      })),
    });
    render(<PaymentCalculator wasmModule={wasmModule} />);

    expect(await screen.findByText(/unreasonably long/)).toBeInTheDocument();
    // The input fields' own "$"/"%" suffixes are always present; what
    // must NOT render is a stale results grid from a previous calculation.
    expect(document.querySelector('.stat-grid')).not.toBeInTheDocument();
  });

  it('renders nothing for the result panel until the wasm module is ready', () => {
    render(<PaymentCalculator wasmModule={null} />);
    expect(screen.queryByText('Payment')).not.toBeInTheDocument();
  });
});

describe('PaymentCalculator, on a package that steps up', () => {
  // The shape of every Singapore home loan: a promotional spread over SORA
  // for two or three years, then a higher one for the remaining twenty-odd.
  const SG_PACKAGE = {
    ...DEFAULT_SCENARIO,
    rate: {
      kind: 'reverting',
      baseRatePercent: 1.12,
      initialSpreadPercent: 0.3,
      initialYears: 2,
      thereafterSpreadPercent: 0.6,
    },
    termYears: 25,
  };

  function packageModule(overrides = {}) {
    return {
      ...scenarioBindings(),
      calculate_payment: vi.fn(() => ({
        payment: 1584.75,
        payment_after_reversion: 1800.53,
        total_periods: 300,
        total_paid: 534979.6,
        total_interest: 134979.6,
        error: null,
      })),
      list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
      ...overrides,
    };
  }

  it('sends the whole package across, not the promotional rate alone', async () => {
    const wasmModule = packageModule();
    renderControlled(PaymentCalculator, { wasmModule }, SG_PACKAGE);

    await waitFor(() => expect(wasmModule.calculate_payment).toHaveBeenCalled());
    expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith({
      principal: 400000,
      rate: {
        kind: 'reverting',
        base_rate_percent: 1.12,
        initial_spread_percent: 0.3,
        initial_years: 2,
        thereafter_spread_percent: 0.6,
      },
      term_years: 25,
      frequency: 'monthly',
    });
  });

  it('names the instalment the lock-in ends on, beside the one it opens on', async () => {
    // The headline figure expires after two years of a twenty-five year
    // loan. Showing it alone is how a buyer budgets for the wrong number.
    renderControlled(PaymentCalculator, { wasmModule: packageModule() }, SG_PACKAGE);

    expect(await screen.findByText('$1,584.75')).toBeInTheDocument();
    expect(screen.getByText('then $1,800.53')).toBeInTheDocument();
  });

  it('assesses Singapore servicing on the package, not on the teaser', async () => {
    // MAS Notice 645 para 6(b) assesses at the higher of 4% and the
    // *thereafter* rate. This panel used to hand `calculate_singapore` a
    // single rate, which could only ever be the promotional one — passing
    // borrowers a servicing check their own bank would fail them on.
    const wasmModule = packageModule({
      calculate_singapore: vi.fn(() => ({ warnings: [], warning_codes: [] })),
    });
    renderControlled(PaymentCalculator, { wasmModule, region: 'SG' }, SG_PACKAGE);

    await waitFor(() => expect(wasmModule.calculate_singapore).toHaveBeenCalled());
    expect(wasmModule.calculate_singapore).toHaveBeenLastCalledWith(
      expect.objectContaining({
        rate: expect.objectContaining({
          kind: 'reverting',
          thereafter_spread_percent: 0.6,
        }),
      }),
    );
  });

  it('lets the buyer edit the spread it steps up to', async () => {
    // Every field of a package is negotiated, and the thereafter spread is
    // the one that decides most of the interest.
    const wasmModule = packageModule();
    renderControlled(PaymentCalculator, { wasmModule }, SG_PACKAGE);
    await waitFor(() => expect(wasmModule.calculate_payment).toHaveBeenCalled());

    const spread = screen.getByLabelText(/Thereafter spread/);
    await userEvent.clear(spread);
    await userEvent.type(spread, '1.5');

    await waitFor(() =>
      expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith(
        expect.objectContaining({
          rate: expect.objectContaining({ thereafter_spread_percent: 1.5 }),
        }),
      ),
    );
  });

  it('loads a scenario saved before a rate was a shape', async () => {
    // Those records store `rate: 6.5`. They describe a loan the user really
    // entered, and dropping them would lose it.
    const wasmModule = packageModule({
      load_scenario: vi.fn(async () => ({
        scenario: {
          id: '1',
          inputs_json: JSON.stringify({
            homePrice: 500000,
            downPayment: 100000,
            rate: 6.5,
            termYears: 30,
            frequency: 'monthly',
          }),
        },
        error: null,
      })),
      list_scenarios: vi.fn(async () => ({
        scenarios: [
          {
            id: '1',
            name: 'Old record',
            calculator: 'payment',
            created_at: 0,
            inputs_json: JSON.stringify({
              homePrice: 500000,
              downPayment: 100000,
              rate: 6.5,
              termYears: 30,
              frequency: 'monthly',
            }),
          },
        ],
        error: null,
      })),
    });
    renderControlled(PaymentCalculator, { wasmModule }, SG_PACKAGE);

    await userEvent.click(await screen.findByRole('button', { name: 'Load' }));

    await waitFor(() =>
      expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith(
        expect.objectContaining({ rate: { kind: 'fixed', rate_percent: 6.5 } }),
      ),
    );
  });
});
