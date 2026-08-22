import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AmortizationSchedule from './AmortizationSchedule';
import { renderControlled } from '../test/controlled';
import { scenarioBindings } from '../test/wasm';

// Same defect as ComparisonView's: `onLoad` called setters that stopped
// existing when price, rate and term moved into the shared scenario. Every
// Load click threw a ReferenceError and took the tab down. This shipped, and
// the tab had no test file at all to catch it.
function mockWasm(inputs) {
  return {
    ...scenarioBindings(),
    calculate_amortization_schedule: vi.fn(() => ({ rows: [], error: null })),
    calculate_extra_payment_impact: vi.fn(() => ({ error: null })),
    list_scenarios: vi.fn(async () => ({
      scenarios: [{ id: 's1', name: 'Saved run', created_at: Date.now() }],
      error: null,
    })),
    load_scenario: vi.fn(async () => ({
      scenario: { inputs_json: JSON.stringify(inputs) },
      error: null,
    })),
  };
}

describe('AmortizationSchedule saved scenarios', () => {
  it('restores price, deposit, rate and term from a saved run', async () => {
    const user = userEvent.setup();
    renderControlled(AmortizationSchedule, {
      wasmModule: mockWasm({
        homePrice: 900000,
        downPayment: 180000,
        rate: 5.25,
        termYears: 15,
        frequency: 'monthly',
        extraPayment: 250,
      }),
      region: 'US',
    });

    await user.click(await screen.findByRole('button', { name: 'Load' }));

    expect(await screen.findByDisplayValue('900,000')).toBeInTheDocument();
    expect(screen.getByDisplayValue('180,000')).toBeInTheDocument();
    expect(screen.getByDisplayValue('5.25')).toBeInTheDocument();
    expect(screen.getByDisplayValue('15')).toBeInTheDocument();
  });

  it('restores a record saved before the scenario held a deposit', async () => {
    const user = userEvent.setup();
    renderControlled(AmortizationSchedule, {
      wasmModule: mockWasm({
        principal: 250000,
        rate: 6.5,
        termYears: 30,
        frequency: 'monthly',
        extraPayment: 0,
      }),
      region: 'US',
    });

    await user.click(await screen.findByRole('button', { name: 'Load' }));

    // The split was never recorded, so the saved loan becomes the price and
    // the deposit is zero -- no figure invented on the user's behalf. Asserted
    // through the derived loan amount rather than the deposit input, since a
    // bare "0" also matches the extra-payment field.
    expect(await screen.findByDisplayValue('250,000')).toBeInTheDocument();
    expect(document.querySelector('.field-derived-value').textContent).toBe('$250,000.00');
  });
});
