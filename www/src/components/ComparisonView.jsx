import React, { useEffect, useMemo, useRef, useState } from 'react';
import ScenarioFields from './ScenarioFields';
import CalcError from './CalcError';
import ComparisonEntryRow from './ComparisonEntryRow';
import SavedScenarios from './SavedScenarios';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { DEFAULT_SCENARIO, useScenarioSummary } from '../scenario';
import { DEFAULT_RATE, rateFromPreset, toRateTypeDto } from '../rate';

let nextId = 0;
const newId = () => `entry-${nextId++}`;

/** A preset's name in the reader's language, falling back to the English
 *  rendering the core ships alongside the code. */
function presetName(preset, t) {
  return preset.label_message
    ? t(preset.label_message.code, preset.label_message.params)
    : preset.label;
}

// A row carries every field of every rate shape, so switching kind keeps
// what was typed rather than discarding it. Only the fields the active kind
// uses are sent across.
const EMPTY_ENTRY = {
  // The benchmark this row was seeded with, and whether the user has renamed
  // it. Together they decide whether the name still follows the figures.
  labelIndex: null,
  labelEdited: false,
  ...DEFAULT_RATE,
  termYears: 30,
};

function presetToEntry(preset, t) {
  return {
    ...EMPTY_ENTRY,
    id: newId(),
    // Named in the reader's language at the moment it is added. The label is
    // an editable field on the row from then on, so it belongs to the user
    // rather than re-translating under them on a language switch.
    label: presetName(preset, t),
    labelIndex: preset.index ?? null,
    ...rateFromPreset(preset),
    termYears: preset.term_years,
  };
}

function blankEntry(t) {
  // Named by the user from the start, so nothing regenerates over them.
  return { ...EMPTY_ENTRY, id: newId(), label: t('cmp.customScenario'), labelEdited: true };
}

/**
 * What the rows on this table assumed, once each.
 *
 * Rows are compared side by side, so a caveat repeated under every SORA
 * package would be read as decoration. Two rows quoted over the same
 * benchmark share one note; two quoted over different ones get their own,
 * because the figure held still is the whole content of the sentence.
 *
 * Which rows have anything to disclose is Rust's answer, asked per row --
 * see `rate_note`.
 */
function rateNotes(wasmModule, entries) {
  if (!wasmModule?.rate_note) return [];
  const seen = new Map();
  for (const entry of entries) {
    let note = null;
    try {
      note = wasmModule.rate_note(toRateTypeDto(entry));
    } catch {
      note = null;
    }
    if (note && !seen.has(note.text)) seen.set(note.text, note);
  }
  return [...seen.values()];
}

function toWasmEntry(entry) {
  // An entry is a rate with a name and a term on it, so the rate travels the
  // same way it does from every other tab.
  return {
    label: entry.label,
    rate_type: toRateTypeDto(entry),
    term_years: entry.termYears,
  };
}

export default function ComparisonView({
  wasmModule,
  region,
  scenario = DEFAULT_SCENARIO,
  onScenarioChange,
  dataVersion,
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

  const notes = useMemo(() => rateNotes(wasmModule, entries), [wasmModule, entries]);

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
    setEntries((prev) =>
      prev.map((e) => (e.id === id ? { ...updated, label: nameFor(updated, e) } : e)),
    );
  };

  /**
   * A row's name after an edit.
   *
   * A row seeded from a preset is named from its own figures, so editing
   * those figures renames it -- otherwise a row reads "then + 0.60%" while
   * computing 1.50%. Once the user types their own name it is theirs, and
   * nothing overwrites it.
   */
  function nameFor(updated, previous) {
    if (updated.labelEdited || !updated.labelIndex) return updated.label;
    if (updated.label !== previous.label) return updated.label;
    const described = wasmModule?.describe_rate?.({
      rate_type: toWasmEntry(updated).rate_type,
      term_years: updated.termYears,
      index: updated.labelIndex,
    });
    return described ? t(described.code, described.params) : updated.label;
  }

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

      {notes.map((note) => (
        <p className="rate-note" role="note" key={note.text}>
          {t(note.code, note.params) || note.text}
        </p>
      ))}

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
                  <td>
                    {row.effective_rate_percent.toFixed(3)}%
                    {row.thereafter_rate_percent != null && (
                      <span className="cmp-thereafter">
                        {t('rate.thenRate', {
                          rate: row.thereafter_rate_percent.toFixed(3),
                        })}
                      </span>
                    )}
                  </td>
                  <td>{t('duration.years', { years: row.term_years })}</td>
                  <td className={index === verdict?.cheapest_payment ? 'best' : undefined}>
                    {formatMoney(row.payment)}
                    {/* The promotional instalment on its own is the number a
                        buyer decides on and the one that changes. Showing
                        only it is how a package gets compared on its teaser. */}
                    {row.payment_after_reversion != null && (
                      <span className="cmp-thereafter">
                        {t('rate.thenPayment', {
                          payment: formatMoney(row.payment_after_reversion),
                        })}
                      </span>
                    )}
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
        dataVersion={dataVersion}
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
          // Merged over the defaults, so a row saved before a rate shape
          // existed still has every field that shape needs. Without this a
          // record from before reverting rates would restore with undefined
          // spreads the moment its kind was switched, and undefined crosses
          // the wasm boundary as a calculation, not an error.
          setEntries((inputs.entries ?? []).map((e) => ({ ...EMPTY_ENTRY, ...e, id: newId() })));
        }}
      />
    </section>
  );
}
