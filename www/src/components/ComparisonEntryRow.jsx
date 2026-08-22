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
        // Typing a name claims it: the row stops being renamed from its
        // own figures from here on.
        onChange={(e) => set({ label: e.target.value, labelEdited: true })}
        placeholder={t('cmp.scenarioLabel')}
      />

      <div className="comparison-entry-kind">
        <button
          className={entry.kind === 'fixed' ? 'kind-toggle active' : 'kind-toggle'}
          onClick={() => set({ kind: 'fixed' })}
        >
          {t('cmp.fixed')}
        </button>
        <button
          className={entry.kind === 'reverting' ? 'kind-toggle active' : 'kind-toggle'}
          aria-pressed={entry.kind === 'reverting'}
          onClick={() => set({ kind: 'reverting' })}
        >
          {t('cmp.reverting')}
        </button>
        <button
          className={entry.kind === 'floating' ? 'kind-toggle active' : 'kind-toggle'}
          onClick={() => set({ kind: 'floating' })}
        >
          {t('cmp.floating')}
        </button>
      </div>

      {entry.kind === 'fixed' && (
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
      )}

      {/* Every field of a Singapore package is editable, because every one
          of them is negotiated: the index, the promotional spread, how long
          it lasts, and the spread it steps up to. The last is the one that
          decides most of the interest and the one buyers are least often
          shown. */}
      {entry.kind === 'reverting' && (
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
            <span>{t('cmp.initialSpread')}</span>
            <input
              type="number"
              step="any"
              value={entry.initialSpreadPercent}
              onChange={(e) => set({ initialSpreadPercent: Number(e.target.value) })}
            />
            <span className="unit">%</span>
          </label>
          <label className="comparison-entry-field">
            <span>{t('cmp.lockIn')}</span>
            <input
              type="number"
              step="any"
              min="0"
              value={entry.initialYears}
              onChange={(e) => set({ initialYears: Number(e.target.value) })}
            />
            <span className="unit">{t('cmp.yrs')}</span>
          </label>
          <label className="comparison-entry-field">
            <span>{t('cmp.thereafterSpread')}</span>
            <input
              type="number"
              step="any"
              value={entry.thereafterSpreadPercent}
              onChange={(e) => set({ thereafterSpreadPercent: Number(e.target.value) })}
            />
            <span className="unit">%</span>
          </label>
        </>
      )}

      {entry.kind === 'floating' && (
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
