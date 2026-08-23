import React, { useId } from 'react';
import NumberField from './NumberField';
import RateNote from './RateNote';
import { RATE_FIELDS, RATE_KINDS, normalizeRate } from '../rate';
import { useI18n } from '../i18n';

/**
 * Picks which shape a quote has. Rendered wherever a rate is entered, so
 * that a package that steps up is available on every tab rather than only
 * on Compare.
 */
export function RateKindToggle({ kind, onChange }) {
  const { t } = useI18n();
  return (
    <div className="rate-kind">
      {RATE_KINDS.map((option) => (
        <button
          key={option}
          type="button"
          className={kind === option ? 'kind-toggle active' : 'kind-toggle'}
          aria-pressed={kind === option}
          onClick={() => onChange(option)}
        >
          {t(`rate.${option}`)}
        </button>
      ))}
    </div>
  );
}

/**
 * The inputs one quoted rate is made of, in panel-form styling.
 *
 * Which inputs those are comes from `RATE_FIELDS`, the same list the compact
 * Compare rows read, so the two can't drift apart.
 */
export function BaseFloatsToggle({ checked, onChange }) {
  const { t } = useI18n();
  return (
    <label className="field field-check">
      <input
        type="checkbox"
        checked={Boolean(checked)}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className="field-label">{t('rate.baseFloats')}</span>
    </label>
  );
}

export default function RateFields({ rate, onChange, label = 'field.interestRate', wasmModule }) {
  const { t } = useI18n();
  const shape = normalizeRate(rate);
  const fields = RATE_FIELDS[shape.kind] ?? RATE_FIELDS.fixed;
  // A group of buttons, not a labelled control: `<label>` around them would
  // name whichever one a click landed on rather than the choice itself.
  const labelId = useId();

  return (
    <>
      <div className="field" role="group" aria-labelledby={labelId}>
        <span className="field-label" id={labelId}>
          {t(label)}
        </span>
        <RateKindToggle kind={shape.kind} onChange={(kind) => onChange({ ...shape, kind })} />
      </div>

      {fields.map((field) => (
        <NumberField
          key={field.key}
          label={t(field.label)}
          value={shape[field.key]}
          onChange={(value) => onChange({ ...shape, [field.key]: value })}
          suffix={t(field.unit)}
          min={field.min ?? 0}
        />
      ))}

      {/* Asked only of a package that steps up. A floating quote is
          benchmark-based by construction and a fixed one has no base at
          all, so for those two the answer is not the user's to give. */}
      {shape.kind === 'reverting' && (
        <BaseFloatsToggle
          checked={shape.baseFloats}
          onChange={(baseFloats) => onChange({ ...shape, baseFloats })}
        />
      )}

      <RateNote wasmModule={wasmModule} rate={shape} />
    </>
  );
}
