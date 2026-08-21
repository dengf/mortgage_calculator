import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import UnitedStatesPanel from './UnitedStatesPanel';

const inputs = {
  home_price: 500000,
  zip: '90210',
  pmi_rate_percent: 0.75,
  use_tax_deduction: false,
  marginal_tax_rate_percent: 24,
};

const result = {
  loan_type: 'Conforming',
  property_tax_rate_percent: 0.7,
  monthly_property_tax: 291.67,
  down_payment: 100000,
  down_payment_percent: 20,
  pmi_required: false,
  monthly_pmi: 0,
  monthly_piti: 2819.94,
  monthly_tax_savings: null,
  net_monthly_cost: null,
  error: null,
};

describe('UnitedStatesPanel', () => {
  it('shows the PITI total, not just principal and interest', () => {
    render(<UnitedStatesPanel inputs={inputs} onChange={() => {}} result={result} />);

    expect(screen.getByText('Monthly PITI')).toBeInTheDocument();
    expect(screen.getByText('$2,819.94')).toBeInTheDocument();
  });

  it('says PMI is not required at 20% down and shows no premium', () => {
    render(<UnitedStatesPanel inputs={inputs} onChange={() => {}} result={result} />);

    expect(screen.getByText('PMI (not required)')).toBeInTheDocument();
    expect(screen.queryByText(/PMI applies below 20% down/)).not.toBeInTheDocument();
  });

  it('tells the user what would remove PMI when it applies', () => {
    render(
      <UnitedStatesPanel
        inputs={{ ...inputs, home_price: 440000 }}
        onChange={() => {}}
        result={{ ...result, pmi_required: true, monthly_pmi: 250, down_payment_percent: 9.1 }}
      />,
    );

    expect(screen.getByText('PMI (required)')).toBeInTheDocument();
    // 20% of 440,000
    expect(screen.getByText(/\$88,000\.00 removes it/)).toBeInTheDocument();
  });

  it('explains an unrecognized ZIP rather than silently taxing at zero', () => {
    render(
      <UnitedStatesPanel
        inputs={{ ...inputs, zip: '00000' }}
        onChange={() => {}}
        result={{ ...result, property_tax_rate_percent: null, monthly_property_tax: 0 }}
      />,
    );

    expect(screen.getByText(/doesn't match a state/)).toBeInTheDocument();
  });

  it('hides the deduction figures until the estimate is switched on', () => {
    const { rerender } = render(
      <UnitedStatesPanel inputs={inputs} onChange={() => {}} result={result} />,
    );
    expect(screen.queryByText('Net monthly cost')).not.toBeInTheDocument();

    rerender(
      <UnitedStatesPanel
        inputs={{ ...inputs, use_tax_deduction: true }}
        onChange={() => {}}
        result={{ ...result, monthly_tax_savings: 520, net_monthly_cost: 2299.94 }}
      />,
    );
    expect(screen.getByText('Net monthly cost')).toBeInTheDocument();
    expect(screen.getByText('$2,299.94')).toBeInTheDocument();
  });

  it('surfaces a jumbo classification', () => {
    render(
      <UnitedStatesPanel
        inputs={inputs}
        onChange={() => {}}
        result={{ ...result, loan_type: 'Jumbo' }}
      />,
    );
    expect(screen.getByText('Jumbo')).toBeInTheDocument();
  });

  it('strips non-digits from the ZIP rather than sending them to the binding', async () => {
    const onChange = vi.fn();
    render(<UnitedStatesPanel inputs={{ ...inputs, zip: '' }} onChange={onChange} result={result} />);

    await userEvent.type(screen.getByLabelText('ZIP code'), '9a');

    // `inputs` is a fixed prop here, so every keystroke is applied to the
    // same empty starting value rather than accumulating — asserting on the
    // last call would be asserting on that, not on the stripping. The real
    // invariant is that a non-digit never reaches the binding.
    expect(onChange).toHaveBeenCalled();
    expect(onChange.mock.calls.every(([next]) => /^\d*$/.test(next.zip))).toBe(true);
  });
});
