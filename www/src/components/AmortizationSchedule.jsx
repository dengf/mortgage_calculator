import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import ScenarioFields from './ScenarioFields';
import SavedScenarios from './SavedScenarios';
import { BalanceChart } from './Charts';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { describeDuration, formatDuration, payoffDate } from '../duration';
import { DEFAULT_SCENARIO, useScenarioSummary } from '../scenario';

export default function AmortizationSchedule({
  wasmModule,
  region,
  scenario = DEFAULT_SCENARIO,
  onScenarioChange,
}) {
  const { t, locale } = useI18n();
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);
  // Extra payment stays local: it's a question about this loan, not part of
  // its terms, and carrying it onto the Payment tab would silently change
  // the headline figure there.
  const [extraPayment, setExtraPayment] = useState(0);
  const [showFullSchedule, setShowFullSchedule] = useState(false);

  const { rate, termYears, frequency } = scenario;
  const { principal } = useScenarioSummary(wasmModule, scenario);
  const loan = { principal, annual_rate_percent: rate, term_years: termYears, frequency };

  const schedule = useMemo(() => {
    if (!wasmModule) return null;
    if (!allFilled(scenario.homePrice, scenario.downPayment, rate, termYears)) return null;
    return wasmModule.calculate_amortization_schedule({ loan, extra_payment: extraPayment || 0 });
  }, [wasmModule, principal, rate, termYears, frequency, extraPayment]);

  const impact = useMemo(() => {
    if (!wasmModule || !extraPayment) return null;
    if (!allFilled(scenario.homePrice, scenario.downPayment, rate, termYears)) return null;
    return wasmModule.calculate_extra_payment_impact({ loan, extra_payment: extraPayment });
  }, [wasmModule, principal, rate, termYears, frequency, extraPayment]);

  // Cadence and every period-to-time conversion come from the core rather
  // than a table kept here -- see mortgage-core/src/frequency.rs.
  const saved = describeDuration(wasmModule, impact?.periods_saved, frequency);
  const payoff = describeDuration(wasmModule, impact?.payoff_periods, frequency);
  const plotted = describeDuration(wasmModule, schedule?.rows?.length, frequency);

  // Grouped by the same call that produced the rows, so the yearly totals and
  // the periods they sum are never the output of two separate calculations.
  const yearlyRows = schedule?.yearly ?? [];

  return (
    <section className="panel">
      <ScenarioFields
        wasmModule={wasmModule}
        scenario={scenario}
        onChange={onScenarioChange}
        money={money}
        formatMoney={formatMoney}
      />

      <div className="panel-form">
        <NumberField
          label={t('amort.extraPayment')}
          value={extraPayment}
          onChange={setExtraPayment}
          suffix={money}
          min={0}
          grouped
        />
      </div>

      {impact && !impact.error && (
        <div className="stat-grid" aria-live="polite">
          <div className="stat stat-primary">
            <span className="stat-label">{t('amort.timeSaved')}</span>
            <span className="stat-value">{formatDuration(saved, t)}</span>
            <span className="stat-note">
              {t('amort.payments', { count: impact.periods_saved })}
            </span>
          </div>
          <div className="stat">
            <span className="stat-label">{t('amort.interestSaved')}</span>
            <span className="stat-value">{formatMoney(impact.interest_saved)}</span>
          </div>
          <div className="stat">
            <span className="stat-label">{t('amort.newPayoff')}</span>
            <span className="stat-value">{formatDuration(payoff, t)}</span>
            <span className="stat-note">
              {t('amort.payoffDate', {
                date: payoffDate(payoff, locale),
              })}
            </span>
          </div>
        </div>
      )}

      {schedule?.error && <div className="error">{schedule.error}</div>}

      {schedule?.rows?.length > 0 && (
        <BalanceChart
          rows={schedule.rows}
          principal={principal}
          yearsPlotted={plotted.years_exact}
          formatMoney={formatMoney}
        />
      )}

      {yearlyRows.length > 0 && (
        <div className="schedule-table-wrap">
          <div className="schedule-table-header">
            <h3>{showFullSchedule ? t('amort.fullSchedule') : t('amort.yearlySummary')}</h3>
            <button className="link-button" onClick={() => setShowFullSchedule((v) => !v)}>
              {showFullSchedule ? t('amort.showYearly') : t('amort.showEvery')}
            </button>
          </div>
          <table className="schedule-table">
            <thead>
              <tr>
                <th>{showFullSchedule ? t('amort.period') : t('amort.year')}</th>
                <th>{t('amort.paid')}</th>
                <th>{t('amort.principal')}</th>
                <th>{t('amort.interest')}</th>
                <th>{t('amort.balance')}</th>
              </tr>
            </thead>
            <tbody>
              {(showFullSchedule ? schedule.rows : yearlyRows).map((row) => (
                <tr key={showFullSchedule ? row.period : row.year}>
                  <td>{showFullSchedule ? row.period : row.year}</td>
                  <td>{formatMoney(showFullSchedule ? row.payment : row.paid)}</td>
                  <td>{formatMoney(showFullSchedule ? row.principal_portion : row.principal)}</td>
                  <td>{formatMoney(showFullSchedule ? row.interest_portion : row.interest)}</td>
                  <td>{formatMoney(row.remaining_balance)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="amortization"
        getCurrentInputs={() => ({
          homePrice: scenario.homePrice,
          downPayment: scenario.downPayment,
          rate,
          termYears,
          frequency,
          extraPayment,
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
            rate: inputs.rate ?? scenario.rate,
            termYears: inputs.termYears ?? scenario.termYears,
            frequency: inputs.frequency ?? scenario.frequency,
          });
          setExtraPayment(inputs.extraPayment ?? 0);
        }}
      />
    </section>
  );
}
