import React from 'react';
import { useI18n } from '../i18n';

const ENTRIES = [
  ['about.q1', 'about.a1'],
  ['about.q2', 'about.a2'],
  ['about.q3', 'about.a3'],
  ['about.q4', 'about.a4'],
];

/**
 * Explanatory content under the calculator.
 *
 * Partly for readers — these are the questions the figures above actually
 * raise — and partly because a page that is nothing but a widget has almost
 * no indexable text, which caps how well it can rank however good the tool
 * is. It renders in the active locale, so the Chinese build has real content
 * of its own rather than an English page wearing a translated interface.
 */
export default function About() {
  const { t } = useI18n();
  return (
    <section className="about">
      <h2>{t('about.title')}</h2>
      <dl>
        {ENTRIES.map(([q, a]) => (
          <div className="about-item" key={q}>
            <dt>{t(q)}</dt>
            <dd>{t(a)}</dd>
          </div>
        ))}
      </dl>
      <p className="about-disclaimer">{t('about.disclaimer')}</p>
    </section>
  );
}
