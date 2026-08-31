import React from 'react';
import { LOCALES, useI18n } from '../i18n';
import MeifioMark from './MeifioMark';

/* The family's home. Moves to https://meifio.app once the domain is live;
   it is a constant so that is a one-line change. */
const MEIFIO_HOME = 'https://dengf.github.io/meifio-blog/';

const TABS = [
  { id: 'payment', key: 'nav.payment' },
  { id: 'amortization', key: 'nav.amortization' },
  { id: 'affordability', key: 'nav.affordability' },
  { id: 'refinance', key: 'nav.refinance' },
  { id: 'compare', key: 'nav.compare' },
  { id: 'report', key: 'nav.report' },
];

const REGIONS = [
  { id: 'US', label: 'US' },
  { id: 'SG', label: 'SG' },
];

export default function Header({ activeTab, onTabChange, region, onRegionChange }) {
  const { t, locale, setLocale } = useI18n();

  return (
    // A fragment, not one <header>: the tab nav needs to be a direct child
    // of .app, not nested inside this short header block, so that
    // `position: sticky` on .app-tabs has the whole page as its containing
    // block on a phone. Nested inside .app-header it could only stay pinned
    // for as long as that short block was still on screen, then it would
    // scroll away along with it.
    <>
      <header className="app-header">
        <div className="app-brand">
          {/* The page had no h1 at all. This is the one, and it names the
              product rather than repeating a tab label. */}
          <h1 className="app-title">{t('app.title')}</h1>

          {/* The house mark it used to carry is gone: the family mark now lives in
              the byline's logotype, and showing both read as two logos arguing.

              The byline is split rather than interpolated because the brand is an
              element, not a string -- and the word order around it differs by
              locale ("a meifio app" vs "meifio 出品"), so the mark cannot simply be
              pinned to one end. */}
          <a className="app-byline" href={MEIFIO_HOME}>
            {t('app.byline').split('{logo}').flatMap((part, i) =>
              i === 0 ? [part] : [<MeifioMark key="mark" />, part],
            )}
          </a>
        </div>

        <div className="app-switches">
          {/* Language and region are separate axes on purpose: someone reading
              in Chinese may well be buying in the US, and vice versa. */}
          <div className="app-regions" role="group" aria-label={t('app.language')}>
            {LOCALES.map((l) => (
              <button
                key={l.id}
                type="button"
                className={l.id === locale ? 'app-region active' : 'app-region'}
                aria-pressed={l.id === locale}
                title={l.name}
                lang={l.id}
                onClick={() => setLocale(l.id)}
              >
                {l.label}
              </button>
            ))}
          </div>

          {onRegionChange && (
            <div className="app-regions" role="group" aria-label={t('app.region')}>
              {REGIONS.map((r) => (
                <button
                  key={r.id}
                  type="button"
                  className={r.id === region ? 'app-region active' : 'app-region'}
                  aria-pressed={r.id === region}
                  onClick={() => onRegionChange(r.id)}
                >
                  {r.label}
                </button>
              ))}
            </div>
          )}
        </div>
      </header>

      <nav className="app-tabs">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={tab.id === activeTab ? 'app-tab active' : 'app-tab'}
            onClick={() => onTabChange(tab.id)}
          >
            {t(tab.key)}
          </button>
        ))}
      </nav>
    </>
  );
}
