import React from 'react';
import NumberField from './NumberField';
import { makeFormatMoney } from '../currency';

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
      <h3 className="sg-panel-title">Singapore rules</h3>

      <div className="panel-form sg-panel-form">
        <NumberField
          label="Property price"
          value={inputs.home_price}
          onChange={set('home_price')}
          suffix="S$"
          min={0}
        />
        <NumberField
          label="Gross monthly income"
          value={inputs.gross_monthly_income}
          onChange={set('gross_monthly_income')}
          suffix="S$"
          min={0}
        />
        <NumberField
          label="Other monthly debts"
          value={inputs.other_monthly_debts}
          onChange={set('other_monthly_debts')}
          suffix="S$"
          min={0}
        />
        <NumberField
          label="CPF OA available monthly"
          value={inputs.cpf_oa_available}
          onChange={set('cpf_oa_available')}
          suffix="S$"
          min={0}
        />
        <label className="field">
          <span className="field-label">Residency</span>
          <select
            className="field-select"
            value={inputs.residency}
            onChange={(e) => set('residency')(e.target.value)}
          >
            <option value="Citizen">Citizen</option>
            <option value="PR">Permanent Resident</option>
            <option value="Foreigner">Foreigner</option>
          </select>
        </label>
        <label className="field">
          <span className="field-label">Properties owned after purchase</span>
          <select
            className="field-select"
            value={inputs.property_count}
            onChange={(e) => set('property_count')(e.target.value)}
          >
            <option value="1st">1st property</option>
            <option value="2nd">2nd property</option>
            <option value="3rd+">3rd or more</option>
          </select>
        </label>
        <label className="field">
          <span className="field-label">Property type</span>
          <select
            className="field-select"
            value={inputs.is_hdb_or_ec ? 'hdb' : 'private'}
            onChange={(e) => set('is_hdb_or_ec')(e.target.value === 'hdb')}
          >
            <option value="private">Private property</option>
            <option value="hdb">HDB flat / EC</option>
          </select>
        </label>
        <label className="field">
          <span className="field-label">Loan type</span>
          <select
            className="field-select"
            value={inputs.loan_type}
            onChange={(e) => set('loan_type')(e.target.value)}
          >
            <option value="Bank Loan">Bank loan</option>
            <option value="HDB Loan">HDB concessionary loan</option>
          </select>
        </label>
      </div>

      {result?.error && <div className="error">{result.error}</div>}

      {result?.loan_type_warning && <div className="error">{result.loan_type_warning}</div>}

      {result?.warnings?.map((w) => (
        <div className="error" key={w}>
          {w}
        </div>
      ))}

      {result && !result.error && (
        <>
          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">TDSR (limit 55%)</span>
              <span className={ratioClass(result.tdsr_exceeded, result.tdsr_near_limit)}>
                {formatPercent(result.tdsr_ratio_percent)}
              </span>
            </div>
            {result.msr_ratio_percent != null && (
              <div className="stat">
                <span className="stat-label">MSR (limit 30%)</span>
                <span className={ratioClass(result.msr_exceeded, result.msr_near_limit)}>
                  {formatPercent(result.msr_ratio_percent)}
                </span>
              </div>
            )}
            <div className="stat">
              <span className="stat-label">Paid from CPF OA</span>
              <span className="stat-value">{formatSgd(result.cpf_used)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Cash needed monthly</span>
              <span className="stat-value">{formatSgd(result.cash_required)}</span>
            </div>
          </div>

          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">Buyer's Stamp Duty</span>
              <span className="stat-value">{formatSgd(result.bsd)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Additional Buyer's Stamp Duty</span>
              <span className="stat-value">{formatSgd(result.absd)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Down payment</span>
              <span className="stat-value">{formatSgd(result.down_payment)}</span>
            </div>
            <div className="stat stat-primary">
              <span className="stat-label">Cash needed at completion</span>
              <span className="stat-value">{formatSgd(result.total_cash_required)}</span>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
