import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';

const formatMoney = (n) =>
  n == null ? '—' : n.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

export default function AffordabilityCalculator({ wasmModule }) {
  const [income, setIncome] = useState(10000);
  const [debts, setDebts] = useState(500);
  const [downPayment, setDownPayment] = useState(60000);
  const [rate, setRate] = useState(6.5);
  const [termYears, setTermYears] = useState(30);
  const [maxDti, setMaxDti] = useState(36);
  const [taxRate, setTaxRate] = useState(1.2);
  const [insurance, setInsurance] = useState(1500);
  const [hoa, setHoa] = useState(0);

  const result = useMemo(() => {
    if (!wasmModule) return null;
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
        <NumberField label="Gross monthly income" value={income} onChange={setIncome} suffix="$" min={0} />
        <NumberField label="Other monthly debts" value={debts} onChange={setDebts} suffix="$" min={0} />
        <NumberField label="Down payment" value={downPayment} onChange={setDownPayment} suffix="$" min={0} />
        <NumberField label="Interest rate" value={rate} onChange={setRate} suffix="%" min={0} />
        <NumberField label="Loan term" value={termYears} onChange={setTermYears} suffix="years" min={1} />
        <NumberField label="Max debt-to-income" value={maxDti} onChange={setMaxDti} suffix="%" min={1} />
        <NumberField label="Property tax rate" value={taxRate} onChange={setTaxRate} suffix="%/yr" min={0} />
        <NumberField label="Annual insurance" value={insurance} onChange={setInsurance} suffix="$" min={0} />
        <NumberField label="Monthly HOA" value={hoa} onChange={setHoa} suffix="$" min={0} />
      </div>

      <div className="panel-results">
        {result?.error && <div className="error">{result.error}</div>}
        {result && !result.error && (
          <div className="stat-grid">
            <div className="stat stat-primary">
              <span className="stat-label">Max home price</span>
              <span className="stat-value">{formatMoney(result.max_home_price)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Max loan amount</span>
              <span className="stat-value">{formatMoney(result.max_loan_amount)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Principal &amp; interest</span>
              <span className="stat-value">{formatMoney(result.max_principal_and_interest)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Front-end DTI</span>
              <span className="stat-value">{result.front_end_dti_percent.toFixed(1)}%</span>
            </div>
            <div className="stat">
              <span className="stat-label">Back-end DTI</span>
              <span className="stat-value">{result.back_end_dti_percent.toFixed(1)}%</span>
            </div>
          </div>
        )}
      </div>

      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="affordability"
        getCurrentInputs={() => ({
          income,
          debts,
          downPayment,
          rate,
          termYears,
          maxDti,
          taxRate,
          insurance,
          hoa,
        })}
        onLoad={(inputs) => {
          setIncome(inputs.income);
          setDebts(inputs.debts);
          setDownPayment(inputs.downPayment);
          setRate(inputs.rate);
          setTermYears(inputs.termYears);
          setMaxDti(inputs.maxDti);
          setTaxRate(inputs.taxRate);
          setInsurance(inputs.insurance);
          setHoa(inputs.hoa);
        }}
      />
    </section>
  );
}
