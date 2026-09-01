import { useEffect, useRef } from 'react';

/**
 * Auto-persists a calculator's in-progress inputs, separate from the
 * explicit "Save current as..." feature: restores them once on mount, then
 * keeps them saved as they change, so a reload lands where the user left
 * off instead of resetting to hardcoded defaults.
 *
 * `storageKey` must be unique to this component — not the calculator kind
 * used for named scenarios, since two calculators (US and Singapore
 * affordability) share one kind despite incompatible field shapes.
 *
 * Best-effort by design: a failed restore or save is swallowed rather than
 * surfaced, since this runs silently in the background rather than in
 * response to something the user pressed. Losing an in-progress draft is
 * far less bad than an alarming error banner on every page load for a
 * browser that merely struggles with IndexedDB.
 */
export function useCurrentInputs({
  wasmModule,
  storageKey,
  getCurrentInputs,
  onLoad,
  dataVersion,
  defaultInputs,
}) {
  const hydrated = useRef(false);
  const lastSavedRef = useRef(null);
  const dataVersionSeen = useRef(dataVersion);

  useEffect(() => {
    if (!wasmModule) return undefined;
    let cancelled = false;

    (async () => {
      try {
        const result = await wasmModule.load_current_inputs(storageKey);
        if (!cancelled && result?.inputs_json) {
          onLoad(JSON.parse(result.inputs_json));
        }
      } catch {
        // No persisted draft, or storage is unavailable -- fall back to
        // whatever is already on screen.
      } finally {
        if (!cancelled) hydrated.current = true;
      }
    })();

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wasmModule]);

  useEffect(() => {
    if (!wasmModule || !hydrated.current) return undefined;
    const json = JSON.stringify(getCurrentInputs());
    if (json === lastSavedRef.current) return undefined;

    const id = setTimeout(() => {
      lastSavedRef.current = json;
      wasmModule.save_current_inputs?.(storageKey, json)?.catch?.(() => {});
    }, 500);
    return () => clearTimeout(id);
  });

  useEffect(() => {
    if (dataVersion === dataVersionSeen.current) return;
    dataVersionSeen.current = dataVersion;
    onLoad(defaultInputs);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dataVersion]);
}

/**
 * Like [`useCurrentInputs`], but for state whose default depends on
 * `region` (a rate/term seeded from that market's own presets). Restoring a
 * value saved under a *different* region than the one now in effect would
 * silently reintroduce a foreign rate or term — the same correctness
 * problem `seedRateForRegion` exists to prevent when the region changes
 * live. The persisted blob is tagged with the region it was saved under; on
 * a mismatch, `reseedForRegion(restored)` re-derives it for the current
 * region instead of trusting it as-is.
 */
export function useRegionAwareCurrentInputs({
  wasmModule,
  storageKey,
  region,
  getCurrentInputs,
  onLoad,
  reseedForRegion,
  onHydrated,
  dataVersion,
  defaultInputs,
}) {
  const hydrated = useRef(false);
  const lastSavedRef = useRef(null);
  const dataVersionSeen = useRef(dataVersion);

  useEffect(() => {
    if (!wasmModule) return undefined;
    let cancelled = false;

    (async () => {
      try {
        const result = await wasmModule.load_current_inputs(storageKey);
        if (!cancelled && result?.inputs_json) {
          const { region: savedRegion, ...rest } = JSON.parse(result.inputs_json);
          onLoad(savedRegion === region ? rest : reseedForRegion(rest));
        }
      } catch {
        // No persisted draft, or storage is unavailable -- fall back to
        // whatever is already on screen.
      } finally {
        if (!cancelled) {
          hydrated.current = true;
          onHydrated?.();
        }
      }
    })();

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wasmModule]);

  useEffect(() => {
    if (!wasmModule || !hydrated.current) return undefined;
    const json = JSON.stringify({ ...getCurrentInputs(), region });
    if (json === lastSavedRef.current) return undefined;

    const id = setTimeout(() => {
      lastSavedRef.current = json;
      wasmModule.save_current_inputs?.(storageKey, json)?.catch?.(() => {});
    }, 500);
    return () => clearTimeout(id);
  });

  useEffect(() => {
    if (dataVersion === dataVersionSeen.current) return;
    dataVersionSeen.current = dataVersion;
    onLoad(defaultInputs);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dataVersion]);
}
