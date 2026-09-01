import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';
import { makeFormatEstimate, makeFormatMoney } from './../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { useCurrentInputs } from '../currentInputs';

// This panel only renders under the SG region, so its currency is fixed.
const formatSgd = makeFormatMoney('SG');
// Ceilings are estimates, not quotes — see makeFormatEstimate.
const estimateSgd = makeFormatEstimate('SG');

const DEFAULTS = {
  fixedIncome: 12000,
  variableIncome: 0,
  debts: 0,
  cash: 400000,
  cpf: 0,
  rate: 1.42,
  thereafterRate: 1.72,
  termYears: 25,
  age: 35,
  isHdb: false,
  residency: 'Citizen',
  propertyCount: '1st',
  outstandingLoans: 0,
};

/**
 * Singapore affordability: how much property a buyer can actually complete on.
 *
 * Deliberately not the US panel with a currency symbol swapped. Singapore has
 * no debt-to-income convention, no HOA and no flat property-tax rate — it has
 * MAS's TDSR/MSR servicing ceilings, the Notice 632 LTV limits, and IRAS stamp
 * duty payable in cash at completion. Every figure below comes from
 * `mortgage_calc::singapore` through the `calculate_sg_affordability` binding.
 */
export default function SingaporeAffordability({ wasmModule, dataVersion }) {
  const { t } = useI18n();
  const [fixedIncome, setFixedIncome] = useState(DEFAULTS.fixedIncome);
  const [variableIncome, setVariableIncome] = useState(DEFAULTS.variableIncome);
  const [debts, setDebts] = useState(DEFAULTS.debts);
  const [cash, setCash] = useState(DEFAULTS.cash);
  const [cpf, setCpf] = useState(DEFAULTS.cpf);
  // The rate the package opens at, and the one it steps up to. Singapore
  // packages are quoted this way and MAS assesses servicing on the second --
  // see mortgage-calc/src/singapore.rs.
  const [rate, setRate] = useState(DEFAULTS.rate);
  const [thereafterRate, setThereafterRate] = useState(DEFAULTS.thereafterRate);
  const [termYears, setTermYears] = useState(DEFAULTS.termYears);
  const [age, setAge] = useState(DEFAULTS.age);
  const [isHdb, setIsHdb] = useState(DEFAULTS.isHdb);
  const [residency, setResidency] = useState(DEFAULTS.residency);
  const [propertyCount, setPropertyCount] = useState(DEFAULTS.propertyCount);
  const [outstandingLoans, setOutstandingLoans] = useState(DEFAULTS.outstandingLoans);

  // Wider than <SavedScenarios>'s own getCurrentInputs/onLoad below (which
  // predates thereafterRate and doesn't capture it) -- that pair is the
  // named-save feature and is left as-is; this auto-persist can capture
  // everything the form actually shows.
  useCurrentInputs({
    wasmModule,
    storageKey: 'affordability-sg',
    getCurrentInputs: () => ({
      fixedIncome,
      variableIncome,
      debts,
      cash,
      cpf,
      rate,
      thereafterRate,
      termYears,
      age,
      isHdb,
      residency,
      propertyCount,
      outstandingLoans,
    }),
    onLoad: (inputs) => {
      setFixedIncome(inputs.fixedIncome);
      setVariableIncome(inputs.variableIncome);
      setDebts(inputs.debts);
      setCash(inputs.cash);
      setCpf(inputs.cpf);
      setRate(inputs.rate);
      setThereafterRate(inputs.thereafterRate);
      setTermYears(inputs.termYears);
      setAge(inputs.age);
      setIsHdb(inputs.isHdb);
      setResidency(inputs.residency);
      setPropertyCount(inputs.propertyCount);
      setOutstandingLoans(inputs.outstandingLoans);
    },
    dataVersion,
    defaultInputs: DEFAULTS,
  });

  const result = useMemo(() => {
    if (!wasmModule?.calculate_sg_affordability) return null;
    if (!allFilled(fixedIncome, variableIncome, debts, cash, cpf, rate, thereafterRate, termYears))
      return null;
    return wasmModule.calculate_sg_affordability({
      fixed_monthly_income: fixedIncome,
      variable_monthly_income: variableIncome,
      other_monthly_debts: debts,
      cash_available: cash,
      cpf_oa_available: cpf,
      annual_rate_percent: rate,
      thereafter_annual_rate_percent: thereafterRate,
      term_years: termYears,
      borrower_age: allFilled(age) ? age : null,
      is_hdb_or_ec: isHdb,
      residency,
      property_count: propertyCount,
      outstanding_housing_loans: outstandingLoans,
    });
  }, [
    wasmModule,
    fixedIncome,
    variableIncome,
    debts,
    cash,
    cpf,
    rate,
    thereafterRate,
    termYears,
    age,
    isHdb,
    residency,
    propertyCount,
    outstandingLoans,
  ]);

  const ok = result && !result.error;

  return (
    <section className="panel">
      <div className="panel-form">
        <NumberField
          label={t('sgaff.fixedIncome')}
          value={fixedIncome}
          onChange={setFixedIncome}
          suffix="S$"
          min={0}
          grouped
        />
        <NumberField
          label={t('sgaff.variableIncome')}
          value={variableIncome}
          onChange={setVariableIncome}
          suffix="S$"
          min={0}
          grouped
        />
        <NumberField
          label={t('sg.otherDebts')}
          value={debts}
          onChange={setDebts}
          suffix="S$"
          min={0}
          grouped
        />
        <NumberField
          label={t('sgaff.cash')}
          value={cash}
          onChange={setCash}
          suffix="S$"
          min={0}
          grouped
        />
        <NumberField
          label={t('sgaff.cpf')}
          value={cpf}
          onChange={setCpf}
          suffix="S$"
          min={0}
          grouped
        />
        <NumberField
          label={t('sgaff.initialRate')}
          value={rate}
          onChange={setRate}
          suffix="%"
          min={0}
        />
        <NumberField
          label={t('sgaff.thereafterRate')}
          value={thereafterRate}
          onChange={setThereafterRate}
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
          label={t('sgaff.age')}
          value={age}
          onChange={setAge}
          suffix={t('sgaff.yearsOld')}
          min={18}
        />

        <label className="field">
          <span className="field-label">{t('sg.propertyType')}</span>
          <select
            className="field-select"
            value={isHdb ? 'hdb' : 'private'}
            onChange={(e) => setIsHdb(e.target.value === 'hdb')}
          >
            <option value="private">{t('sg.private')}</option>
            <option value="hdb">{t('sg.hdb')}</option>
          </select>
        </label>

        <label className="field">
          <span className="field-label">{t('sg.residency')}</span>
          <select
            className="field-select"
            value={residency}
            onChange={(e) => setResidency(e.target.value)}
          >
            <option value="Citizen">{t('sg.citizen')}</option>
            <option value="PR">{t('sg.pr')}</option>
            <option value="Foreigner">{t('sg.foreigner')}</option>
            <option value="FTA">{t('sg.ftaNational')}</option>
          </select>
        </label>

        <label className="field">
          <span className="field-label">{t('sg.propertyCount')}</span>
          <select
            className="field-select"
            value={propertyCount}
            onChange={(e) => setPropertyCount(e.target.value)}
          >
            <option value="1st">{t('sg.first')}</option>
            <option value="2nd">{t('sg.second')}</option>
            <option value="3rd+">{t('sg.thirdPlus')}</option>
          </select>
        </label>

        {/* Separate from property count: a buyer can own a flat outright and
            still be taking their first housing loan, and it is the loan count
            that sets the LTV row. */}
        <label className="field">
          <span className="field-label">{t('sgaff.outstandingLoans')}</span>
          <select
            className="field-select"
            value={String(outstandingLoans)}
            onChange={(e) => setOutstandingLoans(Number(e.target.value))}
          >
            <option value="0">{t('sgaff.loansNone')}</option>
            <option value="1">{t('sgaff.loansOne')}</option>
            <option value="2">{t('sgaff.loansTwoPlus')}</option>
          </select>
        </label>
      </div>

      <div className="panel-results" aria-live="polite">
        {result?.error && (
          <div className="error">
            {result.error_message
              ? t(result.error_message.code, result.error_message.params)
              : result.error}
          </div>
        )}

        {ok && (
          <>
            <div className="stat-grid">
              <div className="stat stat-primary">
                <span className="stat-label">{t('sgaff.maxPrice')}</span>
                <span className="stat-value">{estimateSgd(result.max_price)}</span>
              </div>
              <div className="stat">
                <span className="stat-label">{t('sgaff.maxLoan')}</span>
                <span className="stat-value">{estimateSgd(result.max_loan)}</span>
                <span className="stat-note">
                  {t('sgaff.ltvNote', { ltv: result.ltv_percent.toFixed(0) })}
                  {result.extended_tenure ? ` · ${t('sgaff.extendedTenure')}` : ''}
                </span>
              </div>
              <div className="stat">
                <span className="stat-label">{t('sgaff.maxInstalment')}</span>
                <span className="stat-value">{formatSgd(result.max_monthly_instalment)}</span>
                <span className="stat-note">
                  {t('sg.assessedAt', {
                    rate: result.assessment_rate_percent.toFixed(2),
                    instalment: formatSgd(result.max_monthly_instalment),
                  })}
                </span>
              </div>
            </div>

            {/* Naming the binding rule is the actionable part: it tells the
                buyer whether earning more or saving more would move the
                number. */}
            <div className="sg-constraint">{t(`sgaff.bound.${result.binding_constraint}`)}</div>

            {/* The FTA remission is claimed from IRAS, not applied at the
                counter — saying so matters, because a buyer who assumes it
                is automatic will be short S$276k on completion day. */}
            {residency === 'FTA' && <div className="sg-constraint">{t('sgaff.ftaNote')}</div>}

            <div className="stat-grid">
              <div className="stat">
                <span className="stat-label">{t('sg.downPayment')}</span>
                <span className="stat-value">{formatSgd(result.deposit)}</span>
                <span className="stat-note">
                  {t('sgaff.minCash', { amount: formatSgd(result.min_cash_required) })}
                </span>
              </div>
              <div className="stat">
                <span className="stat-label">{t('sg.bsd')}</span>
                <span className="stat-value">{formatSgd(result.bsd)}</span>
              </div>
              <div className="stat">
                <span className="stat-label">{t('sg.absd')}</span>
                <span className="stat-value">{formatSgd(result.absd)}</span>
              </div>
              <div className="stat">
                <span className="stat-label">{t('sgaff.cpfUsed')}</span>
                <span className="stat-value">{formatSgd(result.cpf_used)}</span>
              </div>
              <div className="stat stat-primary">
                <span className="stat-label">{t('sgaff.cashRequired')}</span>
                <span className="stat-value">{formatSgd(result.cash_required)}</span>
                <span className="stat-note">{t('sgaff.cashNote')}</span>
              </div>
            </div>
          </>
        )}
      </div>

      <SavedScenarios
        wasmModule={wasmModule}
        dataVersion={dataVersion}
        calculatorKind="affordability"
        getCurrentInputs={() => ({
          fixedIncome,
          variableIncome,
          debts,
          cash,
          cpf,
          rate,
          termYears,
          age,
          isHdb,
          residency,
          propertyCount,
          outstandingLoans,
        })}
        onLoad={(inputs) => {
          setFixedIncome(inputs.fixedIncome);
          setVariableIncome(inputs.variableIncome);
          setDebts(inputs.debts);
          setCash(inputs.cash);
          setCpf(inputs.cpf);
          setRate(inputs.rate);
          setTermYears(inputs.termYears);
          setAge(inputs.age);
          setIsHdb(inputs.isHdb);
          setResidency(inputs.residency);
          setPropertyCount(inputs.propertyCount);
          setOutstandingLoans(inputs.outstandingLoans);
        }}
      />
    </section>
  );
}
