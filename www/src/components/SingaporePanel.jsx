import React from 'react';
import NumberField from './NumberField';
import { makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';

// This panel only ever renders under the SG region, so its currency is
// fixed rather than passed in.
const formatSgd = makeFormatMoney('SG');

const formatPercent = (n) => (n == null ? '—' : `${n.toFixed(1)}%`);

/**
 * MAS/CPF/IRAS panel: TDSR and MSR borrowing limits, the CPF-versus-cash
 * split of the monthly payment, and BSD/ABSD stamp duty.
 *
 * Every figure here comes from `mortgage_calc::singapore` through the
 * `calculate_singapore` wasm binding — none of these rules are
 * reimplemented in JavaScript.
 */
export default function SingaporePanel({ inputs, onChange, result }) {
  const { t } = useI18n();
  const set = (key) => (value) => onChange({ ...inputs, [key]: value });

  // A ratio that breaches its ceiling reads as an error; one inside the
  // warning band below it reads as a caution.
  const ratioClass = (exceeded, near) => {
    if (exceeded) return 'sg-ratio breach';
    if (near) return 'sg-ratio near';
    return 'sg-ratio';
  };

  return (
    <div className="sg-panel">
      <h3 className="sg-panel-title">{t('sg.title')}</h3>

      <div className="panel-form sg-panel-form">
        <NumberField
          label={t('sgaff.fixedIncome')}
          value={inputs.fixed_monthly_income}
          onChange={set('fixed_monthly_income')}
          suffix="S$"
          min={0}
        />
        <NumberField
          label={t('sgaff.variableIncome')}
          value={inputs.variable_monthly_income}
          onChange={set('variable_monthly_income')}
          suffix="S$"
          min={0}
        />
        <NumberField
          label={t('sg.otherDebts')}
          value={inputs.other_monthly_debts}
          onChange={set('other_monthly_debts')}
          suffix="S$"
          min={0}
        />
        <NumberField
          label={t('sg.cpfAvailable')}
          value={inputs.cpf_oa_available}
          onChange={set('cpf_oa_available')}
          suffix="S$"
          min={0}
        />
        <label className="field">
          <span className="field-label">{t('sg.residency')}</span>
          <select
            className="field-select"
            value={inputs.residency}
            onChange={(e) => set('residency')(e.target.value)}
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
            value={inputs.property_count}
            onChange={(e) => set('property_count')(e.target.value)}
          >
            <option value="1st">{t('sg.first')}</option>
            <option value="2nd">{t('sg.second')}</option>
            <option value="3rd+">{t('sg.thirdPlus')}</option>
          </select>
        </label>
        <label className="field">
          <span className="field-label">{t('sg.propertyType')}</span>
          <select
            className="field-select"
            value={inputs.is_hdb_or_ec ? 'hdb' : 'private'}
            onChange={(e) => set('is_hdb_or_ec')(e.target.value === 'hdb')}
          >
            <option value="private">{t('sg.private')}</option>
            <option value="hdb">{t('sg.hdb')}</option>
          </select>
        </label>
        <label className="field">
          <span className="field-label">{t('sg.loanType')}</span>
          <select
            className="field-select"
            value={inputs.loan_type}
            onChange={(e) => set('loan_type')(e.target.value)}
          >
            <option value="Bank Loan">{t('sg.bankLoan')}</option>
            <option value="HDB Loan">{t('sg.hdbLoan')}</option>
          </select>
        </label>
      </div>

      {result?.error && (
        <div className="error">
          {result.error_message ? t(result.error_message.code, result.error_message.params) : result.error}
        </div>
      )}

      {result?.loan_type_warning && (
        <div className="error">
          {result.loan_type_warning_code
            ? t(result.loan_type_warning_code)
            : result.loan_type_warning}
        </div>
      )}

      {result?.warnings?.map((w, i) => (
        <div className="error" key={w}>
          {result.warning_codes?.[i] ? t(result.warning_codes[i]) : w}
        </div>
      ))}

      {result && !result.error && (
        <>
          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">{t('sg.tdsr')}</span>
              <span className={ratioClass(result.tdsr_exceeded, result.tdsr_near_limit)}>
                {formatPercent(result.tdsr_ratio_percent)}
              </span>
              {result.assessed_monthly_instalment != null && (
                <span className="stat-note">
                  {t('sg.assessedAt', {
                    rate: result.assessment_rate_percent.toFixed(2),
                    instalment: formatSgd(result.assessed_monthly_instalment),
                  })}
                </span>
              )}
            </div>
            {result.msr_ratio_percent != null && (
              <div className="stat">
                <span className="stat-label">{t('sg.msr')}</span>
                <span className={ratioClass(result.msr_exceeded, result.msr_near_limit)}>
                  {formatPercent(result.msr_ratio_percent)}
                </span>
              </div>
            )}
            <div className="stat">
              <span className="stat-label">{t('sg.cpfUsed')}</span>
              <span className="stat-value">{formatSgd(result.cpf_used)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('sg.cashMonthly')}</span>
              <span className="stat-value">{formatSgd(result.cash_required)}</span>
            </div>
          </div>

          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">{t('sg.bsd')}</span>
              <span className="stat-value">{formatSgd(result.bsd)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('sg.absd')}</span>
              <span className="stat-value">{formatSgd(result.absd)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('sg.downPayment')}</span>
              <span className="stat-value">{formatSgd(result.down_payment)}</span>
            </div>
            <div className="stat stat-primary">
              <span className="stat-label">{t('sg.cashAtCompletion')}</span>
              <span className="stat-value">{formatSgd(result.total_cash_required)}</span>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
