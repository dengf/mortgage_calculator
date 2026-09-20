import React, { useId } from 'react';
import { useI18n } from '../i18n';

// Hand-rolled SVG rather than a charting library: these are two fixed chart
// shapes, and pulling in Recharts/Chart.js would cost more gzipped than the
// whole wasm module the app is built around.

// No colour constants here on purpose. These used to be `#4f8cff`/`#f59e0b`
// (the pre-rebrand blue/amber) and `#243044` for the gridlines, hardcoded
// past the token system, which produced three separate faults: the split
// bar below draws in `--accent`/`--gold` while its own legend drew in
// blue/amber, so the key disagreed with the chart it labelled; the grid was
// a dark-theme value painted on a white card in light theme; and no chart
// in the app carried the brand at all. Every colour now lives in main.css
// against the same tokens the bar uses -- `--accent` for what you owe,
// `--gold` for what the borrowing costs, `--line` for the grid -- so a
// theme switch moves the charts with everything else.

/** Viewbox units. The SVG scales to its container via viewBox + width 100%. */
const W = 300;
const H = 140;
const PAD_L = 4;
const PAD_R = 4;
const PAD_T = 6;
const PAD_B = 6;

function pathFrom(values, max, { close = false } = {}) {
  if (!values.length || max <= 0) return '';
  const innerW = W - PAD_L - PAD_R;
  const innerH = H - PAD_T - PAD_B;
  const x = (i) => PAD_L + (innerW * i) / Math.max(1, values.length - 1);
  const y = (v) => PAD_T + innerH * (1 - Math.min(1, Math.max(0, v / max)));

  const line = values.map((v, i) => `${i === 0 ? 'M' : 'L'}${x(i).toFixed(2)},${y(v).toFixed(2)}`);
  if (!close) return line.join(' ');
  return `${line.join(' ')} L${x(values.length - 1).toFixed(2)},${(H - PAD_B).toFixed(2)} L${x(0).toFixed(2)},${(H - PAD_B).toFixed(2)} Z`;
}

/**
 * Remaining balance and cumulative interest over the life of the loan, on a
 * shared vertical scale so the crossover point is honest — that point is
 * where interest paid to date overtakes what's still owed.
 */
export function BalanceChart({ rows, principal, yearsPlotted, formatMoney }) {
  const { t } = useI18n();
  const gradientId = useId();
  if (!rows?.length) return null;

  // One point per row would be thousands of nodes for a weekly 30-year
  // schedule; ~120 samples is past the resolution of the rendered width.
  // Walk the rows once accumulating interest, sampling as we go — indexing
  // back into `rows` per sample would make this quadratic.
  const step = Math.max(1, Math.ceil(rows.length / 120));
  const balances = [];
  const cumulativeInterest = [];
  let running = 0;
  for (let i = 0; i < rows.length; i += 1) {
    running += rows[i].interest_portion;
    if (i % step === 0 || i === rows.length - 1) {
      balances.push(rows[i].remaining_balance);
      cumulativeInterest.push(running);
    }
  }

  const totalInterest = running;
  const max = Math.max(principal, totalInterest, 1);

  return (
    <figure className="chart">
      <figcaption className="chart-title">{t('chart.balanceVsInterest')}</figcaption>
      <div className="chart-legend">
        <span className="chart-key chart-key-balance">
          <i /> {t('chart.remainingBalance')}
        </span>
        <span className="chart-key chart-key-interest">
          <i /> {t('chart.interestToDate')}
        </span>
      </div>
      <svg
        className="chart-svg"
        viewBox={`0 0 ${W} ${H}`}
        preserveAspectRatio="none"
        role="img"
        aria-label={t('chart.balanceAria', {
          principal: formatMoney(principal),
          interest: formatMoney(totalInterest),
        })}
      >
        <defs>
          {/* `stop-color` is set from CSS rather than the `stopColor`
              attribute: an SVG presentation *attribute* cannot take a
              `var()`, but the corresponding CSS property can, which is what
              lets the fade under the balance line follow `--accent` through
              a theme change. */}
          <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
            <stop className="chart-area-from" offset="0%" />
            <stop className="chart-area-to" offset="100%" />
          </linearGradient>
        </defs>
        {[0.25, 0.5, 0.75].map((f) => (
          <line
            key={f}
            className="chart-grid-line"
            x1={PAD_L}
            x2={W - PAD_R}
            y1={PAD_T + (H - PAD_T - PAD_B) * f}
            y2={PAD_T + (H - PAD_T - PAD_B) * f}
          />
        ))}
        <path d={pathFrom(balances, max, { close: true })} fill={`url(#${gradientId})`} />
        <path
          className="chart-line chart-line-balance"
          d={pathFrom(balances, max)}
          vectorEffect="non-scaling-stroke"
        />
        <path
          className="chart-line chart-line-interest"
          d={pathFrom(cumulativeInterest, max)}
          vectorEffect="non-scaling-stroke"
        />
      </svg>
      {/* The horizontal axis is time; the shared vertical scale is called out
          separately so the two aren't confused for one another. */}
      <div className="chart-axis">
        <span>{t('chart.yearN', { n: 0 })}</span>
        {/* Labelled from the schedule that was actually plotted, not the
            nominal term. With extra payments the curve retires early — it
            used to reach zero under a label reading "Year 30", which is
            exactly the fact the user added extra payments to see. */}
        <span>{t('chart.axisEnd', { n: Number(Number(yearsPlotted ?? 0).toFixed(1)) })}</span>
      </div>
      <p className="chart-note">
        {t('chart.sharedScale', { min: formatMoney(0), max: formatMoney(max) })}
      </p>
    </figure>
  );
}

/**
 * How much of everything paid is the loan itself versus the cost of
 * borrowing it. A single proportional bar reads faster than a donut and
 * stays legible at any width.
 */
export function PrincipalInterestSplit({
  principal,
  totalInterest,
  interestSharePercent,
  formatMoney,
}) {
  const { t } = useI18n();
  // The share is the core's -- see `PaymentSummary::interest_share`. It is
  // absent rather than zero when nothing is paid at all: a bar drawn at 0%
  // would assert that none of the money is interest, rather than that there
  // is no money.
  if (interestSharePercent == null) return null;
  const interestShare = interestSharePercent;

  return (
    <figure className="chart">
      <figcaption className="chart-title">{t('chart.moneyGoes')}</figcaption>
      <div
        className="split-bar"
        role="img"
        aria-label={t('chart.splitAria', {
          principal: formatMoney(principal),
          interest: formatMoney(totalInterest),
          percent: interestShare.toFixed(0),
        })}
      >
        <span className="split-bar-principal" style={{ width: `${100 - interestShare}%` }} />
        <span className="split-bar-interest" style={{ width: `${interestShare}%` }} />
      </div>
      {/* These two keys label the bar directly above them, so they take the
          bar's own colours (`.split-bar-principal`/`.split-bar-interest`)
          rather than a second palette. They did not, which is how a plum
          bar ended up with a blue key. */}
      <div className="chart-legend">
        <span className="chart-key chart-key-balance">
          <i /> {t('chart.principalLegend', { amount: formatMoney(principal) })}
        </span>
        <span className="chart-key chart-key-interest">
          <i /> {t('chart.interestLegend', { amount: formatMoney(totalInterest) })}
        </span>
      </div>
      <p className="chart-note">
        {t('chart.interestShare', { percent: interestShare.toFixed(0) })}
      </p>
    </figure>
  );
}
