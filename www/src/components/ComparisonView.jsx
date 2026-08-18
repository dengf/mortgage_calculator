import React, { useEffect, useMemo, useState } from 'react';
import NumberField from './NumberField';
import ComparisonEntryRow from './ComparisonEntryRow';
import SavedScenarios from './SavedScenarios';

const formatMoney = (n) =>
  n == null ? '—' : n.toLocaleString('en-US', { style: 'currency', currency: 'USD' });

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

export default function ComparisonView({ wasmModule }) {
  const [principal, setPrincipal] = useState(400000);
  const [frequency, setFrequency] = useState('monthly');
  const [presets, setPresets] = useState([]);
  const [entries, setEntries] = useState([]);

  useEffect(() => {
    if (!wasmModule) return;
    const loaded = wasmModule.get_common_rate_presets();
    setPresets(loaded);
    if (entries.length === 0) {
      setEntries([presetToEntry(loaded[0]), presetToEntry(loaded[2])]);
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
        <NumberField label="Home loan amount" value={principal} onChange={setPrincipal} suffix="$" min={0} />
        <label className="field">
          <span className="field-label">Payment frequency</span>
          <select className="field-select" value={frequency} onChange={(e) => setFrequency(e.target.value)}>
            <option value="monthly">Monthly</option>
            <option value="biweekly">Bi-weekly</option>
            <option value="weekly">Weekly</option>
          </select>
        </label>
      </div>

      <div className="comparison-presets">
        <span className="field-label">Quick add:</span>
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
          + Custom
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
        {entries.length === 0 && <p className="saved-scenarios-empty">Add a scenario to compare.</p>}
      </div>

      {result?.error && <div className="error">{result.error}</div>}

      {result?.rows?.length > 0 && (
        <div className="schedule-table-wrap">
          <table className="schedule-table">
            <thead>
              <tr>
                <th>Scenario</th>
                <th>Rate</th>
                <th>Term</th>
                <th>Payment</th>
                <th>Total paid</th>
                <th>Total interest</th>
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
