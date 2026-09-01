import React from 'react';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import UpdateBanner from './UpdateBanner';
import { reloadOnto } from '../version-check';

vi.mock('../version-check', () => ({ reloadOnto: vi.fn() }));

function dispatchStale(buildId) {
  // The real event comes from index.js's window-level listener, entirely
  // outside React's own event system, so React won't batch/flush the
  // resulting state update on its own -- act() forces that flush the same
  // way a user-driven event would get it via testing-library's fireEvent.
  act(() => {
    window.dispatchEvent(new CustomEvent('mc:stale-version', { detail: { buildId } }));
  });
}

afterEach(() => {
  vi.clearAllMocks();
});

describe('UpdateBanner', () => {
  it('renders nothing until a new deploy is announced', () => {
    render(<UpdateBanner />);
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });

  it('shows the banner once the tab is told a new build is deployed', () => {
    render(<UpdateBanner />);

    dispatchStale('new999');

    expect(screen.getByRole('status')).toHaveTextContent('A new version of Mortgage Calculator is ready.');
  });

  it('reloads onto the announced build when Reload is clicked', async () => {
    render(<UpdateBanner />);
    dispatchStale('new999');

    await userEvent.click(screen.getByRole('button', { name: 'Reload' }));

    expect(reloadOnto).toHaveBeenCalledWith('new999');
  });

  it('hides the banner when dismissed, without reloading', async () => {
    render(<UpdateBanner />);
    dispatchStale('new999');

    await userEvent.click(screen.getByRole('button', { name: 'Dismiss' }));

    expect(screen.queryByRole('status')).not.toBeInTheDocument();
    expect(reloadOnto).not.toHaveBeenCalled();
  });
});
