import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const HREF = 'https://example.test/mortgage_calculator/';

/**
 * `startVersionCheck` reads `__BUILD_ID__`, which webpack's DefinePlugin
 * normally substitutes at compile time. Set it per-test and re-import so
 * each case gets a module instance bound to the right id.
 */
async function loadModule(buildId) {
  vi.resetModules();
  globalThis.__BUILD_ID__ = buildId;
  return import('./version-check');
}

function stubLocation() {
  const replace = vi.fn();
  delete window.location;
  window.location = { href: HREF, replace };
  return replace;
}

function respondWith(buildId) {
  globalThis.fetch = vi.fn(async () => ({
    ok: true,
    json: async () => ({ buildId }),
  }));
}

beforeEach(() => {
  sessionStorage.clear();
  vi.spyOn(window.history, 'replaceState').mockImplementation(() => {});
});

afterEach(() => {
  vi.restoreAllMocks();
  delete globalThis.__BUILD_ID__;
});

describe('startVersionCheck', () => {
  it('does nothing when the page is already the deployed build', async () => {
    const replace = stubLocation();
    respondWith('abc123');
    const { startVersionCheck } = await loadModule('abc123');

    startVersionCheck();
    await vi.waitFor(() => expect(globalThis.fetch).toHaveBeenCalled());

    expect(replace).not.toHaveBeenCalled();
  });

  it('reloads onto the deployed build when this page is stale', async () => {
    const replace = stubLocation();
    respondWith('new999');
    const { startVersionCheck } = await loadModule('old111');

    startVersionCheck();

    // Navigates to a URL the HTTP cache has never seen — a plain reload is
    // allowed to re-serve the same stale HTML.
    await vi.waitFor(() => expect(replace).toHaveBeenCalledTimes(1));
    expect(replace.mock.calls[0][0]).toContain('v=new999');
  });

  it('bypasses the cache when asking what is deployed', async () => {
    stubLocation();
    respondWith('new999');
    const { startVersionCheck } = await loadModule('old111');

    startVersionCheck();
    await vi.waitFor(() => expect(globalThis.fetch).toHaveBeenCalled());

    expect(globalThis.fetch).toHaveBeenCalledWith('version.json', { cache: 'no-store' });
  });

  it('reloads only once for a given build, so it cannot loop', async () => {
    const replace = stubLocation();
    respondWith('new999');
    // Simulates the post-reload load: the guard is already set, yet the
    // bundle is somehow still the old one.
    sessionStorage.setItem('mc:reloaded-for-build', 'new999');
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const { startVersionCheck } = await loadModule('old111');

    startVersionCheck();
    await vi.waitFor(() => expect(warn).toHaveBeenCalled());

    expect(replace).not.toHaveBeenCalled();
  });

  it('stays put when version.json cannot be fetched', async () => {
    const replace = stubLocation();
    globalThis.fetch = vi.fn(async () => {
      throw new Error('offline');
    });
    const { startVersionCheck } = await loadModule('old111');

    startVersionCheck();
    await vi.waitFor(() => expect(globalThis.fetch).toHaveBeenCalled());

    expect(replace).not.toHaveBeenCalled();
  });

  it('stays put when version.json is missing', async () => {
    const replace = stubLocation();
    globalThis.fetch = vi.fn(async () => ({ ok: false }));
    const { startVersionCheck } = await loadModule('old111');

    startVersionCheck();
    await vi.waitFor(() => expect(globalThis.fetch).toHaveBeenCalled());

    expect(replace).not.toHaveBeenCalled();
  });

  it('notifies onStale instead of reloading once the tab is in use and visible', async () => {
    const replace = stubLocation();
    respondWith('old111'); // matches current build -- nothing stale yet
    const { startVersionCheck } = await loadModule('old111');
    const onStale = vi.fn();

    startVersionCheck({ onStale });
    await vi.waitFor(() => expect(globalThis.fetch).toHaveBeenCalled());

    // A deploy lands while the tab stays open and focused (document.hidden
    // is false by default in jsdom) -- reloading would discard whatever
    // the visitor has typed, so this must notify instead of navigating.
    respondWith('new999');
    document.dispatchEvent(new Event('visibilitychange'));

    await vi.waitFor(() => expect(onStale).toHaveBeenCalledWith('new999'));
    expect(replace).not.toHaveBeenCalled();
  });

  it('is inert outside a webpack build, where no id is compiled in', async () => {
    const replace = stubLocation();
    globalThis.fetch = vi.fn();
    const { startVersionCheck } = await loadModule(undefined);
    delete globalThis.__BUILD_ID__;

    const stop = startVersionCheck();

    expect(globalThis.fetch).not.toHaveBeenCalled();
    expect(replace).not.toHaveBeenCalled();
    expect(typeof stop).toBe('function');
  });
});
