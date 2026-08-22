import React, { useEffect, useMemo, useState } from 'react';
import ScenarioFields from './ScenarioFields';
import ComparisonEntryRow from './ComparisonEntryRow';
import SavedScenarios from './SavedScenarios';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { DEFAULT_SCENARIO, principalOf } from '../scenario';


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

export default function ComparisonView({
  wasmModule,
  region,
  scenario = DEFAULT_SCENARIO,
  onScenarioChange,
}) {
  const { t } = useI18n();
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);
  // Rate and term vary per comparison row, so only the amount and cadence
  // come from the shared scenario.
  const { frequency } = scenario;
  const principal = principalOf(scenario);
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
    if (!allFilled(scenario.homePrice, scenario.downPayment)) return null;
    return wasmModule.calculate_comparison({
      principal,
      frequency,
      entries: entries.map(toWasmEntry),
    });
  }, [wasmModule, principal, frequency, entries]);

  const rows = result?.rows ?? [];
  const lowestBy = (key) =>
    rows.length ? rows.reduce((best, r) => (r[key] < best[key] ? r : best)) : null;
  const cheapestPayment = lowestBy('payment');
  const cheapestInterest = lowestBy('total_interest');
  const cheapestTotal = lowestBy('total_paid');

  // The classic fixed-term trade-off: the option that costs least overall
  // usually costs most each month. Say which, and by how much.
  const tradeoff = (() => {
    if (rows.length < 2 || !cheapestPayment || !cheapestInterest) return null;
    if (cheapestPayment === cheapestInterest) {
      return t('cmp.outright', { label: cheapestInterest.label });
    }
    return t('cmp.tradeoff', {
      cheaper: cheapestInterest.label,
      lighter: cheapestPayment.label,
      paymentDelta: formatMoney(cheapestInterest.payment - cheapestPayment.payment),
      interestDelta: formatMoney(cheapestPayment.total_interest - cheapestInterest.total_interest),
    });
  })();

  const updateEntry = (id, updated) => {
    setEntries((prev) => prev.map((e) => (e.id === id ? updated : e)));
  };

  const removeEntry = (id) => {
    setEntries((prev) => prev.filter((e) => e.id !== id));
  };

  return (
    <section className="panel">
      <ScenarioFields
        scenario={scenario}
        onChange={onScenarioChange}
        money={money}
        formatMoney={formatMoney}
        fields={['price', 'downPayment', 'frequency']}
      />

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
                <th>{t('cmp.rate')}</th>
                <th>{t('cmp.term')}</th>
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
                  <td>{t('duration.years', { years: row.term_years })}</td>
                  <td className={row === cheapestPayment ? 'best' : undefined}>
                    {formatMoney(row.payment)}
                  </td>
                  <td className={row === cheapestTotal ? 'best' : undefined}>
                    {formatMoney(row.total_paid)}
                  </td>
                  <td className={row === cheapestInterest ? 'best' : undefined}>
                    {formatMoney(row.total_interest)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* The difference is the entire question a comparison is asked to
          answer, and the table never stated it — two columns of figures and
          the arithmetic left to the reader. */}
      {tradeoff && <div className="cmp-tradeoff">{tradeoff}</div>}

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
