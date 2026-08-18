import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';

const formatMoney = (n) =>
  n == null ? '—' : n.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

const PERIODS_PER_YEAR = { monthly: 12, biweekly: 26, weekly: 52 };

function summarizeByYear(rows, periodsPerYear) {
  const years = [];
  for (let i = 0; i < rows.length; i += periodsPerYear) {
    const chunk = rows.slice(i, i + periodsPerYear);
    years.push({
      year: years.length + 1,
      paid: chunk.reduce((sum, r) => sum + r.payment, 0),
      principal: chunk.reduce((sum, r) => sum + r.principal_portion, 0),
      interest: chunk.reduce((sum, r) => sum + r.interest_portion, 0),
      remaining_balance: chunk[chunk.length - 1].remaining_balance,
    });
  }
  return years;
}

export default function AmortizationSchedule({ wasmModule }) {
  const [principal, setPrincipal] = useState(400000);
  const [rate, setRate] = useState(6.5);
  const [termYears, setTermYears] = useState(30);
  const [frequency, setFrequency] = useState('monthly');
  const [extraPayment, setExtraPayment] = useState(0);
  const [showFullSchedule, setShowFullSchedule] = useState(false);

  const loan = { principal, annual_rate_percent: rate, term_years: termYears, frequency };

  const schedule = useMemo(() => {
    if (!wasmModule) return null;
    return wasmModule.calculate_amortization_schedule({ loan, extra_payment: extraPayment || 0 });
  }, [wasmModule, principal, rate, termYears, frequency, extraPayment]);

  const impact = useMemo(() => {
    if (!wasmModule || !extraPayment) return null;
    return wasmModule.calculate_extra_payment_impact({ loan, extra_payment: extraPayment });
  }, [wasmModule, principal, rate, termYears, frequency, extraPayment]);

  const yearlyRows = useMemo(() => {
    if (!schedule?.rows?.length) return [];
    return summarizeByYear(schedule.rows, PERIODS_PER_YEAR[frequency] ?? 12);
  }, [schedule, frequency]);

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
        <NumberField
          label="Extra payment per period"
          value={extraPayment}
          onChange={setExtraPayment}
          suffix="$"
          min={0}
        />
      </div>

      {impact && !impact.error && (
        <div className="stat-grid">
          <div className="stat stat-primary">
            <span className="stat-label">Payoff time saved</span>
            <span className="stat-value">{impact.periods_saved} payments</span>
          </div>
          <div className="stat">
            <span className="stat-label">Interest saved</span>
            <span className="stat-value">{formatMoney(impact.interest_saved)}</span>
          </div>
          <div className="stat">
            <span className="stat-label">New payoff</span>
            <span className="stat-value">{impact.payoff_periods} payments</span>
          </div>
        </div>
      )}

      {schedule?.error && <div className="error">{schedule.error}</div>}

      {yearlyRows.length > 0 && (
        <div className="schedule-table-wrap">
          <div className="schedule-table-header">
            <h3>{showFullSchedule ? 'Full schedule' : 'Yearly summary'}</h3>
            <button className="link-button" onClick={() => setShowFullSchedule((v) => !v)}>
              {showFullSchedule ? 'Show yearly summary' : 'Show every payment'}
            </button>
          </div>
          <table className="schedule-table">
            <thead>
              <tr>
                <th>{showFullSchedule ? 'Period' : 'Year'}</th>
                <th>Paid</th>
                <th>Principal</th>
                <th>Interest</th>
                <th>Balance</th>
              </tr>
            </thead>
            <tbody>
              {(showFullSchedule ? schedule.rows : yearlyRows).map((row) => (
                <tr key={showFullSchedule ? row.period : row.year}>
                  <td>{showFullSchedule ? row.period : row.year}</td>
                  <td>{formatMoney(showFullSchedule ? row.payment : row.paid)}</td>
                  <td>{formatMoney(showFullSchedule ? row.principal_portion : row.principal)}</td>
                  <td>{formatMoney(showFullSchedule ? row.interest_portion : row.interest)}</td>
                  <td>{formatMoney(row.remaining_balance)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="amortization"
        getCurrentInputs={() => ({ principal, rate, termYears, frequency, extraPayment })}
        onLoad={(inputs) => {
          setPrincipal(inputs.principal);
          setRate(inputs.rate);
          setTermYears(inputs.termYears);
          setFrequency(inputs.frequency);
          setExtraPayment(inputs.extraPayment);
        }}
      />
    </section>
  );
}
