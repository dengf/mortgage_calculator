import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';
import { currencySymbol, makeFormatMoney } from '../currency';


export default function RefinanceCalculator({ wasmModule, region }) {
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);
  const [currentBalance, setCurrentBalance] = useState(300000);
  const [currentRate, setCurrentRate] = useState(7.5);
  const [remainingPeriods, setRemainingPeriods] = useState(300);
  const [newRate, setNewRate] = useState(6.0);
  const [newTermYears, setNewTermYears] = useState(30);
  const [closingCosts, setClosingCosts] = useState(6000);

  const result = useMemo(() => {
    if (!wasmModule) return null;
    return wasmModule.calculate_refinance({
      current_balance: currentBalance,
      current_annual_rate_percent: currentRate,
      remaining_periods: remainingPeriods,
      new_annual_rate_percent: newRate,
      new_term_years: newTermYears,
      closing_costs: closingCosts,
      frequency: 'monthly',
    });
  }, [wasmModule, currentBalance, currentRate, remainingPeriods, newRate, newTermYears, closingCosts]);

  return (
    <section className="panel">
      <div className="panel-form">
        <NumberField label="Current balance" value={currentBalance} onChange={setCurrentBalance} suffix={money} min={0} />
        <NumberField label="Current rate" value={currentRate} onChange={setCurrentRate} suffix="%" min={0} />
        <NumberField
          label="Payments remaining"
          value={remainingPeriods}
          onChange={setRemainingPeriods}
          suffix="months"
          min={1}
          step="1"
        />
        <NumberField label="New rate" value={newRate} onChange={setNewRate} suffix="%" min={0} />
        <NumberField label="New term" value={newTermYears} onChange={setNewTermYears} suffix="years" min={1} />
        <NumberField label="Closing costs" value={closingCosts} onChange={setClosingCosts} suffix={money} min={0} />
      </div>

      <div className="panel-results">
        {result?.error && <div className="error">{result.error}</div>}
        {result && !result.error && (
          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">Current payment</span>
              <span className="stat-value">{formatMoney(result.current_payment)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">New payment</span>
              <span className="stat-value">{formatMoney(result.new_payment)}</span>
            </div>
            <div className="stat stat-primary">
              <span className="stat-label">Monthly savings</span>
              <span className="stat-value">{formatMoney(result.payment_savings)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Break-even</span>
              <span className="stat-value">
                {result.break_even_periods ? `${result.break_even_periods} months` : 'Never'}
              </span>
            </div>
            <div className="stat">
              <span className="stat-label">Lifetime savings (net of costs)</span>
              <span className="stat-value">{formatMoney(result.lifetime_savings)}</span>
            </div>
          </div>
        )}
      </div>

      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="refinance"
        getCurrentInputs={() => ({
          currentBalance,
          currentRate,
          remainingPeriods,
          newRate,
          newTermYears,
          closingCosts,
        })}
        onLoad={(inputs) => {
          setCurrentBalance(inputs.currentBalance);
          setCurrentRate(inputs.currentRate);
          setRemainingPeriods(inputs.remainingPeriods);
          setNewRate(inputs.newRate);
          setNewTermYears(inputs.newTermYears);
          setClosingCosts(inputs.closingCosts);
        }}
      />
    </section>
  );
}
