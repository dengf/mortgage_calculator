import React, { useEffect, useMemo, useState } from 'react';
import NumberField from './NumberField';
import ComparisonEntryRow from './ComparisonEntryRow';
import SavedScenarios from './SavedScenarios';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';


let nextId = 0;
const newId = () => `entry-${nextId++}`;

function presetToEntry(preset) {
  const isFixed = preset.rate_type.kind === 'fixed';
  return {
    id: newId(),
    label: preset.label,
    kind: isFixed ? 'fixed' : 'floating',
    ratePercent: isFixed ? preset.rate_type.rate_percent : 6.5,
    baseRatePercent: isFixed ? 4.3 : preset.rate_type.base_rate_percent,
    spreadPercent: isFixed ? 2 : preset.rate_type.spread_percent,
    termYears: preset.term_years,
  };
}

function blankEntry() {
  return {
    id: newId(),
    label: 'Custom scenario',
    kind: 'fixed',
    ratePercent: 6.5,
    baseRatePercent: 4.3,
    spreadPercent: 2,
    termYears: 30,
  };
}

function toWasmEntry(entry) {
  const rate_type =
    entry.kind === 'fixed'
      ? { kind: 'fixed', rate_percent: entry.ratePercent }
      : { kind: 'floating', base_rate_percent: entry.baseRatePercent, spread_percent: entry.spreadPercent };
  return { label: entry.label, rate_type, term_years: entry.termYears };
}

export default function ComparisonView({ wasmModule, region }) {
  const { t } = useI18n();
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);
  const [principal, setPrincipal] = useState(400000);
  const [frequency, setFrequency] = useState('monthly');
  const [presets, setPresets] = useState([]);
  const [entries, setEntries] = useState([]);

  useEffect(() => {
    if (!wasmModule) return;
    const loaded = wasmModule.get_common_rate_presets?.() ?? [];
    setPresets(loaded);
    if (entries.length === 0 && loaded.length > 0) {
      // Seed with two contrasting terms when the list is long enough, but
      // index defensively: reaching straight into [0] and [2] threw a
      // TypeError and took down the whole tab if the preset list was ever
      // shorter than three, or absent from a stale cached wasm build.
      const seed = [loaded[0], loaded[2] ?? loaded[1]].filter(Boolean);
      setEntries(seed.map(presetToEntry));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wasmModule]);

  const result = useMemo(() => {
    if (!wasmModule || entries.length === 0) return null;
    return wasmModule.calculate_comparison({
      principal,
      frequency,
      entries: entries.map(toWasmEntry),
    });
  }, [wasmModule, principal, frequency, entries]);

  const updateEntry = (id, updated) => {
    setEntries((prev) => prev.map((e) => (e.id === id ? updated : e)));
  };

  const removeEntry = (id) => {
    setEntries((prev) => prev.filter((e) => e.id !== id));
  };

  return (
    <section className="panel">
      <div className="panel-form">
        <NumberField label={t('field.loanAmount')} value={principal} onChange={setPrincipal} suffix={money} min={0} />
        <label className="field">
          <span className="field-label">{t('field.paymentFrequency')}</span>
          <select className="field-select" value={frequency} onChange={(e) => setFrequency(e.target.value)}>
            <option value="monthly">{t('freq.monthly')}</option>
            <option value="biweekly">{t('freq.biweekly')}</option>
            <option value="weekly">{t('freq.weekly')}</option>
          </select>
        </label>
      </div>

      <div className="comparison-presets">
        <span className="field-label">{t('cmp.quickAdd')}</span>
        {presets.map((preset) => (
          <button
            key={preset.label}
            className="link-button"
            onClick={() => setEntries((prev) => [...prev, presetToEntry(preset)])}
          >
            + {preset.label}
          </button>
        ))}
        <button className="link-button" onClick={() => setEntries((prev) => [...prev, blankEntry()])}>
          + {t('cmp.custom')}
        </button>
      </div>

      <div className="comparison-entries">
        {entries.map((entry) => (
          <ComparisonEntryRow
            key={entry.id}
            entry={entry}
            onChange={(updated) => updateEntry(entry.id, updated)}
            onRemove={() => removeEntry(entry.id)}
          />
        ))}
        {entries.length === 0 && <p className="saved-scenarios-empty">{t('cmp.addScenario')}</p>}
      </div>

      {result?.error && <div className="error">{result.error}</div>}

      {result?.rows?.length > 0 && (
        <div className="schedule-table-wrap">
          <table className="schedule-table">
            <thead>
              <tr>
                <th>{t('cmp.scenario')}</th>
                <th>Rate</th>
                <th>Term</th>
                <th>{t('cmp.payment')}</th>
                <th>{t('cmp.totalPaid')}</th>
                <th>{t('cmp.totalInterest')}</th>
              </tr>
            </thead>
            <tbody>
              {result.rows.map((row) => (
                <tr key={row.label}>
                  <td>{row.label}</td>
                  <td>{row.effective_rate_percent.toFixed(3)}%</td>
                  <td>{row.term_years}yr</td>
                  <td>{formatMoney(row.payment)}</td>
                  <td>{formatMoney(row.total_paid)}</td>
                  <td>{formatMoney(row.total_interest)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="comparison"
        getCurrentInputs={() => ({ principal, frequency, entries })}
        onLoad={(inputs) => {
          setPrincipal(inputs.principal);
          setFrequency(inputs.frequency);
          setEntries(inputs.entries.map((e) => ({ ...e, id: newId() })));
        }}
      />
    </section>
  );
}
