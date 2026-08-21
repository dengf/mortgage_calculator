import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';
import SingaporePanel from './SingaporePanel';
import UnitedStatesPanel from './UnitedStatesPanel';
import { PrincipalInterestSplit } from './Charts';

// Symbols are spelled out rather than left to Intl: it renders SGD in en-SG
// as a bare "$", indistinguishable from USD once the region toggle exists.
const CURRENCY = { US: ['en-US', '$'], SG: ['en-SG', 'S$'] };

const US_DEFAULTS = {
  home_price: 500000,
  zip: '90210',
  pmi_rate_percent: 0.75,
  use_tax_deduction: false,
  marginal_tax_rate_percent: 24,
};

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
  const [usInputs, setUsInputs] = useState(US_DEFAULTS);

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
    // Guarded on the function, not just the module: a returning visitor can
    // hold a cached wasm build from before a binding existed, and calling a
    // missing export would take down the whole tab rather than one panel.
    if (!wasmModule?.calculate_singapore || region !== 'SG') return null;
    const monthlyPayment =
      result && !result.error && frequency === 'monthly' ? result.payment : null;
    return wasmModule.calculate_singapore({
      ...sgInputs,
      monthly_payment: monthlyPayment,
      principal,
    });
  }, [wasmModule, region, sgInputs, result, frequency, principal]);

  // PITI, PMI and the deduction estimate are all monthly figures, so like
  // the Singapore panel they only apply to a monthly schedule. Property tax
  // and PMI price off the home price and loan amount and still compute.
  const usResult = useMemo(() => {
    if (!wasmModule?.calculate_united_states || region !== 'US') return null;
    const monthlyPi =
      result && !result.error && frequency === 'monthly' ? result.payment : null;
    return wasmModule.calculate_united_states({
      ...usInputs,
      monthly_pi: monthlyPi,
      principal,
      annual_rate_percent: rate,
    });
  }, [wasmModule, region, usInputs, result, frequency, principal, rate]);

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

      {result && !result.error && (
        <PrincipalInterestSplit
          principal={principal}
          totalInterest={result.total_interest}
          formatMoney={formatMoney}
        />
      )}

      {region === 'US' && (
        <UnitedStatesPanel inputs={usInputs} onChange={setUsInputs} result={usResult} />
      )}

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
