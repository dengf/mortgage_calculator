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

async function main() {
  // Before rendering: if this page is a cached copy from before the last
  // deploy, it reloads onto the current one rather than quietly running
  // stale code.
  startVersionCheck();

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
