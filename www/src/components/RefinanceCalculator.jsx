import React, { useMemo, useState } from 'react';
import NumberField from './NumberField';
import SavedScenarios from './SavedScenarios';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { formatDuration } from '../duration';


export default function RefinanceCalculator({ wasmModule, region }) {
  const { t } = useI18n();
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);
  const [currentBalance, setCurrentBalance] = useState(300000);
  const [currentRate, setCurrentRate] = useState(7.5);
  const [remainingPeriods, setRemainingPeriods] = useState(300);
  const [newRate, setNewRate] = useState(6.0);
  const [newTermYears, setNewTermYears] = useState(30);
  const [closingCosts, setClosingCosts] = useState(6000);

  const result = useMemo(() => {
    if (!wasmModule) return null;
    if (!allFilled(currentBalance, currentRate, remainingPeriods, newRate, newTermYears, closingCosts))
      return null;
    return wasmModule.calculate_refinance({
      current_balance: currentBalance,
      current_annual_rate_percent: currentRate,
      remaining_periods: remainingPeriods,
      new_annual_rate_percent: newRate,
      new_term_years: newTermYears,
      closing_costs: closingCosts,
      frequency: 'monthly',
    });
  }, [wasmModule, currentBalance, currentRate, remainingPeriods, newRate, newTermYears, closingCosts]);

  // Refinancing into a fresh 30-year loan when 25 years remain lowers the
  // payment and lengthens the debt. Both facts matter.
  const extraPeriods = Math.round(Number(newTermYears) * 12) - Number(remainingPeriods);
  const termExtension =
    Number.isFinite(extraPeriods) && extraPeriods > 0
      ? t('refi.termWarning', {
          newTerm: formatDuration(Number(newTermYears) * 12, 12, t),
          remaining: formatDuration(remainingPeriods, 12, t),
          extra: formatDuration(extraPeriods, 12, t),
        })
      : null;

  return (
    <section className="panel">
      <div className="panel-form">
        <NumberField label={t('refi.currentBalance')} value={currentBalance} onChange={setCurrentBalance} suffix={money} min={0} />
        <NumberField label={t('refi.currentRate')} value={currentRate} onChange={setCurrentRate} suffix="%" min={0} />
        <NumberField
          label={t('refi.remainingPeriods')}
          value={remainingPeriods}
          onChange={setRemainingPeriods}
          suffix="months"
          min={1}
          step="1"
        />
        <NumberField label={t('refi.newRate')} value={newRate} onChange={setNewRate} suffix="%" min={0} />
        <NumberField label={t('refi.newTerm')} value={newTermYears} onChange={setNewTermYears} suffix={t('field.years')} min={1} />
        <NumberField label={t('refi.closingCosts')} value={closingCosts} onChange={setClosingCosts} suffix={money} min={0} />
      </div>

      {/* "Lifetime savings" is honest as total cash out the door, but it
          silently compares the years left on the current loan against a
          fresh full term. A reader shouldn't have to notice that themselves. */}
      {termExtension && <div className="refi-term-warning">{termExtension}</div>}

      <div className="panel-results" aria-live="polite">
        {result?.error && <div className="error">{result.error}</div>}
        {result && !result.error && (
          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">{t('refi.currentPayment')}</span>
              <span className="stat-value">{formatMoney(result.current_payment)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('refi.newPayment')}</span>
              <span className="stat-value">{formatMoney(result.new_payment)}</span>
            </div>
            <div className="stat stat-primary">
              <span className="stat-label">{t('refi.monthlySavings')}</span>
              <span className="stat-value">{formatMoney(result.payment_savings)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('refi.breakEven')}</span>
              <span className="stat-value">
                {result.break_even_periods ? `${result.break_even_periods} months` : 'Never'}
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
          setNewRate(inputs.newRate);
          setNewTermYears(inputs.newTermYears);
          setClosingCosts(inputs.closingCosts);
        }}
      />
    </section>
  );
}
