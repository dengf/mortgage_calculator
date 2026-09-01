import React from 'react';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import YourDataMenu from './YourDataMenu';
import { EXPORT_FORMAT } from '../backup';

function mockWasmModule(overrides = {}) {
  return {
    list_scenarios: vi.fn(async () => ({
      scenarios: [
        { id: 'a', calculator: 'payment', name: '30yr fixed', created_at: 1700000000000, inputs_json: '{}' },
      ],
      error: null,
    })),
    save_scenario: vi.fn(async () => ({ id: 'x', error: null })),
    clear_all_scenarios: vi.fn(async () => ({ success: true, error: null })),
    clear_current_inputs: vi.fn(async () => ({ success: true, error: null })),
    ...overrides,
  };
}

function makeFile(payload) {
  return new File([JSON.stringify(payload)], 'backup.json', { type: 'application/json' });
}

describe('YourDataMenu', () => {
  beforeEach(() => {
    global.URL.createObjectURL = vi.fn(() => 'blob:mock');
    global.URL.revokeObjectURL = vi.fn();
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('opens on trigger and closes on the dialog close button', async () => {
    render(<YourDataMenu wasmModule={mockWasmModule()} onDataChanged={() => {}} />);
    const trigger = screen.getByRole('button', { name: 'Your data' });
    expect(trigger).toHaveAttribute('aria-expanded', 'false');

    await userEvent.click(trigger);
    expect(trigger).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByRole('dialog')).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Close' }));
    expect(trigger).toHaveAttribute('aria-expanded', 'false');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('exports every scenario as a downloadable JSON file', async () => {
    const wasmModule = mockWasmModule();
    render(<YourDataMenu wasmModule={wasmModule} onDataChanged={() => {}} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));
    await userEvent.click(screen.getByRole('button', { name: 'Export all data' }));

    await waitFor(() => expect(wasmModule.list_scenarios).toHaveBeenCalledWith(undefined));
    expect(global.URL.createObjectURL).toHaveBeenCalled();

    const blob = global.URL.createObjectURL.mock.calls[0][0];
    const payload = JSON.parse(await blob.text());
    expect(payload.format).toBe(EXPORT_FORMAT);
    expect(payload.scenarios).toHaveLength(1);
    expect(payload.scenarios[0].name).toBe('30yr fixed');
  });

  it('imports a valid file once the replace is confirmed, then closes', async () => {
    const wasmModule = mockWasmModule();
    const onDataChanged = vi.fn();
    const { container } = render(<YourDataMenu wasmModule={wasmModule} onDataChanged={onDataChanged} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));

    const file = makeFile({
      format: EXPORT_FORMAT,
      exported_at: '2026-08-29T00:00:00.000Z',
      scenarios: [
        { id: 'imp-1', calculator: 'refinance', name: 'Imported', created_at: 1700000001000, inputs_json: '{}' },
      ],
    });
    const input = container.querySelector('input[type="file"]');
    await userEvent.upload(input, file);

    const confirmDialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(confirmDialog).getByRole('button', { name: 'Replace' }));

    await waitFor(() => expect(wasmModule.clear_all_scenarios).toHaveBeenCalledTimes(1));
    await waitFor(() =>
      expect(wasmModule.save_scenario).toHaveBeenCalledWith({
        calculator: 'refinance',
        name: 'Imported',
        inputs_json: '{}',
        id: 'imp-1',
        created_at: 1700000001000,
      }),
    );
    await waitFor(() => expect(onDataChanged).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
    // Import-replace only touches the named-scenario store -- it must not
    // wipe whatever the user is mid-typing on an unrelated tab.
    expect(wasmModule.clear_current_inputs).not.toHaveBeenCalled();
  });

  it('leaves the data cleared alone when the replace is cancelled', async () => {
    const wasmModule = mockWasmModule();
    const { container } = render(<YourDataMenu wasmModule={wasmModule} onDataChanged={() => {}} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));

    const file = makeFile({
      format: EXPORT_FORMAT,
      scenarios: [{ id: 'imp-1', calculator: 'refinance', name: 'Imported', created_at: 1, inputs_json: '{}' }],
    });
    const input = container.querySelector('input[type="file"]');
    await userEvent.upload(input, file);

    const confirmDialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(confirmDialog).getByRole('button', { name: 'Cancel' }));

    expect(wasmModule.clear_all_scenarios).not.toHaveBeenCalled();
    expect(wasmModule.save_scenario).not.toHaveBeenCalled();
    // The import dialog itself stays open on a cancel.
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('shows an inline error for a file that is not a Mortgage Calculator export, and leaves the dialog open', async () => {
    const wasmModule = mockWasmModule();
    const { container } = render(<YourDataMenu wasmModule={wasmModule} onDataChanged={() => {}} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));

    const file = makeFile({ format: 'something.else', scenarios: [] });
    const input = container.querySelector('input[type="file"]');
    await userEvent.upload(input, file);

    expect(await screen.findByRole('alert')).toHaveTextContent(
      "isn't a Mortgage Calculator export",
    );
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(wasmModule.clear_all_scenarios).not.toHaveBeenCalled();
  });

  it('clears every scenario once confirmed, then closes', async () => {
    const wasmModule = mockWasmModule();
    const onDataChanged = vi.fn();
    render(<YourDataMenu wasmModule={wasmModule} onDataChanged={onDataChanged} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));
    await userEvent.click(screen.getByRole('button', { name: 'Clear all data' }));

    const confirmDialog = await screen.findByRole('alertdialog');
    expect(confirmDialog).toHaveTextContent('This cannot be undone');
    await userEvent.click(within(confirmDialog).getByRole('button', { name: 'Clear all data' }));

    await waitFor(() => expect(wasmModule.clear_all_scenarios).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(onDataChanged).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
  });

  it('clears the in-progress draft alongside saved scenarios, not just on import', async () => {
    const wasmModule = mockWasmModule();
    render(<YourDataMenu wasmModule={wasmModule} onDataChanged={() => {}} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));
    await userEvent.click(screen.getByRole('button', { name: 'Clear all data' }));

    const confirmDialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(confirmDialog).getByRole('button', { name: 'Clear all data' }));

    await waitFor(() => expect(wasmModule.clear_current_inputs).toHaveBeenCalledTimes(1));
  });

  it('shows the store error instead of silently succeeding when clear fails, and leaves the dialog open', async () => {
    const wasmModule = mockWasmModule({
      clear_all_scenarios: vi.fn(async () => ({ success: false, error: 'storage quota exceeded' })),
    });
    const onDataChanged = vi.fn();
    render(<YourDataMenu wasmModule={wasmModule} onDataChanged={onDataChanged} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));
    await userEvent.click(screen.getByRole('button', { name: 'Clear all data' }));

    const confirmDialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(confirmDialog).getByRole('button', { name: 'Clear all data' }));

    expect(await screen.findByText('storage quota exceeded')).toBeInTheDocument();
    expect(onDataChanged).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('shows a message instead of failing silently when clear_all_scenarios throws rather than returning an error', async () => {
    const wasmModule = mockWasmModule({
      clear_all_scenarios: vi.fn(async () => {
        throw new Error('IndexedDB is unavailable in Private Browsing');
      }),
    });
    const onDataChanged = vi.fn();
    render(<YourDataMenu wasmModule={wasmModule} onDataChanged={onDataChanged} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));
    await userEvent.click(screen.getByRole('button', { name: 'Clear all data' }));

    const confirmDialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(confirmDialog).getByRole('button', { name: 'Clear all data' }));

    expect(await screen.findByText(/IndexedDB is unavailable in Private Browsing/)).toBeInTheDocument();
    expect(onDataChanged).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('shows the store error instead of silently succeeding when an import fails partway through', async () => {
    const wasmModule = mockWasmModule({
      save_scenario: vi.fn(async () => ({ error: 'storage quota exceeded' })),
    });
    const onDataChanged = vi.fn();
    const { container } = render(<YourDataMenu wasmModule={wasmModule} onDataChanged={onDataChanged} />);
    await userEvent.click(screen.getByRole('button', { name: 'Your data' }));

    const file = makeFile({
      format: EXPORT_FORMAT,
      scenarios: [{ id: 'imp-1', calculator: 'refinance', name: 'Imported', created_at: 1, inputs_json: '{}' }],
    });
    const input = container.querySelector('input[type="file"]');
    await userEvent.upload(input, file);

    const confirmDialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(confirmDialog).getByRole('button', { name: 'Replace' }));

    expect(await screen.findByText('storage quota exceeded')).toBeInTheDocument();
    expect(onDataChanged).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });
});
