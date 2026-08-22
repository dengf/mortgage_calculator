import React from 'react';
import { LOCALES, useI18n } from '../i18n';

const TABS = [
  { id: 'payment', key: 'nav.payment' },
  { id: 'amortization', key: 'nav.amortization' },
  { id: 'affordability', key: 'nav.affordability' },
  { id: 'refinance', key: 'nav.refinance' },
  { id: 'compare', key: 'nav.compare' },
];

const REGIONS = [
  { id: 'US', label: 'US' },
  { id: 'SG', label: 'SG' },
];

export default function Header({ activeTab, onTabChange, region, onRegionChange }) {
  const { t, locale, setLocale } = useI18n();

  return (
    <header className="app-header">
      <div className="app-title">
        {/* Relative, not root-absolute, so it still resolves when the app is
            served from a subpath (e.g. a GitHub Pages project site).
            Decorative: the adjacent text already names the app. */}
        <img className="app-title-mark" src="icon-192.png" alt="" width="32" height="32" />
        {t('app.title')}
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
    </header>
  );
}
