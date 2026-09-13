import React from 'react';

const ICON_PROPS = {
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 1.8,
  strokeLinecap: 'round',
  strokeLinejoin: 'round',
  'aria-hidden': true,
};

/**
 * Header trigger for `YourDataMenu` -- three preference sliders, not a
 * gear: a radiating-spokes gear at header-icon size reads as a sun/
 * brightness toggle instead, and this app now has an actual light/dark
 * toggle to confuse it with. Ported from budget_planner's icons.jsx,
 * same rationale.
 */
export function SettingsIcon() {
  return (
    <svg {...ICON_PROPS} className="nav-icon">
      <path d="M4 7h20M4 12h20M4 17h20" />
      <circle cx="15" cy="7" r="2.4" fill="currentColor" stroke="none" />
      <circle cx="8" cy="12" r="2.4" fill="currentColor" stroke="none" />
      <circle cx="17" cy="17" r="2.4" fill="currentColor" stroke="none" />
    </svg>
  );
}
