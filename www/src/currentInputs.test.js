import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { useCurrentInputs } from './currentInputs';
import { createUnavailableModule } from './unavailable';

function mockWasmModule(overrides = {}) {
  return {
    load_current_inputs: vi.fn(async () => ({ inputs_json: null, error: null })),
    save_current_inputs: vi.fn(async () => ({ success: true, error: null })),
    ...overrides,
  };
}

afterEach(() => {
  vi.useRealTimers();
});

describe('useCurrentInputs', () => {
  it('restores via onLoad only when a persisted draft exists', async () => {
    const wasmModule = mockWasmModule({
      load_current_inputs: vi.fn(async () => ({ inputs_json: '{"homePrice":600000}', error: null })),
    });
    const onLoad = vi.fn();

    renderHook(() =>
      useCurrentInputs({
        wasmModule,
        storageKey: 'payment',
        getCurrentInputs: () => ({}),
        onLoad,
        dataVersion: 0,
        defaultInputs: {},
      }),
    );

    await waitFor(() => expect(onLoad).toHaveBeenCalledWith({ homePrice: 600000 }));
  });

  it('does not call onLoad when nothing was persisted', async () => {
    const wasmModule = mockWasmModule();
    const onLoad = vi.fn();

    renderHook(() =>
      useCurrentInputs({
        wasmModule,
        storageKey: 'payment',
        getCurrentInputs: () => ({}),
        onLoad,
        dataVersion: 0,
        defaultInputs: {},
      }),
    );

    await waitFor(() => expect(wasmModule.load_current_inputs).toHaveBeenCalledWith('payment'));
    expect(onLoad).not.toHaveBeenCalled();
  });

  it('swallows a rejected restore instead of throwing', async () => {
    const wasmModule = mockWasmModule({
      load_current_inputs: vi.fn(async () => {
        throw new Error('IndexedDB unavailable');
      }),
    });
    const onLoad = vi.fn();

    renderHook(() =>
      useCurrentInputs({
        wasmModule,
        storageKey: 'payment',
        getCurrentInputs: () => ({}),
        onLoad,
        dataVersion: 0,
        defaultInputs: {},
      }),
    );

    await waitFor(() => expect(wasmModule.load_current_inputs).toHaveBeenCalled());
    expect(onLoad).not.toHaveBeenCalled();
  });

  it('debounces edits into one coalesced save, not one per render', async () => {
    const wasmModule = mockWasmModule();
    let homePrice = 500000;

    const { rerender } = renderHook(() =>
      useCurrentInputs({
        wasmModule,
        storageKey: 'payment',
        getCurrentInputs: () => ({ homePrice }),
        onLoad: () => {},
        dataVersion: 0,
        defaultInputs: {},
      }),
    );

    // The mount restore must resolve (flipping `hydrated`) before the
    // auto-save effect is allowed to schedule anything -- real timers here,
    // switched to fake only once that's settled.
    await waitFor(() => expect(wasmModule.load_current_inputs).toHaveBeenCalled());

    vi.useFakeTimers();
    homePrice = 600000;
    rerender();
    homePrice = 700000;
    rerender();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(500);
    });

    expect(wasmModule.save_current_inputs).toHaveBeenCalledTimes(1);
    expect(wasmModule.save_current_inputs).toHaveBeenCalledWith(
      'payment',
      JSON.stringify({ homePrice: 700000 }),
    );
  });

  it('resets to defaultInputs when dataVersion changes after hydration, exactly once', async () => {
    const wasmModule = mockWasmModule();
    const onLoad = vi.fn();
    const defaultInputs = { homePrice: 500000 };

    const { rerender } = renderHook(
      ({ dataVersion }) =>
        useCurrentInputs({
          wasmModule,
          storageKey: 'payment',
          getCurrentInputs: () => ({}),
          onLoad,
          dataVersion,
          defaultInputs,
        }),
      { initialProps: { dataVersion: 0 } },
    );

    await waitFor(() => expect(wasmModule.load_current_inputs).toHaveBeenCalled());

    rerender({ dataVersion: 1 });
    expect(onLoad).toHaveBeenCalledTimes(1);
    expect(onLoad).toHaveBeenCalledWith(defaultInputs);

    rerender({ dataVersion: 1 });
    expect(onLoad).toHaveBeenCalledTimes(1);
  });

  it('does not reset on mount when dataVersion starts non-zero', async () => {
    const wasmModule = mockWasmModule();
    const onLoad = vi.fn();

    renderHook(() =>
      useCurrentInputs({
        wasmModule,
        storageKey: 'payment',
        getCurrentInputs: () => ({}),
        onLoad,
        dataVersion: 5,
        defaultInputs: {},
      }),
    );

    await waitFor(() => expect(wasmModule.load_current_inputs).toHaveBeenCalled());
    expect(onLoad).not.toHaveBeenCalled();
  });

  it('produces no unhandled errors against the degraded (wasm-unavailable) module', async () => {
    const wasmModule = createUnavailableModule();
    const onLoad = vi.fn();

    const { rerender } = renderHook(() =>
      useCurrentInputs({
        wasmModule,
        storageKey: 'payment',
        getCurrentInputs: () => ({ homePrice: 500000 }),
        onLoad,
        dataVersion: 0,
        defaultInputs: {},
      }),
    );

    await waitFor(() => expect(onLoad).not.toHaveBeenCalled());
    // A rerender exercises the debounced auto-save effect too (scheduled,
    // never asserted on) -- the point is that neither effect throws against
    // this module's stubs.
    rerender();
  });
});
