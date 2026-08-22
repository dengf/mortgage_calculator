import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { BalanceChart, PrincipalInterestSplit } from './Charts';

const formatMoney = (n) =>
  n == null ? '—' : n.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

/** A schedule that pays `principal` down linearly, with flat interest. */
function schedule(periods, principal, interestPerPeriod) {
  return Array.from({ length: periods }, (_, i) => ({
    period: i + 1,
    payment: principal / periods + interestPerPeriod,
    principal_portion: principal / periods,
    interest_portion: interestPerPeriod,
    remaining_balance: principal - (principal / periods) * (i + 1),
  }));
}

describe('BalanceChart', () => {
  it('downsamples long schedules instead of emitting a node per row', () => {
    const { container } = render(
      <BalanceChart
        rows={schedule(1560, 400000, 300)}
        principal={400000}
        periodsPerYear={12}
        formatMoney={formatMoney}
      />,
    );

    const balancePath = container.querySelectorAll('.chart-svg path')[1];
    const nodes = balancePath.getAttribute('d').split(/[ML]/).length - 1;

    // ~120 samples regardless of the 1560 input rows.
    expect(nodes).toBeLessThan(130);
    expect(nodes).toBeGreaterThan(100);
  });

  it('scales both series against a shared maximum so the crossover is honest', () => {
    render(
      <BalanceChart
        rows={schedule(360, 400000, 1417)}
        principal={400000}
        periodsPerYear={12}
        formatMoney={formatMoney}
      />,
    );

    // Total interest (360 * 1417 = 510,120) exceeds principal, so it sets
    // the shared scale.
    expect(screen.getByText(/\$510,120\.00/)).toBeInTheDocument();
  });

  it('labels the horizontal axis from the schedule it actually plotted', () => {
    render(
      <BalanceChart
        rows={schedule(360, 400000, 1417)}
        principal={400000}
        periodsPerYear={12}
        formatMoney={formatMoney}
      />,
    );

    expect(screen.getByText('Year 0')).toBeInTheDocument();
    expect(screen.getByText('Year 30')).toBeInTheDocument();
  });

  it('ends the axis at the real payoff when extra payments retire the loan early', () => {
    // A 30-year loan paid off in 233 months. The axis used to read "Year 30"
    // while the curve hit zero at the right edge — mislabelling precisely the
    // fact the user added extra payments to see.
    render(
      <BalanceChart
        rows={schedule(233, 400000, 3028)}
        principal={400000}
        periodsPerYear={12}
        formatMoney={formatMoney}
      />,
    );

    expect(screen.getByText('Year 19.4')).toBeInTheDocument();
    expect(screen.queryByText('Year 30')).not.toBeInTheDocument();
  });

  it('renders nothing without rows', () => {
    const { container } = render(
      <BalanceChart rows={[]} principal={400000} periodsPerYear={12} formatMoney={formatMoney} />,
    );
    expect(container).toBeEmptyDOMElement();
  });
});

describe('PrincipalInterestSplit', () => {
  it('states interest as a share of everything paid', () => {
    render(
      <PrincipalInterestSplit
        principal={400000}
        totalInterest={510177.2}
        formatMoney={formatMoney}
      />,
    );

    // 510,177 / 910,177 = 56%
    expect(screen.getByText(/Interest is 56% of everything you pay\./)).toBeInTheDocument();
  });

  it('sizes the two segments in proportion', () => {
    const { container } = render(
      <PrincipalInterestSplit principal={300000} totalInterest={100000} formatMoney={formatMoney} />,
    );

    expect(container.querySelector('.split-bar-principal')).toHaveStyle({ width: '75%' });
    expect(container.querySelector('.split-bar-interest')).toHaveStyle({ width: '25%' });
  });

  it('renders nothing when there is nothing to split', () => {
    const { container } = render(
      <PrincipalInterestSplit principal={0} totalInterest={0} formatMoney={formatMoney} />,
    );
    expect(container).toBeEmptyDOMElement();
  });
});
