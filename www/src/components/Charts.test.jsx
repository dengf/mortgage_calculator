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
        yearsPlotted={30}
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
        yearsPlotted={30}
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
        yearsPlotted={30}
        formatMoney={formatMoney}
      />,
    );

    expect(screen.getByText('Year 0')).toBeInTheDocument();
    expect(screen.getByText('Year 30')).toBeInTheDocument();
  });

  it('labels a part-year span to one decimal, and a whole one without it', () => {
    // 233 months of a 30-year loan. That the span shown is the *plotted*
    // schedule rather than the nominal term is now AmortizationSchedule's
    // job -- it is asserted there, since this component is handed the span.
    const { rerender } = render(
      <BalanceChart
        rows={schedule(233, 400000, 3028)}
        principal={400000}
        yearsPlotted={19.416666}
        formatMoney={formatMoney}
      />,
    );
    expect(screen.getByText('Year 19.4')).toBeInTheDocument();

    rerender(
      <BalanceChart
        rows={schedule(360, 400000, 1417)}
        principal={400000}
        yearsPlotted={30}
        formatMoney={formatMoney}
      />,
    );
    // Not "Year 30.0": a whole span reads as a whole number.
    expect(screen.getByText('Year 30')).toBeInTheDocument();
  });

  it('renders nothing without rows', () => {
    const { container } = render(
      <BalanceChart rows={[]} principal={400000} yearsPlotted={30} formatMoney={formatMoney} />,
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
        interestSharePercent={56.05}
        formatMoney={formatMoney}
      />,
    );

    // 510,177 / 910,177 = 56%
    expect(screen.getByText(/Interest is 56% of everything you pay\./)).toBeInTheDocument();
  });

  it('sizes the two segments in proportion', () => {
    const { container } = render(
      <PrincipalInterestSplit
        principal={300000}
        totalInterest={100000}
        interestSharePercent={25}
        formatMoney={formatMoney}
      />,
    );

    expect(container.querySelector('.split-bar-principal')).toHaveStyle({ width: '75%' });
    expect(container.querySelector('.split-bar-interest')).toHaveStyle({ width: '25%' });
  });

  it('renders nothing when the core reported no share', () => {
    // Absent, not zero: a bar at 0% asserts none of the money is interest,
    // rather than that there is no money.
    const { container } = render(
      <PrincipalInterestSplit
        principal={0}
        totalInterest={0}
        interestSharePercent={null}
        formatMoney={formatMoney}
      />,
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('draws a real bar for a genuinely interest-free loan', () => {
    const { container } = render(
      <PrincipalInterestSplit
        principal={12000}
        totalInterest={0}
        interestSharePercent={0}
        formatMoney={formatMoney}
      />,
    );

    expect(container.querySelector('.split-bar-principal')).toHaveStyle({ width: '100%' });
    expect(container.querySelector('.split-bar-interest')).toHaveStyle({ width: '0%' });
  });
});
