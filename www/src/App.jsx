import React, { useState } from 'react';
import Header from './components/Header';
import PaymentCalculator from './components/PaymentCalculator';
import AmortizationSchedule from './components/AmortizationSchedule';
import AffordabilityCalculator from './components/AffordabilityCalculator';
import RefinanceCalculator from './components/RefinanceCalculator';

const PANELS = {
  payment: PaymentCalculator,
  amortization: AmortizationSchedule,
  affordability: AffordabilityCalculator,
  refinance: RefinanceCalculator,
};

export default function App({ wasmModule }) {
  const [activeTab, setActiveTab] = useState('payment');
  const ActivePanel = PANELS[activeTab];

  return (
    <div className="app">
      <Header activeTab={activeTab} onTabChange={setActiveTab} />
      <main className="app-main">
        <ActivePanel wasmModule={wasmModule} />
      </main>
      <footer className="app-footer">
        Calculations run entirely client-side, compiled from Rust to WebAssembly.
      </footer>
    </div>
  );
}
