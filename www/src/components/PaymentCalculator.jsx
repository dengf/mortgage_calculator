import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';
import SingaporePanel from './SingaporePanel';

// Symbols are spelled out rather than left to Intl: it renders SGD in en-SG
// as a bare "$", indistinguishable from USD once the region toggle exists.
const CURRENCY = { US: ['en-US', '$'], SG: ['en-SG', 'S$'] };

const SG_DEFAULTS = {
  home_price: 1000000,
  gross_monthly_income: 12000,
  other_monthly_debts: 0,
  cpf_oa_available: 1500,
  residency: 'Citizen',
  property_count: '1st',
  is_hdb_or_ec: false,
  loan_type: 'Bank Loan',
};

export default function PaymentCalculator({ wasmModule, region = 'US' }) {
  const [principal, setPrincipal] = useState(400000);
  const [rate, setRate] = useState(6.5);
  const [termYears, setTermYears] = useState(30);
  const [frequency, setFrequency] = useState('monthly');
  const [sgInputs, setSgInputs] = useState(SG_DEFAULTS);

  const [locale, symbol] = CURRENCY[region] ?? CURRENCY.US;
  const formatMoney = (n) =>
    n == null
      ? '—'
      : `${symbol}${n.toLocaleString(locale, {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        })}`;

  const result = useMemo(() => {
    if (!wasmModule) return null;
    return wasmModule.calculate_payment({
      principal,
      annual_rate_percent: rate,
      term_years: termYears,
      frequency,
    });
  }, [wasmModule, principal, rate, termYears, frequency]);

  // TDSR/MSR and the CPF split are monthly ceilings, so a non-monthly
  // schedule has no meaningful payment to test against them. Stamp duty
  // still computes either way, since it prices off the property alone.
  const sgResult = useMemo(() => {
    if (!wasmModule || region !== 'SG') return null;
    const monthlyPayment =
      result && !result.error && frequency === 'monthly' ? result.payment : null;
    return wasmModule.calculate_singapore({
      ...sgInputs,
      monthly_payment: monthlyPayment,
      principal,
    });
  }, [wasmModule, region, sgInputs, result, frequency, principal]);

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

      {region === 'SG' && (
        <SingaporePanel inputs={sgInputs} onChange={setSgInputs} result={sgResult} />
      )}

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
