import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import ScenarioFields from './ScenarioFields';
import SavedScenarios from './SavedScenarios';
import { BalanceChart } from './Charts';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { formatDuration, payoffDate } from '../duration';
import { DEFAULT_SCENARIO, principalOf } from '../scenario';


const PERIODS_PER_YEAR = { monthly: 12, biweekly: 26, weekly: 52 };

function summarizeByYear(rows, periodsPerYear) {
  const years = [];
  for (let i = 0; i < rows.length; i += periodsPerYear) {
    const chunk = rows.slice(i, i + periodsPerYear);
    years.push({
      year: years.length + 1,
      paid: chunk.reduce((sum, r) => sum + r.payment, 0),
      principal: chunk.reduce((sum, r) => sum + r.principal_portion, 0),
      interest: chunk.reduce((sum, r) => sum + r.interest_portion, 0),
      remaining_balance: chunk[chunk.length - 1].remaining_balance,
    });
  }
  return years;
}

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
  const principal = principalOf(scenario);
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

  const periodsPerYear = PERIODS_PER_YEAR[frequency] ?? 12;

  const yearlyRows = useMemo(() => {
    if (!schedule?.rows?.length) return [];
    return summarizeByYear(schedule.rows, periodsPerYear);
  }, [schedule, periodsPerYear]);

  return (
    <section className="panel">
      <ScenarioFields
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
        />
      </div>

      {impact && !impact.error && (
        <div className="stat-grid" aria-live="polite">
          <div className="stat stat-primary">
            <span className="stat-label">{t('amort.timeSaved')}</span>
            <span className="stat-value">
              {formatDuration(impact.periods_saved, periodsPerYear, t)}
            </span>
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
            <span className="stat-value">
              {formatDuration(impact.payoff_periods, periodsPerYear, t)}
            </span>
            <span className="stat-note">
              {t('amort.payoffDate', {
                date: payoffDate(impact.payoff_periods, periodsPerYear, locale),
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
          periodsPerYear={periodsPerYear}
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
        getCurrentInputs={() => ({ principal, rate, termYears, frequency, extraPayment })}
        onLoad={(inputs) => {
          setPrincipal(inputs.principal);
          setRate(inputs.rate);
          setTermYears(inputs.termYears);
          setFrequency(inputs.frequency);
          setExtraPayment(inputs.extraPayment);
        }}
      />
    </section>
  );
}
