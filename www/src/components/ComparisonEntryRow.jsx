import React from 'react';
import { useI18n } from '../i18n';

export default function ComparisonEntryRow({ entry, onChange, onRemove }) {
  const { t } = useI18n();
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
          {t('cmp.fixed')}
        </button>
        <button
          className={entry.kind === 'floating' ? 'kind-toggle active' : 'kind-toggle'}
          onClick={() => set({ kind: 'floating' })}
        >
          {t('cmp.floating')}
        </button>
      </div>

      {entry.kind === 'fixed' ? (
        <label className="comparison-entry-field">
          <span>{t('cmp.rate')}</span>
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
            <span>{t('cmp.base')}</span>
            <input
              type="number"
              step="any"
              value={entry.baseRatePercent}
              onChange={(e) => set({ baseRatePercent: Number(e.target.value) })}
            />
            <span className="unit">%</span>
          </label>
          <label className="comparison-entry-field">
            <span>{t('cmp.spread')}</span>
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
        <span>{t('cmp.term')}</span>
        <input
          type="number"
          step="1"
          min="1"
          value={entry.termYears}
          onChange={(e) => set({ termYears: Number(e.target.value) })}
        />
        <span className="unit">{t('cmp.yrs')}</span>
      </label>

      <button className="link-button danger" onClick={onRemove}>
        {t('cmp.remove')}
      </button>
    </div>
  );
}
