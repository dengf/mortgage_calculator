import React from 'react';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import SavedScenarios from './SavedScenarios';

function mockWasmModule(overrides = {}) {
  return {
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    save_scenario: vi.fn(async () => ({ error: null })),
    load_scenario: vi.fn(async () => ({ error: null })),
    delete_scenario: vi.fn(async () => ({ error: null })),
    ...overrides,
  };
}

describe('SavedScenarios', () => {
  it('lists scenarios returned by the wasm store on mount', async () => {
    const wasmModule = mockWasmModule({
      list_scenarios: vi.fn(async () => ({
        scenarios: [{ id: 'a', name: '30yr fixed', created_at: Date.now() }],
        error: null,
      })),
    });
    render(
      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({})}
        onLoad={() => {}}
      />,
    );

    expect(await screen.findByText('30yr fixed')).toBeInTheDocument();
    expect(wasmModule.list_scenarios).toHaveBeenCalledWith('payment');
  });

  it('saves the current inputs under the entered name and refreshes the list', async () => {
    const wasmModule = mockWasmModule();
    const getCurrentInputs = vi.fn(() => ({ principal: 400000 }));
    render(
      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={getCurrentInputs}
        onLoad={() => {}}
      />,
    );
    await waitFor(() => expect(wasmModule.list_scenarios).toHaveBeenCalledTimes(1));

    await userEvent.click(screen.getByRole('button', { name: '+ Save current as...' }));
    await userEvent.type(screen.getByPlaceholderText('Scenario name'), 'My scenario');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    await waitFor(() =>
      expect(wasmModule.save_scenario).toHaveBeenCalledWith(
        expect.objectContaining({
          calculator: 'payment',
          name: 'My scenario',
          inputs_json: JSON.stringify({ principal: 400000 }),
        }),
      ),
    );
    // Save triggers a second list_scenarios call to pick up the new entry.
    await waitFor(() => expect(wasmModule.list_scenarios).toHaveBeenCalledTimes(2));
  });

  it('does not save when the name field is left blank', async () => {
    const wasmModule = mockWasmModule();
    render(
      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({})}
        onLoad={() => {}}
      />,
    );
    await waitFor(() => expect(wasmModule.list_scenarios).toHaveBeenCalledTimes(1));

    await userEvent.click(screen.getByRole('button', { name: '+ Save current as...' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(wasmModule.save_scenario).not.toHaveBeenCalled();
  });

  it('loads a scenario and hands its parsed inputs to onLoad', async () => {
    const wasmModule = mockWasmModule({
      list_scenarios: vi.fn(async () => ({
        scenarios: [{ id: 'a', name: '30yr fixed', created_at: Date.now() }],
        error: null,
      })),
      load_scenario: vi.fn(async () => ({
        scenario: { inputs_json: JSON.stringify({ principal: 400000, rate: 6.5 }) },
        error: null,
      })),
    });
    const onLoad = vi.fn();
    render(
      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({})}
        onLoad={onLoad}
      />,
    );

    await userEvent.click(await screen.findByRole('button', { name: 'Load' }));

    expect(wasmModule.load_scenario).toHaveBeenCalledWith('a');
    await waitFor(() => expect(onLoad).toHaveBeenCalledWith({ principal: 400000, rate: 6.5 }));
  });

  it('reports a corrupt scenario instead of throwing when its inputs_json will not parse', async () => {
    // A record that got into storage some other way than this app's own save
    // path (a poisoned import predating the backup.js validation, or direct
    // tampering) must surface as a load error, not an uncaught exception.
    const wasmModule = mockWasmModule({
      list_scenarios: vi.fn(async () => ({
        scenarios: [{ id: 'a', name: '30yr fixed', created_at: Date.now() }],
        error: null,
      })),
      load_scenario: vi.fn(async () => ({
        scenario: { inputs_json: '{not valid json' },
        error: null,
      })),
    });
    const onLoad = vi.fn();
    render(
      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({})}
        onLoad={onLoad}
      />,
    );

    await userEvent.click(await screen.findByRole('button', { name: 'Load' }));

    expect(
      await screen.findByText("This saved scenario is corrupted and can't be loaded."),
    ).toBeInTheDocument();
    expect(onLoad).not.toHaveBeenCalled();
  });

  it('shows the store error instead of silently failing on a save error', async () => {
    const wasmModule = mockWasmModule({
      save_scenario: vi.fn(async () => ({ error: 'storage quota exceeded' })),
    });
    render(
      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({})}
        onLoad={() => {}}
      />,
    );
    await waitFor(() => expect(wasmModule.list_scenarios).toHaveBeenCalledTimes(1));

    await userEvent.click(screen.getByRole('button', { name: '+ Save current as...' }));
    await userEvent.type(screen.getByPlaceholderText('Scenario name'), 'x');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(await screen.findByText('storage quota exceeded')).toBeInTheDocument();
  });

  it('asks for confirmation before deleting a scenario, and does not delete until confirmed', async () => {
    const wasmModule = mockWasmModule({
      list_scenarios: vi.fn(async () => ({
        scenarios: [{ id: 'a', name: '30yr fixed', created_at: Date.now() }],
        error: null,
      })),
    });
    render(
      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({})}
        onLoad={() => {}}
      />,
    );

    await userEvent.click(await screen.findByRole('button', { name: 'Delete' }));

    expect(await screen.findByText('Delete "30yr fixed"? This cannot be undone.')).toBeInTheDocument();
    expect(wasmModule.delete_scenario).not.toHaveBeenCalled();

    const dialog = screen.getByRole('alertdialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));

    await waitFor(() => expect(wasmModule.delete_scenario).toHaveBeenCalledWith('a'));
  });

  it('leaves the scenario in place when the delete confirmation is cancelled', async () => {
    const wasmModule = mockWasmModule({
      list_scenarios: vi.fn(async () => ({
        scenarios: [{ id: 'a', name: '30yr fixed', created_at: Date.now() }],
        error: null,
      })),
    });
    render(
      <SavedScenarios
        wasmModule={wasmModule}
        calculatorKind="payment"
        getCurrentInputs={() => ({})}
        onLoad={() => {}}
      />,
    );

    await userEvent.click(await screen.findByRole('button', { name: 'Delete' }));
    const dialog = screen.getByRole('alertdialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));

    expect(wasmModule.delete_scenario).not.toHaveBeenCalled();
    expect(screen.getByText('30yr fixed')).toBeInTheDocument();
  });
});
