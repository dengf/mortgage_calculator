import React from 'react';
import NumberField from './NumberField';

const formatUsd = (n) =>
  n == null ? '—' : n.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

/**
 * US panel: conforming/jumbo classification, ZIP-derived property tax, the
 * PMI trigger below 20% down, the resulting PITI, and an optional
 * mortgage-interest deduction estimate.
 *
 * Every figure comes from `mortgage_calc::united_states` through the
 * `calculate_united_states` binding — no US rules are reimplemented here.
 */
export default function UnitedStatesPanel({ inputs, onChange, result }) {
  const set = (key) => (value) => onChange({ ...inputs, [key]: value });

  return (
    <div className="sg-panel">
      <h3 className="sg-panel-title">US costs &amp; PMI</h3>

      <div className="panel-form sg-panel-form">
        <NumberField
          label="Home price"
          value={inputs.home_price}
          onChange={set('home_price')}
          suffix="$"
          min={0}
        />
        <label className="field">
          <span className="field-label">ZIP code</span>
          <div className="field-input">
            <input
              type="text"
              inputMode="numeric"
              maxLength={5}
              value={inputs.zip}
              onChange={(e) => set('zip')(e.target.value.replace(/\D/g, ''))}
            />
          </div>
        </label>
        <NumberField
          label="PMI rate"
          value={inputs.pmi_rate_percent}
          onChange={set('pmi_rate_percent')}
          suffix="%"
          min={0}
        />
        <label className="field">
          <span className="field-label">Estimate tax deduction</span>
          <select
            className="field-select"
            value={inputs.use_tax_deduction ? 'yes' : 'no'}
            onChange={(e) => set('use_tax_deduction')(e.target.value === 'yes')}
          >
            <option value="no">No</option>
            <option value="yes">Yes</option>
          </select>
        </label>
        {inputs.use_tax_deduction && (
          <NumberField
            label="Marginal tax rate"
            value={inputs.marginal_tax_rate_percent}
            onChange={set('marginal_tax_rate_percent')}
            suffix="%"
            min={0}
          />
        )}
      </div>

      {result?.error && <div className="error">{result.error}</div>}

      {result?.property_tax_rate_percent == null && inputs.zip.length > 0 && (
        <div className="error">
          ZIP {inputs.zip} doesn&apos;t match a state we have a property tax rate for, so tax
          is excluded below.
        </div>
      )}

      {result && !result.error && (
        <>
          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">Loan type</span>
              <span className="stat-value">{result.loan_type}</span>
            </div>
            <div className="stat">
              <span className="stat-label">Down payment</span>
              <span className="stat-value">
                {formatUsd(result.down_payment)}{' '}
                <small>({result.down_payment_percent.toFixed(1)}%)</small>
              </span>
            </div>
            <div className="stat">
              <span className="stat-label">
                Property tax
                {result.property_tax_rate_percent != null &&
                  ` (${result.property_tax_rate_percent.toFixed(2)}%)`}
              </span>
              <span className="stat-value">{formatUsd(result.monthly_property_tax)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">
                PMI {result.pmi_required ? '(required)' : '(not required)'}
              </span>
              <span className="stat-value">{formatUsd(result.monthly_pmi)}</span>
            </div>
          </div>

          <div className="stat-grid">
            <div className="stat stat-primary">
              <span className="stat-label">Monthly PITI</span>
              <span className="stat-value">{formatUsd(result.monthly_piti)}</span>
            </div>
            {result.monthly_tax_savings != null && (
              <div className="stat">
                <span className="stat-label">Tax savings</span>
                <span className="stat-value">{formatUsd(result.monthly_tax_savings)}</span>
              </div>
            )}
            {result.net_monthly_cost != null && (
              <div className="stat">
                <span className="stat-label">Net monthly cost</span>
                <span className="stat-value">{formatUsd(result.net_monthly_cost)}</span>
              </div>
            )}
          </div>

          {result.pmi_required && (
            <p className="chart-note">
              PMI applies below 20% down. Raising the down payment to{' '}
              {formatUsd(inputs.home_price * 0.2)} removes it.
            </p>
          )}
        </>
      )}
    </div>
  );
}
