import React from 'react';

export default function ComparisonEntryRow({ entry, onChange, onRemove }) {
  const set = (patch) => onChange({ ...entry, ...patch });

  return (
    <div className="comparison-entry">
      <input
        className="comparison-entry-label"
        value={entry.label}
        onChange={(e) => set({ label: e.target.value })}
        placeholder="Scenario label"
      />

      <div className="comparison-entry-kind">
        <button
          className={entry.kind === 'fixed' ? 'kind-toggle active' : 'kind-toggle'}
          onClick={() => set({ kind: 'fixed' })}
        >
          Fixed
        </button>
        <button
          className={entry.kind === 'floating' ? 'kind-toggle active' : 'kind-toggle'}
          onClick={() => set({ kind: 'floating' })}
        >
          Floating
        </button>
      </div>

      {entry.kind === 'fixed' ? (
        <label className="comparison-entry-field">
          <span>Rate</span>
          <input
            type="number"
            step="any"
            value={entry.ratePercent}
            onChange={(e) => set({ ratePercent: Number(e.target.value) })}
          />
          <span className="unit">%</span>
        </label>
      ) : (
        <>
          <label className="comparison-entry-field">
            <span>Base</span>
            <input
              type="number"
              step="any"
              value={entry.baseRatePercent}
              onChange={(e) => set({ baseRatePercent: Number(e.target.value) })}
            />
            <span className="unit">%</span>
          </label>
          <label className="comparison-entry-field">
            <span>Spread</span>
            <input
              type="number"
              step="any"
              value={entry.spreadPercent}
              onChange={(e) => set({ spreadPercent: Number(e.target.value) })}
            />
            <span className="unit">%</span>
          </label>
        </>
      )}

      <label className="comparison-entry-field">
        <span>Term</span>
        <input
          type="number"
          step="1"
          min="1"
          value={entry.termYears}
          onChange={(e) => set({ termYears: Number(e.target.value) })}
        />
        <span className="unit">yrs</span>
      </label>

      <button className="link-button danger" onClick={onRemove}>
        Remove
      </button>
    </div>
  );
}
