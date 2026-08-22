import React from 'react';
import { useI18n } from '../i18n';

/**
 * What this is, and the one thing that makes it different, above the fold.
 *
 * The privacy claim used to live in the footer, in the smallest and lowest
 * contrast type on the page — the strongest argument the product has, placed
 * where nobody reads. It also invites verification: nobody believes a
 * privacy promise, but everybody believes an empty network tab.
 *
 * The notice is linked from the claim itself rather than added to the nav.
 * Someone who reads "nothing is sent anywhere" and wants the detail wants it
 * at that moment, not from a menu; the footer link stays for people who look
 * for legal text where legal text usually lives.
 */
export default function Intro({ region = 'US' }) {
  const { t } = useI18n();
  return (
    <section className="intro">
      <p className="intro-lead">{t('intro.lead')}</p>
      <p className="intro-privacy">
        <strong>{t('intro.privacy')}</strong>{' '}
        <span className="intro-verify">{t('intro.verify')}</span>{' '}
        {/* Relative so it resolves under a GitHub Pages project subpath. */}
        <a className="intro-privacy-link" href="privacy.html">
          {t('app.privacy')}
        </a>
      </p>
      {/* Only the rules that apply where the user is buying. */}
      <p className="intro-depth">{t(`intro.depth.${region}`)}</p>
    </section>
  );
}
