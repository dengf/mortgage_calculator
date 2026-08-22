import React, { useState } from 'react';
import Header from './components/Header';
import PaymentCalculator from './components/PaymentCalculator';
import AmortizationSchedule from './components/AmortizationSchedule';
import AffordabilityCalculator from './components/AffordabilityCalculator';
import SingaporeAffordability from './components/SingaporeAffordability';
import RefinanceCalculator from './components/RefinanceCalculator';
import ComparisonView from './components/ComparisonView';
import { I18nProvider, detectLocale, useI18n } from './i18n';
import { DEFAULT_SCENARIO } from './scenario';

const PANELS = {
  payment: PaymentCalculator,
  amortization: AmortizationSchedule,
  affordability: AffordabilityCalculator,
  refinance: RefinanceCalculator,
  compare: ComparisonView,
};

/**
 * Affordability is the one tab with no shared model across regions: the US
 * works backwards from a debt-to-income ratio, Singapore from MAS servicing
 * ceilings and LTV limits. They're separate components rather than one with
 * branches, so neither carries the other's fields.
 */
function panelFor(tab, region) {
  if (tab === 'affordability' && region === 'SG') return SingaporeAffordability;
  return PANELS[tab];
}

/**
 * Picks the starting region from the browser's locale, the same way the
 * Slint app reads the device locale at startup. The header toggle
 * overrides it.
 */
function detectRegion() {
  const locales = navigator.languages?.length ? navigator.languages : [navigator.language];
  return locales.some((l) => typeof l === 'string' && l.toUpperCase().endsWith('-SG')) ? 'SG' : 'US';
}

/// Exported so tests can drive the real tab wiring — the shared scenario
/// only means anything across a tab switch, which App owns.
export function AppShell({ wasmModule }) {
  const [activeTab, setActiveTab] = useState('payment');
  const [region, setRegion] = useState(detectRegion);
  // One loan, described from several angles — see src/scenario.js.
  const [scenario, setScenario] = useState(DEFAULT_SCENARIO);
  const { t } = useI18n();
  const ActivePanel = panelFor(activeTab, region);

  return (
    <div className="app">
      <Header
        activeTab={activeTab}
        onTabChange={setActiveTab}
        region={region}
        onRegionChange={setRegion}
      />
      <main className="app-main">
        <ActivePanel
          wasmModule={wasmModule}
          region={region}
          scenario={scenario}
          onScenarioChange={setScenario}
        />
      </main>
      <footer className="app-footer">
        {t('app.footer')}{' '}
        {/* Relative so it resolves under a GitHub Pages project subpath. */}
        <a href="privacy.html">{t('app.privacy')}</a>
        {' · '}
        <a href="https://github.com/dengf/mortgage_calculator">{t('app.source')}</a>
      </footer>
    </div>
  );
}

export default function App({ wasmModule }) {
  return (
    <I18nProvider initialLocale={detectLocale()}>
      <AppShell wasmModule={wasmModule} />
    </I18nProvider>
  );
}
