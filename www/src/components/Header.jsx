import React from 'react';

const TABS = [
  { id: 'payment', label: 'Payment' },
  { id: 'amortization', label: 'Amortization' },
  { id: 'affordability', label: 'Affordability' },
  { id: 'refinance', label: 'Refinance' },
];

export default function Header({ activeTab, onTabChange }) {
  return (
    <header className="app-header">
      <div className="app-title">
        <span className="app-title-mark">$</span>
        Mortgage Calculator
      </div>
      <nav className="app-tabs">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={tab.id === activeTab ? 'app-tab active' : 'app-tab'}
            onClick={() => onTabChange(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </nav>
    </header>
  );
}
