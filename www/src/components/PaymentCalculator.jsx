import React, { useMemo, useState } from 'react';
import ScenarioFields from './ScenarioFields';
import SavedScenarios from './SavedScenarios';
import SingaporePanel from './SingaporePanel';
import UnitedStatesPanel from './UnitedStatesPanel';
import { PrincipalInterestSplit } from './Charts';
import { currencySymbol, makeFormatMoney } from '../currency';
import { allFilled, useSticky } from '../inputs';
import { DEFAULT_SCENARIO, useScenarioSummary } from '../scenario';
import { useI18n } from '../i18n';

const US_DEFAULTS = {
  zip: '90210',
  pmi_rate_percent: 0.75,
  use_tax_deduction: false,
  marginal_tax_rate_percent: 24,
};

const SG_DEFAULTS = {
  fixed_monthly_income: 12000,
  variable_monthly_income: 0,
  other_monthly_debts: 0,
  cpf_oa_available: 1500,
  residency: 'Citizen',
  property_count: '1st',
  is_hdb_or_ec: false,
  loan_type: 'Bank Loan',
};

export default function PaymentCalculator({
  wasmModule,
  region = 'US',
  scenario = DEFAULT_SCENARIO,
  onScenarioChange,
}) {
  const [sgInputs, setSgInputs] = useState(SG_DEFAULTS);
  const [usInputs, setUsInputs] = useState(US_DEFAULTS);

  // Read from the shared scenario rather than local state, so a loan dialled
  // in here survives a move to Amortization or Compare.
  const { homePrice, rate, termYears, frequency } = scenario;
  const { principal } = useScenarioSummary(wasmModule, scenario);

  const { t } = useI18n();
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);

  // A blank field is an input mid-edit, not an error: don't cross the wasm
  // boundary with it. serde would reject `''` where it wants an f64, and
  // that failure used to surface as a raw parse error on the page.
  const liveResult = useMemo(() => {
    if (!wasmModule) return null;
    if (!allFilled(homePrice, scenario.downPayment, rate, termYears)) return null;
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
    if (!allFilled(homePrice, scenario.downPayment, rate, termYears)) return null;
    const monthlyPayment =
      result && !result.error && frequency === 'monthly' ? result.payment : null;
    return wasmModule.calculate_singapore({
      ...sgInputs,
      home_price: homePrice,
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
    if (!allFilled(homePrice, scenario.downPayment, rate, termYears)) return null;
    const monthlyPi = result && !result.error && frequency === 'monthly' ? result.payment : null;
    return wasmModule.calculate_united_states({
      ...usInputs,
      home_price: homePrice,
      monthly_pi: monthlyPi,
      principal,
      annual_rate_percent: rate,
    });
  }, [wasmModule, region, usInputs, result, frequency, principal, rate]);

  return (
    <section className="panel">
      <ScenarioFields
        wasmModule={wasmModule}
        scenario={scenario}
        onChange={onScenarioChange}
        money={money}
        formatMoney={formatMoney}
      />

      <div className={stale ? 'panel-results stale' : 'panel-results'} aria-live="polite">
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
              <span className="stat-label">
                {t('payment.totalOf', { count: result.total_periods })}
              </span>
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
        <UnitedStatesPanel
          inputs={usInputs}
          onChange={setUsInputs}
          result={usResult}
          homePrice={homePrice}
        />
      )}

      {region === 'SG' && (
        <SingaporePanel inputs={sgInputs} onChange={setSgInputs} result={sgResult} />
      )}

      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({ ...scenario })}
        onLoad={(inputs) => {
          // Older saved scenarios stored a loan amount with no price. Recover
          // a price from it so they still load, rather than dropping them.
          onScenarioChange({
            ...scenario,
            ...inputs,
            homePrice: inputs.homePrice ?? inputs.principal ?? scenario.homePrice,
            downPayment: inputs.homePrice == null ? 0 : inputs.downPayment,
          });
        }}
      />
    </section>
  );
}
