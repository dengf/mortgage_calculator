import React, { useEffect, useMemo, useRef, useState } from 'react';
import NumberField from './NumberField';
import CalcError from './CalcError';
import RateFields from './RateFields';
import SavedScenarios from './SavedScenarios';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { describeDuration, formatDuration, periodsInYears } from '../duration';
import { DEFAULT_RATE, normalizeRate, rateValues, toRateTypeDto } from '../rate';
import { seedRateForRegion } from '../scenario';
import { useRegionAwareCurrentInputs } from '../currentInputs';

const DEFAULTS = {
  currentBalance: 300000,
  currentRate: 7.5,
  remainingPeriods: 300,
  // The loan being refinanced *into* is a quote like any other, and in
  // Singapore refinancing means moving onto another package that steps up
  // after its own lock-in. A single "New rate" box could only describe the
  // promotional years of it -- which is exactly the number a switch gets
  // sold on, and the first one to stop being true.
  newRate: { ...DEFAULT_RATE, ratePercent: 6.0 },
  newTermYears: 30,
  closingCosts: 6000,
};

export default function RefinanceCalculator({ wasmModule, region, dataVersion }) {
  const { t } = useI18n();
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);
  const [currentBalance, setCurrentBalance] = useState(DEFAULTS.currentBalance);
  const [currentRate, setCurrentRate] = useState(DEFAULTS.currentRate);
  const [remainingPeriods, setRemainingPeriods] = useState(DEFAULTS.remainingPeriods);
  const [newRate, setNewRate] = useState(DEFAULTS.newRate);
  const [newTermYears, setNewTermYears] = useState(DEFAULTS.newTermYears);
  const [closingCosts, setClosingCosts] = useState(DEFAULTS.closingCosts);
  const seededFor = useRef(null);

  useEffect(() => {
    if (!wasmModule || seededFor.current === region) return;
    seededFor.current = region;
    const seeded = seedRateForRegion(wasmModule, region, {
      rate: newRate,
      termYears: newTermYears,
    });
    setNewRate(seeded.rate);
    setNewTermYears(seeded.termYears);
  }, [wasmModule, region]);

  useRegionAwareCurrentInputs({
    wasmModule,
    storageKey: 'refinance',
    region,
    getCurrentInputs: () => ({
      currentBalance,
      currentRate,
      remainingPeriods,
      newRate,
      newTermYears,
      closingCosts,
    }),
    onLoad: (inputs) => {
      setCurrentBalance(inputs.currentBalance);
      setCurrentRate(inputs.currentRate);
      setRemainingPeriods(inputs.remainingPeriods);
      setNewRate(normalizeRate(inputs.newRate));
      setNewTermYears(inputs.newTermYears);
      setClosingCosts(inputs.closingCosts);
    },
    reseedForRegion: (rest) => {
      const seeded = seedRateForRegion(wasmModule, region, {
        rate: normalizeRate(rest.newRate),
        termYears: rest.newTermYears,
      });
      return { ...rest, newRate: seeded.rate, newTermYears: seeded.termYears };
    },
    // The restore effect already decided whether to reseed for this region;
    // stop the plain reseed effect above from redundantly re-firing for it.
    onHydrated: () => {
      seededFor.current = region;
    },
    dataVersion,
    defaultInputs: DEFAULTS,
  });

  const newRateType = toRateTypeDto(newRate);
  const newRateKey = JSON.stringify(newRateType);

  const result = useMemo(() => {
    if (!wasmModule) return null;
    if (
      !allFilled(
        currentBalance,
        currentRate,
        remainingPeriods,
        ...rateValues(newRate),
        newTermYears,
        closingCosts,
      )
    )
      return null;
    return wasmModule.calculate_refinance({
      current_balance: currentBalance,
      current_annual_rate_percent: currentRate,
      remaining_periods: remainingPeriods,
      new_rate: newRateType,
      new_term_years: newTermYears,
      closing_costs: closingCosts,
      frequency: 'monthly',
    });
  }, [
    wasmModule,
    currentBalance,
    currentRate,
    remainingPeriods,
    newRateKey,
    newTermYears,
    closingCosts,
  ]);

  // Refinancing into a fresh 30-year loan when 25 years remain lowers the
  // payment and lengthens the debt. Both facts matter.
  //
  // Refinance quotes are monthly here because `remainingPeriods` is entered
  // as a number of monthly payments; the years-to-periods conversion and
  // the years/months breakdown are both the core's either way.
  const newPeriods = periodsInYears(wasmModule, newTermYears, 'monthly');
  const newTerm = describeDuration(wasmModule, newPeriods, 'monthly');
  const remaining = describeDuration(wasmModule, remainingPeriods, 'monthly');
  const extra = describeDuration(wasmModule, newPeriods - Number(remainingPeriods), 'monthly');
  const termExtension =
    extra.total_months > 0
      ? t('refi.termWarning', {
          newTerm: formatDuration(newTerm, t),
          remaining: formatDuration(remaining, t),
          extra: formatDuration(extra, t),
        })
      : null;

  return (
    <section className="panel">
      <div className="panel-form">
        <NumberField
          label={t('refi.currentBalance')}
          value={currentBalance}
          onChange={setCurrentBalance}
          suffix={money}
          min={0}
          grouped
        />
        <NumberField
          label={t('refi.currentRate')}
          value={currentRate}
          onChange={setCurrentRate}
          suffix="%"
          min={0}
        />
        <NumberField
          label={t('refi.remainingPeriods')}
          value={remainingPeriods}
          onChange={setRemainingPeriods}
          suffix={t('refi.months')}
          min={1}
          step="1"
        />
        <RateFields
          rate={newRate}
          onChange={setNewRate}
          label="refi.newRate"
          wasmModule={wasmModule}
        />
        <NumberField
          label={t('refi.newTerm')}
          value={newTermYears}
          onChange={setNewTermYears}
          suffix={t('field.years')}
          min={1}
        />
        <NumberField
          label={t('refi.closingCosts')}
          value={closingCosts}
          onChange={setClosingCosts}
          suffix={money}
          min={0}
          grouped
        />
      </div>

      {/* "Lifetime savings" is honest as total cash out the door, but it
          silently compares the years left on the current loan against a
          fresh full term. A reader shouldn't have to notice that themselves. */}
      {termExtension && <div className="refi-term-warning">{termExtension}</div>}

      <div className="panel-results" aria-live="polite">
        <CalcError result={result} />
        {result && !result.error && (
          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">{t('refi.currentPayment')}</span>
              <span className="stat-value">{formatMoney(result.current_payment)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('refi.newPayment')}</span>
              <span className="stat-value">{formatMoney(result.new_payment)}</span>
              {result.new_payment_after_reversion != null && (
                <span className="stat-note">
                  {t('rate.thenPayment', {
                    payment: formatMoney(result.new_payment_after_reversion),
                  })}
                </span>
              )}
            </div>
            <div className="stat stat-primary">
              <span className="stat-label">{t('refi.monthlySavings')}</span>
              <span className="stat-value">{formatMoney(result.payment_savings)}</span>
              {/* Named for what it is once the new loan can step up: the
                  saving during the lock-in, not for the life of the loan. */}
              {result.new_payment_after_reversion != null && (
                <span className="stat-note">{t('refi.savingsDuringLockIn')}</span>
              )}
            </div>
            <div className="stat">
              <span className="stat-label">{t('refi.breakEven')}</span>
              <span className="stat-value">
                {result.break_even_periods
                  ? formatDuration(
                      describeDuration(wasmModule, result.break_even_periods, 'monthly'),
                      t,
                    )
                  : t('refi.never')}
              </span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('refi.lifetimeSavingsNet')}</span>
              <span className="stat-value">{formatMoney(result.lifetime_savings)}</span>
            </div>
          </div>
        )}
      </div>

      <SavedScenarios
        wasmModule={wasmModule}
        dataVersion={dataVersion}
        calculatorKind="refinance"
        getCurrentInputs={() => ({
          currentBalance,
          currentRate,
          remainingPeriods,
          newRate,
          newTermYears,
          closingCosts,
        })}
        onLoad={(inputs) => {
          setCurrentBalance(inputs.currentBalance);
          setCurrentRate(inputs.currentRate);
          setRemainingPeriods(inputs.remainingPeriods);
          // Records saved before a rate was a shape hold a bare number.
          setNewRate(normalizeRate(inputs.newRate));
          setNewTermYears(inputs.newTermYears);
          setClosingCosts(inputs.closingCosts);
        }}
      />
    </section>
  );
}
