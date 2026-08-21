import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import SingaporePanel from './SingaporePanel';

const inputs = {
  home_price: 1000000,
  gross_monthly_income: 12000,
  other_monthly_debts: 0,
  cpf_oa_available: 1500,
  residency: 'Citizen',
  property_count: '1st',
  is_hdb_or_ec: false,
  loan_type: 'Bank Loan',
};

const result = {
  tdsr_ratio_percent: 21.1,
  tdsr_exceeded: false,
  tdsr_near_limit: false,
  msr_ratio_percent: null,
  msr_exceeded: false,
  msr_near_limit: false,
  cpf_used: 1500,
  cash_required: 1028.27,
  bsd: 24600,
  absd: 0,
  upfront_total: 24600,
  down_payment: 600000,
  total_cash_required: 624600,
  loan_type_warning: null,
  warnings: [],
  error: null,
};

describe('SingaporePanel', () => {
  it('shows SGD with an explicit S$ so it cannot be read as USD', () => {
    render(<SingaporePanel inputs={inputs} onChange={() => {}} result={result} />);

    expect(screen.getByText('S$624,600.00')).toBeInTheDocument();
  });

  it('hides MSR for private property, since it only applies to HDB/EC', () => {
    render(<SingaporePanel inputs={inputs} onChange={() => {}} result={result} />);

    expect(screen.queryByText(/MSR/)).not.toBeInTheDocument();
    expect(screen.getByText(/TDSR/)).toBeInTheDocument();
  });

  it('marks a breached ratio distinctly from one merely near the limit', () => {
    const { rerender } = render(
      <SingaporePanel
        inputs={inputs}
        onChange={() => {}}
        result={{ ...result, tdsr_ratio_percent: 52, tdsr_near_limit: true }}
      />,
    );
    expect(screen.getByText('52.0%')).toHaveClass('near');

    rerender(
      <SingaporePanel
        inputs={inputs}
        onChange={() => {}}
        result={{ ...result, tdsr_ratio_percent: 63.2, tdsr_exceeded: true }}
      />,
    );
    expect(screen.getByText('63.2%')).toHaveClass('breach');
  });

  it('surfaces MAS limit breaches as visible warnings', () => {
    render(
      <SingaporePanel
        inputs={inputs}
        onChange={() => {}}
        result={{ ...result, warnings: ['Exceeds MAS TDSR limit (55%).'] }}
      />,
    );

    expect(screen.getByText('Exceeds MAS TDSR limit (55%).')).toBeInTheDocument();
  });

  it('warns when an HDB loan is chosen for a property that cannot have one', () => {
    render(
      <SingaporePanel
        inputs={{ ...inputs, loan_type: 'HDB Loan' }}
        onChange={() => {}}
        result={{ ...result, loan_type_warning: 'HDB loans are only available for HDB flats/ECs.' }}
      />,
    );

    expect(screen.getByText(/HDB loans are only available/)).toBeInTheDocument();
  });

  it('reports the residency change rather than mutating its own inputs', async () => {
    const onChange = vi.fn();
    render(<SingaporePanel inputs={inputs} onChange={onChange} result={result} />);

    await userEvent.selectOptions(screen.getByLabelText('Residency'), 'Foreigner');

    expect(onChange).toHaveBeenCalledWith({ ...inputs, residency: 'Foreigner' });
  });
});
