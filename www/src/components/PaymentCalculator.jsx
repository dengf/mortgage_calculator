import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';

const formatMoney = (n) =>
  n == null ? '—' : n.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

export default function PaymentCalculator({ wasmModule }) {
  const [principal, setPrincipal] = useState(400000);
  const [rate, setRate] = useState(6.5);
  const [termYears, setTermYears] = useState(30);
  const [frequency, setFrequency] = useState('monthly');

  const result = useMemo(() => {
    if (!wasmModule) return null;
    return wasmModule.calculate_payment({
      principal,
      annual_rate_percent: rate,
      term_years: termYears,
      frequency,
    });
  }, [wasmModule, principal, rate, termYears, frequency]);

  return (
    <section className="panel">
      <div className="panel-form">
        <NumberField label="Home loan amount" value={principal} onChange={setPrincipal} suffix="$" min={0} />
        <NumberField label="Interest rate" value={rate} onChange={setRate} suffix="%" min={0} />
        <NumberField label="Loan term" value={termYears} onChange={setTermYears} suffix="years" min={1} />
        <label className="field">
          <span className="field-label">Payment frequency</span>
          <select className="field-select" value={frequency} onChange={(e) => setFrequency(e.target.value)}>
            <option value="monthly">Monthly</option>
            <option value="biweekly">Bi-weekly</option>
            <option value="weekly">Weekly</option>
          </select>
        </label>
      </div>

      <div className="panel-results">
        {result?.error && <div className="error">{result.error}</div>}
        {result && !result.error && (
          <div className="stat-grid">
            <div className="stat stat-primary">
              <span className="stat-label">Payment</span>
              <span className="stat-value">{formatMoney(result.payment)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Total of {result.total_periods} payments</span>
              <span className="stat-value">{formatMoney(result.total_paid)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Total interest</span>
              <span className="stat-value">{formatMoney(result.total_interest)}</span>
            </div>
          </div>
        )}
      </div>

      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({ principal, rate, termYears, frequency })}
        onLoad={(inputs) => {
          setPrincipal(inputs.principal);
          setRate(inputs.rate);
          setTermYears(inputs.termYears);
          setFrequency(inputs.frequency);
        }}
      />
    </section>
  );
}
