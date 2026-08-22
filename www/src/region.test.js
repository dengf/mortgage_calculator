import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { DEFAULT_REGION, detectRegion, rememberRegion } from './region';

// These cover the signal *gathering* only. Which signal wins is
// mortgage-core's `Region::detect`, tested in
// crates/mortgage-core/src/region.rs -- deliberately not re-asserted here,
// because a JS test agreeing with a JS copy of the ranking is exactly the
// duplication this module exists to avoid.

const wasmSpy = () => ({ detect_region: vi.fn(() => ({ region: 'SG' })) });
const signalsOf = (wasm) => wasm.detect_region.mock.calls[0][0];

// jsdom in this project ships no localStorage, on `window` or as a bare
// global, so every storage path in the app currently runs straight into its
// catch block under test. Standing one up is what makes the stored-choice
// signal testable at all.
function fakeStorage() {
  const map = new Map();
  return {
    getItem: (k) => (map.has(k) ? map.get(k) : null),
    setItem: (k, v) => map.set(k, String(v)),
    removeItem: (k) => map.delete(k),
    clear: () => map.clear(),
  };
}

beforeEach(() => {
  vi.stubGlobal('localStorage', fakeStorage());
});

afterEach(() => {
  vi.unstubAllGlobals();
  window.history.replaceState(null, '', '/');
});

describe('detectRegion', () => {
  it('hands the browser signals to Rust and returns its answer', () => {
    vi.stubGlobal('navigator', { languages: ['en-GB', 'zh-CN'] });
    // jsdom's Intl reports no time zone, so it is stubbed rather than read:
    // the assertion is that whatever the platform says gets forwarded, and
    // en-GB alongside Asia/Singapore is the exact production case that was
    // being resolved to the US ruleset.
    vi.stubGlobal('Intl', {
      DateTimeFormat: () => ({ resolvedOptions: () => ({ timeZone: 'Asia/Singapore' }) }),
    });
    const wasm = wasmSpy();

    expect(detectRegion(wasm)).toBe('SG');

    expect(signalsOf(wasm).locales).toEqual(['en-GB', 'zh-CN']);
    expect(signalsOf(wasm).time_zone).toBe('Asia/Singapore');
  });

  it('reports no time zone when the platform cannot name one', () => {
    vi.stubGlobal('Intl', {
      DateTimeFormat: () => ({ resolvedOptions: () => ({}) }),
    });
    const wasm = wasmSpy();

    detectRegion(wasm);

    expect(signalsOf(wasm).time_zone).toBeNull();
  });

  it('prefers a ?region= link over a stored preference', () => {
    window.history.replaceState(null, '', '?region=SG');
    rememberRegion('US');
    const wasm = wasmSpy();

    detectRegion(wasm);

    expect(signalsOf(wasm).chosen).toBe('SG');
  });

  it('passes a stored preference on when the URL names none', () => {
    rememberRegion('SG');
    const wasm = wasmSpy();

    detectRegion(wasm);

    expect(signalsOf(wasm).chosen).toBe('SG');
  });

  it('forwards an unrecognized choice rather than screening it', () => {
    // Validation is Rust's, so a stale or hand-edited value must reach it
    // instead of being quietly dropped here.
    window.history.replaceState(null, '', '?region=Atlantis');
    const wasm = wasmSpy();

    detectRegion(wasm);

    expect(signalsOf(wasm).chosen).toBe('Atlantis');
  });

  it('reports no choice when neither the URL nor storage holds one', () => {
    const wasm = wasmSpy();

    detectRegion(wasm);

    expect(signalsOf(wasm).chosen).toBeNull();
  });

  it('falls back to the default when there is no module to ask', () => {
    expect(detectRegion(null)).toBe(DEFAULT_REGION);
    expect(detectRegion({})).toBe(DEFAULT_REGION);
  });

  it('survives a browser that refuses storage entirely', () => {
    vi.stubGlobal('localStorage', {
      getItem: () => {
        throw new DOMException('denied', 'SecurityError');
      },
      setItem: () => {
        throw new DOMException('denied', 'SecurityError');
      },
    });
    const wasm = wasmSpy();

    expect(() => rememberRegion('SG')).not.toThrow();
    expect(detectRegion(wasm)).toBe('SG');
    expect(signalsOf(wasm).chosen).toBeNull();
  });
});
