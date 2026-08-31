import React, { useEffect, useRef, useState } from 'react';
import Header from './components/Header';
import About from './components/About';
import PaymentCalculator from './components/PaymentCalculator';
import AmortizationSchedule from './components/AmortizationSchedule';
import AffordabilityCalculator from './components/AffordabilityCalculator';
import SingaporeAffordability from './components/SingaporeAffordability';
import RefinanceCalculator from './components/RefinanceCalculator';
import ComparisonView from './components/ComparisonView';
import ReportView from './components/ReportView';
import { I18nProvider, detectLocale, useI18n } from './i18n';
import { DEFAULT_SCENARIO, seedRateForRegion } from './scenario';
import { detectRegion, rememberRegion } from './region';

const PANELS = {
  payment: PaymentCalculator,
  amortization: AmortizationSchedule,
  affordability: AffordabilityCalculator,
  refinance: RefinanceCalculator,
  compare: ComparisonView,
  report: ReportView,
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

/// Exported so tests can drive the real tab wiring — the shared scenario
/// only means anything across a tab switch, which App owns.
export function AppShell({ wasmModule }) {
  const [activeTab, setActiveTab] = useState('payment');
  const [region, setRegion] = useState(() => detectRegion(wasmModule));
  // One loan, described from several angles — see src/scenario.js.
  const [scenario, setScenario] = useState(DEFAULT_SCENARIO);
  // Bumped whenever the "Your data" nav menu imports or clears scenarios,
  // so whichever tab's SavedScenarios list is currently mounted refetches
  // instead of showing stale entries until the next tab switch remounts it.
  const [dataVersion, setDataVersion] = useState(0);
  // The market the current loan was seeded for, so a region switch reseeds.
  const seededFor = useRef(null);
  const { t } = useI18n();
  const ActivePanel = panelFor(activeTab, region);

  // The quote a buyer opens on has to be one their market actually offers.
  // A Singapore buyer used to land on a flat 6.5% for thirty years: several
  // times the market, on a product no bank there sells. Reseeding discards
  // whatever was typed for the previous market, deliberately — the same
  // rationale the Compare tab's rows already follow, since a US fixed quote
  // relabelled S$ states a price nobody would honour.
  useEffect(() => {
    if (!wasmModule || seededFor.current === region) return;
    seededFor.current = region;
    setScenario((current) => seedRateForRegion(wasmModule, region, current));
  }, [wasmModule, region]);

  return (
    <div className="app">
      <Header
        activeTab={activeTab}
        onTabChange={setActiveTab}
        region={region}
        onRegionChange={(next) => {
          // Remembered, so a guess the user had to correct is only ever
          // corrected once. Detection reads this back ahead of any inferred
          // signal.
          rememberRegion(next);
          setRegion(next);
        }}
        wasmModule={wasmModule}
        onDataChanged={() => setDataVersion((v) => v + 1)}
      />
      <main className="app-main">
        <ActivePanel
          wasmModule={wasmModule}
          region={region}
          scenario={scenario}
          onScenarioChange={setScenario}
          dataVersion={dataVersion}
        />
        <About region={region} />
      </main>
      <footer className="app-footer">
        {t('app.footer')} {/* Relative so it resolves under a GitHub Pages project subpath. */}
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
