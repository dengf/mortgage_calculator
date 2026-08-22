import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';
import SingaporePanel from './SingaporePanel';
import UnitedStatesPanel from './UnitedStatesPanel';
import { PrincipalInterestSplit } from './Charts';
import { currencySymbol, makeFormatMoney } from '../currency';
import { allFilled, useSticky } from '../inputs';
import { useI18n } from '../i18n';


const US_DEFAULTS = {
  home_price: 500000,
  zip: '90210',
  pmi_rate_percent: 0.75,
  use_tax_deduction: false,
  marginal_tax_rate_percent: 24,
};

const SG_DEFAULTS = {
  home_price: 1000000,
  fixed_monthly_income: 12000,
  variable_monthly_income: 0,
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

  const { t } = useI18n();
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);

  // A blank field is an input mid-edit, not an error: don't cross the wasm
  // boundary with it. serde would reject `''` where it wants an f64, and
  // that failure used to surface as a raw parse error on the page.
  const liveResult = useMemo(() => {
    if (!wasmModule) return null;
    if (!allFilled(principal, rate, termYears)) return null;
    return {
      summary: wasmModule.calculate_payment({
        principal,
        annual_rate_percent: rate,
        term_years: termYears,
        frequency,
      }),
      // Carried alongside so the split chart is drawn from the same inputs
      // as the figures beside it. Reading live state there would mix a held
      // result with an in-flight principal and render a 100%-interest loan.
      principal,
    };
  }, [wasmModule, principal, rate, termYears, frequency]);

  const { value: held, stale } = useSticky(liveResult);
  const result = held?.summary ?? null;
  const shownPrincipal = held?.principal ?? principal;

  // TDSR/MSR and the CPF split are monthly ceilings, so a non-monthly
  // schedule has no meaningful payment to test against them. Stamp duty
  // still computes either way, since it prices off the property alone.
  const sgResult = useMemo(() => {
    // Guarded on the function, not just the module: a returning visitor can
    // hold a cached wasm build from before a binding existed, and calling a
    // missing export would take down the whole tab rather than one panel.
    if (!wasmModule?.calculate_singapore || region !== 'SG') return null;
    if (!allFilled(principal, rate, termYears)) return null;
    const monthlyPayment =
      result && !result.error && frequency === 'monthly' ? result.payment : null;
    return wasmModule.calculate_singapore({
      ...sgInputs,
      monthly_payment: monthlyPayment,
      principal,
      annual_rate_percent: rate,
      term_years: termYears,
    });
  }, [wasmModule, region, sgInputs, result, frequency, principal, rate, termYears]);

  // PITI, PMI and the deduction estimate are all monthly figures, so like
  // the Singapore panel they only apply to a monthly schedule. Property tax
  // and PMI price off the home price and loan amount and still compute.
  const usResult = useMemo(() => {
    if (!wasmModule?.calculate_united_states || region !== 'US') return null;
    if (!allFilled(principal, rate, termYears)) return null;
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
        <NumberField label={t('field.loanAmount')} value={principal} onChange={setPrincipal} suffix={money} min={0} />
        <NumberField label={t('field.interestRate')} value={rate} onChange={setRate} suffix="%" min={0} />
        <NumberField label={t('field.loanTerm')} value={termYears} onChange={setTermYears} suffix={t('field.years')} min={1} />
        <label className="field">
          <span className="field-label">{t('field.paymentFrequency')}</span>
          <select className="field-select" value={frequency} onChange={(e) => setFrequency(e.target.value)}>
            <option value="monthly">{t('freq.monthly')}</option>
            <option value="biweekly">{t('freq.biweekly')}</option>
            <option value="weekly">{t('freq.weekly')}</option>
          </select>
        </label>
      </div>

      <div className={stale ? 'panel-results stale' : 'panel-results'}>
        {result?.error && (
          <div className="error">
            {result.error_message
              ? t(result.error_message.code, result.error_message.params)
              : result.error}
          </div>
        )}
        {result && !result.error && (
          <div className="stat-grid">
            <div className="stat stat-primary">
              <span className="stat-label">{t('payment.payment')}</span>
              <span className="stat-value">{formatMoney(result.payment)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('payment.totalOf', { count: result.total_periods })}</span>
              <span className="stat-value">{formatMoney(result.total_paid)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('payment.totalInterest')}</span>
              <span className="stat-value">{formatMoney(result.total_interest)}</span>
            </div>
          </div>
        )}
      </div>

      {result && !result.error && (
        <div className={stale ? 'stale' : undefined}>
          <PrincipalInterestSplit
            principal={shownPrincipal}
            totalInterest={result.total_interest}
            formatMoney={formatMoney}
          />
        </div>
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
