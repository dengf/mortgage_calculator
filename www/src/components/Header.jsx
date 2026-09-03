import React, { useEffect, useRef } from 'react';
import { LOCALES, useI18n } from '../i18n';
import MeifioMark from './MeifioMark';
import YourDataMenu from './YourDataMenu';

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

export default function Header({
  activeTab,
  onTabChange,
  region,
  onRegionChange,
  wasmModule,
  onDataChanged,
}) {
  const { t, locale, setLocale } = useI18n();
  const tabsRef = useRef(null);

  // Amortization and Report pin their "Loan details" bar just below this nav
  // on a phone (see .scenario-fields-collapsible), so a reader scrolling a
  // long schedule never loses sight of which loan it describes. That bar
  // needs this nav's actual rendered height to stack under rather than
  // overlap it -- and the nav wraps onto a different number of rows
  // depending on the viewport width and the locale's tab label lengths, so
  // no fixed number would stay right. A ResizeObserver keeps the custom
  // property true across both. jsdom has no ResizeObserver, so this is a
  // no-op under test.
  useEffect(() => {
    const el = tabsRef.current;
    if (!el || typeof ResizeObserver === 'undefined') return undefined;
    const rootStyle = document.documentElement.style;
    const setCustomProperty = rootStyle.setProperty.bind(rootStyle);
    const observer = new ResizeObserver(() => {
      setCustomProperty('--app-tabs-height', el.offsetHeight + 'px');
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

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
            {t('app.byline')
              .split('{logo}')
              .flatMap((part, i) => (i === 0 ? [part] : [<MeifioMark key="mark" />, part]))}
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

      <nav className="app-tabs" ref={tabsRef}>
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={tab.id === activeTab ? 'app-tab active' : 'app-tab'}
            onClick={() => onTabChange(tab.id)}
          >
            {t(tab.key)}
          </button>
        ))}
        <YourDataMenu wasmModule={wasmModule} onDataChanged={onDataChanged} />
      </nav>
    </>
  );
}
