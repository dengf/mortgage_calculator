import React from 'react';
import { useI18n } from '../i18n';

/**
 * What this is, and the one thing that makes it different, above the fold.
 *
 * The privacy claim used to live in the footer, in the smallest and lowest
 * contrast type on the page — the strongest argument the product has, placed
 * where nobody reads. It also invites verification: nobody believes a
 * privacy promise, but everybody believes an empty network tab.
 */
export default function Intro() {
  const { t } = useI18n();
  return (
    <section className="intro">
      <p className="intro-lead">{t('intro.lead')}</p>
      <p className="intro-privacy">
        <strong>{t('intro.privacy')}</strong>{' '}
        <span className="intro-verify">{t('intro.verify')}</span>
      </p>
      <p className="intro-depth">{t('intro.depth')}</p>
    </section>
  );
}
