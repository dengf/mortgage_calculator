import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import PaymentCalculator from './PaymentCalculator';
import { renderControlled } from '../test/controlled';
import { scenarioBindings } from '../test/wasm';

// Driven through PaymentCalculator rather than in isolation: the deposit
// field is controlled, and the percentage it shows is computed a layer up.
// Rendering it alone with fixed props would make every one of these pass
// against a field that cannot actually write anything back.
function mockWasmModule(overrides = {}) {
  return {
    ...scenarioBindings(),
    calculate_payment: vi.fn(() => ({
      payment: 0,
      total_periods: 0,
      total_paid: 0,
      total_interest: 0,
      error: null,
    })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    ...overrides,
  };
}

// Selected by class rather than accessible name: these buttons sit inside
// the field's <label>, so each one's computed name is the whole label text
// ("Down payment% 20 %") rather than the unit it selects. That is worth
// fixing on its own; it is not this change.
const unitToggles = () => document.querySelectorAll('.field-unit');
const amountToggle = () => unitToggles()[0];
const percentToggle = () => unitToggles()[1];

describe('DownPaymentField', () => {
  it('shows the deposit as a share of price', async () => {
    render(<PaymentCalculator wasmModule={mockWasmModule()} />);

    // 100,000 of 500,000, from the default scenario.
    expect(await screen.findByText('20.0% of price')).toBeInTheDocument();
  });

  it('asks the module what a typed percentage comes to', async () => {
    const user = userEvent.setup();
    // A spy over the stub, not a fixed return: the field is controlled, so a
    // constant answer would feed a wrong percentage straight back into the
    // input and the next keystroke would append to it.
    const down_payment_for_percent = vi.fn(scenarioBindings().down_payment_for_percent);
    renderControlled(PaymentCalculator, {
      wasmModule: mockWasmModule({ down_payment_for_percent }),
    });

    await user.click(percentToggle());
    const input = screen.getByDisplayValue('20');
    await user.clear(input);
    await user.type(input, '15');

    // The conversion is the module's, including its rounding -- the
    // component never multiplies the price by anything itself.
    expect(down_payment_for_percent).toHaveBeenLastCalledWith({
      home_price: 500000,
      percent: 15,
    });
    await user.click(amountToggle());
    expect(screen.getByDisplayValue('75,000')).toBeInTheDocument();
  });

  it('says nothing about a percentage when there is no price', async () => {
    const user = userEvent.setup();
    renderControlled(PaymentCalculator, { wasmModule: mockWasmModule() });

    const price = await screen.findByDisplayValue('500,000');
    await user.clear(price);

    // Not "0.0% of price": with no price entered there is no share to state.
    expect(screen.queryByText(/of price/)).not.toBeInTheDocument();
  });
});
