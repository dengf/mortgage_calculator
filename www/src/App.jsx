import React, { useState } from 'react';
import Header from './components/Header';
import PaymentCalculator from './components/PaymentCalculator';
import AmortizationSchedule from './components/AmortizationSchedule';
import AffordabilityCalculator from './components/AffordabilityCalculator';
import RefinanceCalculator from './components/RefinanceCalculator';
import ComparisonView from './components/ComparisonView';
import { I18nProvider, detectLocale, useI18n } from './i18n';

const PANELS = {
  payment: PaymentCalculator,
  amortization: AmortizationSchedule,
  affordability: AffordabilityCalculator,
  refinance: RefinanceCalculator,
  compare: ComparisonView,
};

/**
 * Picks the starting region from the browser's locale, the same way the
 * Slint app reads the device locale at startup. The header toggle
 * overrides it.
 */
function detectRegion() {
  const locales = navigator.languages?.length ? navigator.languages : [navigator.language];
  return locales.some((l) => typeof l === 'string' && l.toUpperCase().endsWith('-SG')) ? 'SG' : 'US';
}

function AppShell({ wasmModule }) {
  const [activeTab, setActiveTab] = useState('payment');
  const [region, setRegion] = useState(detectRegion);
  const { t } = useI18n();
  const ActivePanel = PANELS[activeTab];

  return (
    <div className="app">
      <Header
        activeTab={activeTab}
        onTabChange={setActiveTab}
        region={region}
        onRegionChange={setRegion}
      />
      <main className="app-main">
        <ActivePanel wasmModule={wasmModule} region={region} />
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
