import React, { useState } from 'react';
import Header from './components/Header';
import PaymentCalculator from './components/PaymentCalculator';
import AmortizationSchedule from './components/AmortizationSchedule';
import AffordabilityCalculator from './components/AffordabilityCalculator';
import RefinanceCalculator from './components/RefinanceCalculator';
import ComparisonView from './components/ComparisonView';

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

export default function App({ wasmModule }) {
  const [activeTab, setActiveTab] = useState('payment');
  const [region, setRegion] = useState(detectRegion);
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
        Calculations run entirely client-side, compiled from Rust to WebAssembly.
        Your numbers never leave your device.
      </footer>
    </div>
  );
}
