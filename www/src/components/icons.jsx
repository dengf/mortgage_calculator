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
 * Header trigger for `YourDataMenu` -- the spoked circle, identical to
 * budget_planner's icon of the same name, opening the same menu from the
 * same place in the same header.
 *
 * This used to be three preference sliders, on the argument that a
 * radiating-spokes gear at header-icon size reads as a sun/brightness
 * toggle and both apps have a real light/dark control to confuse it
 * with. That argument is still true and was still overruled: the user
 * asked for the spoked glyph in budget_planner explicitly (see that
 * file's own note), and the two tools then sat side by side with
 * different icons on the same button. One icon per control across the
 * line beats the better of two icons in one tool. Do not swap this back
 * on its own -- change both or neither.
 *
 * Sized via `nav-icon` rather than the inherited `.field-icon`: like the
 * bottom nav's icons this one stands alone in its own 44px tap target
 * instead of sitting beside label text.
 */
export function SettingsIcon() {
  return (
    <svg {...ICON_PROPS} className="nav-icon">
      <circle cx="12" cy="12" r="3" />
      <path d="M12 2.5v2.2M12 19.3v2.2M21.5 12h-2.2M4.7 12H2.5M18.7 5.3l-1.6 1.6M6.9 17.1l-1.6 1.6M18.7 18.7l-1.6-1.6M6.9 6.9 5.3 5.3" />
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
