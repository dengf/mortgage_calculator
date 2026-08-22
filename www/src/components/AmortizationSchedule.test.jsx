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

// The yearly view is what the tab shows by default, and it had no coverage
// at all. It used to be summed in the component with f64 reduces over the
// period rows; it now comes from the same call that produced them.
describe('AmortizationSchedule yearly summary', () => {
  const row = (period, balance) => ({
    period,
    payment: 1000,
    extra_payment: 0,
    principal_portion: 400,
    interest_portion: 600,
    remaining_balance: balance,
  });

  function scheduleWasm(schedule) {
    return {
      ...scenarioBindings(),
      calculate_amortization_schedule: vi.fn(() => schedule),
      calculate_extra_payment_impact: vi.fn(() => ({ error: null })),
      list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    };
  }

  it('renders the years the module returned', async () => {
    renderControlled(AmortizationSchedule, {
      wasmModule: scheduleWasm({
        rows: [row(1, 99600), row(2, 99200)],
        yearly: [
          { year: 1, paid: 12000, principal: 4800, interest: 7200, remaining_balance: 95200 },
          { year: 2, paid: 12000, principal: 4800, interest: 7200, remaining_balance: 90400 },
        ],
        error: null,
      }),
      region: 'US',
    });

    expect(await screen.findByText('$95,200.00')).toBeInTheDocument();
    expect(screen.getByText('$90,400.00')).toBeInTheDocument();
  });

  it('shows no yearly table when the module returned no years', async () => {
    renderControlled(AmortizationSchedule, {
      wasmModule: scheduleWasm({ rows: [], yearly: [], error: null }),
      region: 'US',
    });

    // Previously the component would have derived an empty list itself; now
    // an absent grouping must not render an empty table either.
    expect(await screen.findByDisplayValue('500,000')).toBeInTheDocument();
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
  });
});
