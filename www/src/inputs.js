import { useRef } from 'react';

/**
 * True when every value is something the wasm boundary can read as a number.
 *
 * `NumberField` yields `''` for an empty box, and serde on the Rust side
 * expects an `f64` — handing it `''` fails to deserialize. Guarding here
 * means the calculators simply don't call across the boundary while an input
 * is blank, rather than relying on the error path to describe the mistake.
 */
export function allFilled(...values) {
  return values.every(
    (v) => v !== '' && v !== null && v !== undefined && Number.isFinite(Number(v)),
  );
}

/**
 * Keeps the last usable result on screen while an input is mid-edit.
 *
 * Clearing a field to retype it is the most common interaction in the app,
 * and it leaves the inputs briefly incomplete. Blanking the results in that
 * gap makes the page flicker and reflow on every keystroke, so the previous
 * figures stay put. `stale` lets the caller dim them, so held numbers are
 * never mistaken for a live answer.
 */
export function useSticky(result) {
  const last = useRef(null);
  if (result != null) last.current = result;
  return {
    value: result ?? last.current,
    stale: result == null && last.current != null,
  };
}
