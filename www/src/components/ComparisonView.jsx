import React, { useEffect, useMemo, useRef, useState } from 'react';
import ScenarioFields from './ScenarioFields';
import CalcError from './CalcError';
import ComparisonEntryRow from './ComparisonEntryRow';
import SavedScenarios from './SavedScenarios';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { DEFAULT_SCENARIO, useScenarioSummary } from '../scenario';

let nextId = 0;
const newId = () => `entry-${nextId++}`;

/** A preset's name in the reader's language, falling back to the English
 *  rendering the core ships alongside the code. */
function presetName(preset, t) {
  return preset.label_message
    ? t(preset.label_message.code, preset.label_message.params)
    : preset.label;
}

function presetToEntry(preset, t) {
  const isFixed = preset.rate_type.kind === 'fixed';
  return {
    id: newId(),
    // Named in the reader's language at the moment it is added. The label is
    // an editable field on the row from then on, so it belongs to the user
    // rather than re-translating under them on a language switch.
    label: presetName(preset, t),
    kind: isFixed ? 'fixed' : 'floating',
    ratePercent: isFixed ? preset.rate_type.rate_percent : 6.5,
    baseRatePercent: isFixed ? 4.3 : preset.rate_type.base_rate_percent,
    spreadPercent: isFixed ? 2 : preset.rate_type.spread_percent,
    termYears: preset.term_years,
  };
}

function blankEntry(t) {
  return {
    id: newId(),
    label: t('cmp.customScenario'),
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
      : {
          kind: 'floating',
          base_rate_percent: entry.baseRatePercent,
          spread_percent: entry.spreadPercent,
        };
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
  const { principal } = useScenarioSummary(wasmModule, scenario);
  const [presets, setPresets] = useState([]);
  // The market the current rows were seeded for, so a region switch reseeds.
  const seededFor = useRef(null);
  const [entries, setEntries] = useState([]);

  useEffect(() => {
    if (!wasmModule) return;
    // Which floating index applies is a fact about the market the property
    // is in, not a preference: SGD loans reference SORA, and have not
    // referenced SIBOR since MAS discontinued it after 31 December 2024.
    const loaded = wasmModule.get_common_rate_presets?.(region) ?? [];
    setPresets(loaded);
    if (loaded.length > 0 && seededFor.current !== region) {
      // Re-seeded when the market changes, and the rows that were there are
      // discarded. Carrying them over looks kinder and is worse: a "30-Year
      // Fixed" at 6.5% is not a Singapore scenario -- that product does not
      // exist there and the rate is several times the market -- so the rows
      // would keep computing, relabel themselves S$, and state a quote no
      // Singapore bank offers. Switching region is a deliberate act meaning
      // "show me the other market".
      seededFor.current = region;
      // Seed with two contrasting options when the list is long enough, but
      // index defensively: reaching straight into [0] and [2] threw a
      // TypeError and took down the whole tab if the preset list was ever
      // shorter than three, or absent from a stale cached wasm build.
      const seed = [loaded[0], loaded[2] ?? loaded[1]].filter(Boolean);
      setEntries(seed.map((p) => presetToEntry(p, t)));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wasmModule, region]);

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
  // Which row wins on each measure, and what the choice between them costs,
  // are read off the comparison rather than recomputed here -- see
  // mortgage-calc/src/comparison.rs. The verdict is absent for fewer than
  // two rows, since "cheapest" of a one-line table says nothing.
  const verdict = result?.verdict ?? null;

  // The classic fixed-term trade-off: the option that costs least overall
  // usually costs most each period. Rust says which and by how much; the
  // sentence is composed here, in the reader's language.
  const tradeoff = (() => {
    if (!verdict) return null;
    if (verdict.kind === 'outright') {
      return t('cmp.outright', { label: rows[verdict.cheaper]?.label });
    }
    return t('cmp.tradeoff', {
      cheaper: rows[verdict.cheaper]?.label,
      lighter: rows[verdict.lighter]?.label,
      paymentDelta: formatMoney(verdict.payment_delta),
      interestDelta: formatMoney(verdict.interest_delta),
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
        wasmModule={wasmModule}
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
            onClick={() => setEntries((prev) => [...prev, presetToEntry(preset, t)])}
          >
            + {presetName(preset, t)}
          </button>
        ))}
        <button
          className="link-button"
          onClick={() => setEntries((prev) => [...prev, blankEntry(t)])}
        >
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

      <CalcError result={result} />

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
              {result.rows.map((row, index) => (
                <tr key={row.label}>
                  <td>{row.label}</td>
                  <td>{row.effective_rate_percent.toFixed(3)}%</td>
                  <td>{t('duration.years', { years: row.term_years })}</td>
                  <td className={index === verdict?.cheapest_payment ? 'best' : undefined}>
                    {formatMoney(row.payment)}
                  </td>
                  <td className={index === verdict?.cheapest_total_paid ? 'best' : undefined}>
                    {formatMoney(row.total_paid)}
                  </td>
                  <td className={index === verdict?.cheapest_interest ? 'best' : undefined}>
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
        getCurrentInputs={() => ({
          homePrice: scenario.homePrice,
          downPayment: scenario.downPayment,
          frequency,
          entries,
        })}
        onLoad={(inputs) => {
          // Records saved before price and deposit moved into the shared
          // scenario hold only the loan amount. The split they were entered
          // with was never stored and cannot be recovered, so such a record
          // restores as a price with nothing down -- the one reading that
          // invents no figure the user did not type.
          const legacy = inputs.homePrice == null;
          onScenarioChange({
            ...scenario,
            homePrice: legacy ? inputs.principal : inputs.homePrice,
            downPayment: legacy ? 0 : inputs.downPayment,
            frequency: inputs.frequency ?? scenario.frequency,
          });
          setEntries((inputs.entries ?? []).map((e) => ({ ...e, id: newId() })));
        }}
      />
    </section>
  );
}
