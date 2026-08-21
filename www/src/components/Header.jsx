import React from 'react';

const TABS = [
  { id: 'payment', label: 'Payment' },
  { id: 'amortization', label: 'Amortization' },
  { id: 'affordability', label: 'Affordability' },
  { id: 'refinance', label: 'Refinance' },
  { id: 'compare', label: 'Compare' },
];

export default function Header({ activeTab, onTabChange }) {
  return (
    <header className="app-header">
      <div className="app-title">
        {/* Relative, not root-absolute, so it still resolves when the app is
            served from a subpath (e.g. a GitHub Pages project site).
            Decorative: the adjacent text already names the app. */}
        <img className="app-title-mark" src="icon-192.png" alt="" width="32" height="32" />
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
