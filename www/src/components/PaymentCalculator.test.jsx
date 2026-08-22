import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import PaymentCalculator from './PaymentCalculator';
import { renderControlled } from '../test/controlled';

// PaymentCalculator receives wasmModule as a prop rather than importing the
// wasm-pack build directly, so it can be exercised here with a plain mock —
// no actual wasm compilation needed. SavedScenarios (rendered inside) also
// needs list_scenarios, since it calls it on mount.
function mockWasmModule(overrides = {}) {
  return {
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
    expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith({
      principal: 400000,
      annual_rate_percent: 6.5,
      term_years: 30,
      frequency: 'monthly',
    });
  });

  it('recomputes when an input changes', async () => {
    const wasmModule = mockWasmModule();
    renderControlled(PaymentCalculator, { wasmModule });
    await waitFor(() => expect(wasmModule.calculate_payment).toHaveBeenCalled());

    // NumberField's label wraps both the label text and the suffix span
    // ("%"), so the accessible name is "Interest rate %", not an exact
    // match of the label text alone.
    const rateInput = screen.getByLabelText(/Interest rate/);
    await userEvent.clear(rateInput);
    await userEvent.type(rateInput, '7');

    await waitFor(() =>
      expect(wasmModule.calculate_payment).toHaveBeenLastCalledWith(
        expect.objectContaining({ annual_rate_percent: 7 }),
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
