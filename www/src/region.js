// Browser-side signal gathering for region detection. No decisions here.
//
// Which market's rules the app starts in is `mortgage_core::Region::detect`'s
// call, not JavaScript's -- see crates/mortgage-core/src/region.rs. This file
// exists only because the signals it ranks (time zone, preferred languages,
// stored preference) are things a browser can observe and a wasm module
// cannot. It reads them and hands them over.

const STORAGE_KEY = 'mc:region';
const REGION_PARAM = 'region';

/** Only used when there is no wasm module to ask. */
export const DEFAULT_REGION = 'US';

/** The `?region=` value, if the URL carries one. Not validated here -- Rust
 *  decides whether it names a region, and ignores it if it does not. */
function fromUrl() {
  try {
    return new URLSearchParams(window.location.search).get(REGION_PARAM);
  } catch {
    return null;
  }
}

function fromStorage() {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    // Private mode can refuse storage entirely.
    return null;
  }
}

function timeZone() {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone ?? null;
  } catch {
    return null;
  }
}

function locales() {
  const tags = navigator.languages?.length ? navigator.languages : [navigator.language];
  return (tags ?? []).filter((tag) => typeof tag === 'string');
}

/**
 * The region to start in, decided by the Rust core.
 *
 * `wasmModule` is already initialized by the time App renders (see
 * index.js), so this can be called during the first render. Without the
 * module -- the mock used before `npm run build:wasm` has ever run -- there
 * is nobody to ask, so the app opens in its default region rather than
 * having JavaScript guess.
 */
export function detectRegion(wasmModule) {
  if (!wasmModule?.detect_region) return DEFAULT_REGION;
  // A link beats a saved preference: it is how someone shares, or is sent,
  // a specific region. Both are explicit choices, so which browser store
  // they came from is all that separates them -- the ranking against time
  // zone and language happens in Rust.
  const result = wasmModule.detect_region({
    chosen: fromUrl() ?? fromStorage(),
    time_zone: timeZone(),
    locales: locales(),
  });
  return result?.region ?? DEFAULT_REGION;
}

/** Persists an explicit choice from the header toggle, so a correction only
 *  has to be made once. */
export function rememberRegion(region) {
  try {
    localStorage.setItem(STORAGE_KEY, region);
  } catch {
    // Preference just won't survive the tab; the session still switches.
  }
}
