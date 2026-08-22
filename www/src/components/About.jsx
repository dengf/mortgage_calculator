import React from 'react';
import { useI18n } from '../i18n';

/**
 * Explanatory content under the calculator, scoped to the active region.
 *
 * Showing both rule sets at once was actively confusing: a US buyer was
 * asked to read about TDSR and CPF, and a Singapore buyer about PMI and ZIP
 * codes. Neither applies to the other, and mixing them makes a reader doubt
 * whether the figures above apply to them either.
 *
 * Refinance break-even is the one question that is genuinely region-neutral,
 * so it appears in both lists rather than being duplicated per region.
 */
const ENTRIES = {
  US: [
    ['about.us.payment.q', 'about.us.payment.a'],
    ['about.us.pmi.q', 'about.us.pmi.a'],
    ['about.us.jumbo.q', 'about.us.jumbo.a'],
    ['about.refi.q', 'about.refi.a'],
  ],
  SG: [
    ['about.sg.payment.q', 'about.sg.payment.a'],
    ['about.sg.tdsr.q', 'about.sg.tdsr.a'],
    ['about.sg.afford.q', 'about.sg.afford.a'],
    ['about.refi.q', 'about.refi.a'],
  ],
};

export default function About({ region = 'US' }) {
  const { t } = useI18n();
  const entries = ENTRIES[region] ?? ENTRIES.US;

  return (
    <section className="about">
      <h2>{t('about.title')}</h2>
      <dl>
        {entries.map(([q, a]) => (
          <div className="about-item" key={q}>
            <dt>{t(q)}</dt>
            <dd>{t(a)}</dd>
          </div>
        ))}
      </dl>
      <p className="about-disclaimer">{t(`about.disclaimer.${region}`)}</p>
    </section>
  );
}
