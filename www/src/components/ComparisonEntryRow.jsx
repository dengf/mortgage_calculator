import React from 'react';
import { RateKindToggle } from './RateFields';
import { RATE_FIELDS } from '../rate';
import { useI18n } from '../i18n';

export default function ComparisonEntryRow({ entry, onChange, onRemove }) {
  const { t } = useI18n();
  const set = (patch) => onChange({ ...entry, ...patch });
  // The same list the panel forms render, in the compact styling a row of
  // scenarios needs. Which fields a shape has is decided in one place.
  const fields = RATE_FIELDS[entry.kind] ?? RATE_FIELDS.fixed;

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

      <RateKindToggle kind={entry.kind} onChange={(kind) => set({ kind })} />

      {fields.map((field) => (
        <label className="comparison-entry-field" key={field.key}>
          <span>{t(field.label)}</span>
          <input
            type="number"
            step="any"
            min={field.min}
            value={entry[field.key]}
            onChange={(e) => set({ [field.key]: Number(e.target.value) })}
          />
          <span className="unit">{t(field.unit)}</span>
        </label>
      ))}

      <label className="comparison-entry-field">
        <span>{t('cmp.term')}</span>
        <input
          type="number"
          step="1"
          min="1"
          value={entry.termYears}
          onChange={(e) => set({ termYears: Number(e.target.value) })}
        />
        <span className="unit">{t('rate.yrs')}</span>
      </label>

      <button className="link-button danger" onClick={onRemove}>
        {t('cmp.remove')}
      </button>
    </div>
  );
}
