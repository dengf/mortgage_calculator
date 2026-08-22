import React, { useState } from 'react';
import { downPaymentPercent } from '../scenario';

/**
 * Deposit entry that accepts either an amount or a percentage.
 *
 * Buyers hold the figure both ways — "we've saved $80,000" and "we're putting
 * 20% down" — and which one is natural depends on the conversation they last
 * had. The amount is the stored value either way, so the percentage view is
 * a lens rather than a second source of truth that could drift.
 */
export default function DownPaymentField({ label, scenario, onChange, money }) {
  const [mode, setMode] = useState('amount');
  const percent = downPaymentPercent(scenario);

  const shown =
    mode === 'percent'
      ? percent == null
        ? ''
        : Number(percent.toFixed(4))
      : scenario.downPayment;

  const handle = (value) => {
    if (value === '') return onChange('');
    if (mode === 'amount') return onChange(value);
    const price = Number(scenario.homePrice) || 0;
    return onChange(Math.round(price * (Number(value) / 100) * 100) / 100);
  };

  return (
    <label className="field">
      <span className="field-label">
        {label}
        <span className="field-unit-toggle" role="group">
          {['amount', 'percent'].map((m) => (
            <button
              key={m}
              type="button"
              className={m === mode ? 'field-unit active' : 'field-unit'}
              aria-pressed={m === mode}
              // Inside a <label>: without this, clicking focuses the input
              // and the toggle reads as part of the field's own control.
              onClick={(e) => {
                e.preventDefault();
                setMode(m);
              }}
            >
              {m === 'amount' ? money : '%'}
            </button>
          ))}
        </span>
      </span>
      <div className="field-input">
        <input
          type="number"
          value={shown}
          step="any"
          min={0}
          onChange={(e) => handle(e.target.value === '' ? '' : Number(e.target.value))}
        />
        <span className="field-suffix">{mode === 'amount' ? money : '%'}</span>
      </div>
      {mode === 'amount' && percent != null && (
        <span className="field-hint">{percent.toFixed(1)}% of price</span>
      )}
    </label>
  );
}
