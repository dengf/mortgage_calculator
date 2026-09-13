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

/**
 * The six tab-nav icons -- one per calculator, in Header.jsx's TABS order.
 * Same thin-stroke convention as SettingsIcon above, sized via `.nav-icon`
 * (bigger than a label-adjacent icon would need) since each one sits alone
 * in its own tap target: inline beside a label on the desktop pill row,
 * stacked above one on the phone bottom bar. Ported from budget_planner's
 * icons.jsx, same NAV_ICON_PROPS convention.
 */
const NAV_ICON_PROPS = { ...ICON_PROPS, className: 'nav-icon' };

export function PaymentIcon() {
  return (
    <svg {...NAV_ICON_PROPS}>
      <circle cx="12" cy="12" r="8" />
      <path d="M12 6.5v11" />
      <path d="M14.7 9c0-1.1-1.2-2-2.7-2s-2.7.8-2.7 1.8c0 2.4 5.4 1 5.4 3.4 0 1.1-1.2 1.8-2.7 1.8s-2.7-.9-2.7-2" />
    </svg>
  );
}

export function AmortizationIcon() {
  return (
    <svg {...NAV_ICON_PROPS}>
      <rect x="4" y="5" width="16" height="14" rx="1.5" />
      <path d="M4 10h16M9 10v9" />
    </svg>
  );
}

export function AffordabilityIcon() {
  return (
    <svg {...NAV_ICON_PROPS}>
      <path d="M4 11 12 4l8 7" />
      <path d="M6 10v9a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-9" />
      <path d="M10 20v-5h4v5" />
    </svg>
  );
}

export function RefinanceIcon() {
  return (
    <svg {...NAV_ICON_PROPS}>
      <path d="M4 12a8 8 0 0 1 13.6-5.7" />
      <path d="M20 12a8 8 0 0 1-13.6 5.7" />
      <path d="M17 3v4h-4" />
      <path d="M7 21v-4h4" />
    </svg>
  );
}

export function CompareIcon() {
  return (
    <svg {...NAV_ICON_PROPS}>
      <path d="M6 20V10M12 20V4M18 20v7" />
    </svg>
  );
}

export function ReportIcon() {
  return (
    <svg {...NAV_ICON_PROPS}>
      <path d="M7 3h7l4 4v13a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z" />
      <path d="M14 3v4h4" />
      <path d="M8.5 12h7M8.5 15.5h7M8.5 8.5h3" />
    </svg>
  );
}
