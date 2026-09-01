import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import CalcError from './CalcError';
import SavedScenarios from './SavedScenarios';
import { currencySymbol, makeFormatEstimate, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { useCurrentInputs } from '../currentInputs';

const DEFAULTS = {
  income: 10000,
  debts: 500,
  downPayment: 60000,
  rate: 6.5,
  termYears: 30,
  maxDti: 36,
  taxRate: 1.2,
  insurance: 1500,
  hoa: 0,
};

export default function AffordabilityCalculator({ wasmModule, region, dataVersion }) {
  const { t } = useI18n();
  const formatMoney = makeFormatMoney(region);
  // Ceilings are estimates, not quotes — see makeFormatEstimate.
  const formatEstimate = makeFormatEstimate(region);
  const money = currencySymbol(region);
  const [income, setIncome] = useState(DEFAULTS.income);
  const [debts, setDebts] = useState(DEFAULTS.debts);
  const [downPayment, setDownPayment] = useState(DEFAULTS.downPayment);
  const [rate, setRate] = useState(DEFAULTS.rate);
  const [termYears, setTermYears] = useState(DEFAULTS.termYears);
  const [maxDti, setMaxDti] = useState(DEFAULTS.maxDti);
  const [taxRate, setTaxRate] = useState(DEFAULTS.taxRate);
  const [insurance, setInsurance] = useState(DEFAULTS.insurance);
  const [hoa, setHoa] = useState(DEFAULTS.hoa);

  const getCurrentInputs = () => ({
    income,
    debts,
    downPayment,
    rate,
    termYears,
    maxDti,
    taxRate,
    insurance,
    hoa,
  });
  const onLoad = (inputs) => {
    setIncome(inputs.income);
    setDebts(inputs.debts);
    setDownPayment(inputs.downPayment);
    setRate(inputs.rate);
    setTermYears(inputs.termYears);
    setMaxDti(inputs.maxDti);
    setTaxRate(inputs.taxRate);
    setInsurance(inputs.insurance);
    setHoa(inputs.hoa);
  };

  useCurrentInputs({
    wasmModule,
    storageKey: 'affordability-us',
    getCurrentInputs,
    onLoad,
    dataVersion,
    defaultInputs: DEFAULTS,
  });

  const result = useMemo(() => {
    if (!wasmModule) return null;
    if (!allFilled(income, debts, downPayment, rate, termYears, maxDti, taxRate, insurance, hoa))
      return null;
    return wasmModule.calculate_affordability({
      gross_monthly_income: income,
      monthly_debts: debts,
      down_payment: downPayment,
      annual_rate_percent: rate,
      term_years: termYears,
      max_dti_percent: maxDti,
      annual_property_tax_rate_percent: taxRate,
      annual_insurance: insurance,
      monthly_hoa: hoa,
    });
  }, [wasmModule, income, debts, downPayment, rate, termYears, maxDti, taxRate, insurance, hoa]);

  return (
    <section className="panel">
      <div className="panel-form">
        <NumberField
          label={t('aff.income')}
          value={income}
          onChange={setIncome}
          suffix={money}
          min={0}
          grouped
        />
        <NumberField
          label={t('aff.debts')}
          value={debts}
          onChange={setDebts}
          suffix={money}
          min={0}
          grouped
        />
        <NumberField
          label={t('aff.downPayment')}
          value={downPayment}
          onChange={setDownPayment}
          suffix={money}
          min={0}
          grouped
        />
        <NumberField
          label={t('field.interestRate')}
          value={rate}
          onChange={setRate}
          suffix="%"
          min={0}
        />
        <NumberField
          label={t('field.loanTerm')}
          value={termYears}
          onChange={setTermYears}
          suffix={t('field.years')}
          min={1}
        />
        <NumberField
          label={t('aff.maxDti')}
          value={maxDti}
          onChange={setMaxDti}
          suffix="%"
          min={1}
        />
        <NumberField
          label={t('aff.propertyTaxRate')}
          value={taxRate}
          onChange={setTaxRate}
          suffix={t('field.percentPerYear')}
          min={0}
        />
        <NumberField
          label={t('aff.insurance')}
          value={insurance}
          onChange={setInsurance}
          suffix={money}
          min={0}
          grouped
        />
        <NumberField
          label={t('aff.hoa')}
          value={hoa}
          onChange={setHoa}
          suffix={money}
          min={0}
          grouped
        />
      </div>

      <div className="panel-results" aria-live="polite">
        <CalcError result={result} />
        {result && !result.error && (
          <div className="stat-grid">
            <div className="stat stat-primary">
              <span className="stat-label">{t('aff.maxHomePrice')}</span>
              <span className="stat-value">{formatEstimate(result.max_home_price)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('aff.maxLoan')}</span>
              <span className="stat-value">{formatEstimate(result.max_loan_amount)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('aff.principalAndInterest')}</span>
              <span className="stat-value">{formatMoney(result.max_principal_and_interest)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('aff.frontEndDti')}</span>
              <span className="stat-value">{result.front_end_dti_percent.toFixed(1)}%</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('aff.backEndDti')}</span>
              <span className="stat-value">{result.back_end_dti_percent.toFixed(1)}%</span>
            </div>
          </div>
        )}
      </div>

      <SavedScenarios
        wasmModule={wasmModule}
        dataVersion={dataVersion}
        calculatorKind="affordability"
        getCurrentInputs={getCurrentInputs}
        onLoad={onLoad}
      />
    </section>
  );
}
