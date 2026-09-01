import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { startVersionCheck } from './version-check';
import { createUnavailableModule } from './unavailable';
import './styles/main.css';

let wasmModule = null;

async function initWasm() {
  try {
    // Dynamic import of the WASM module (--target web build).
    const wasm = await import('../pkg');
    // --target web ships an async default init function that instantiates
    // and links the .wasm binary; it must run before any bound function is
    // called.
    if (wasm.default) {
      await wasm.default();
    }
    await wasm.init_storage();
    wasmModule = wasm;
    console.log('WASM module initialized successfully');
    return wasm;
  } catch (error) {
    console.error('Failed to initialize WASM module:', error);
    return createUnavailableModule();
  }
}

export function getWasmModule() {
  return wasmModule;
}

// A stale-but-visible tab can't be reloaded out from under someone (see
// version-check.js's own doc comment on why), so the only way to close
// that gap is to tell them -- otherwise a deploy that landed while their
// tab stayed open and focused is invisible to them indefinitely, not just
// for the ten minutes GitHub Pages caches HTML for. This runs before
// React mounts, so the event carries the news to whichever component
// ends up listening rather than assuming one exists yet.
function notifyStaleVersion(buildId) {
  window.dispatchEvent(new CustomEvent('mc:stale-version', { detail: { buildId } }));
}

async function main() {
  // Before rendering: if this page is a cached copy from before the last
  // deploy, it reloads onto the current one rather than quietly running
  // stale code. If the tab is already open and visible, it can't safely
  // reload out from under whoever is using it -- notifyStaleVersion tells
  // UpdateBanner instead.
  startVersionCheck({ onStale: notifyStaleVersion });

  const wasm = await initWasm();
  wasmModule = wasm;

  const container = document.getElementById('root');
  const root = createRoot(container);
  root.render(
    <React.StrictMode>
      <App wasmModule={wasm} />
    </React.StrictMode>,
  );
}

main();
