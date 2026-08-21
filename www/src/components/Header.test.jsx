import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Header from './Header';

describe('Header', () => {
  it('marks the active tab and no other', () => {
    render(<Header activeTab="refinance" onTabChange={() => {}} />);

    expect(screen.getByRole('button', { name: 'Refinance' })).toHaveClass('active');
    expect(screen.getByRole('button', { name: 'Payment' })).not.toHaveClass('active');
  });

  it('calls onTabChange with the clicked tab id', async () => {
    const onTabChange = vi.fn();
    render(<Header activeTab="payment" onTabChange={onTabChange} />);

    await userEvent.click(screen.getByRole('button', { name: 'Compare' }));

    expect(onTabChange).toHaveBeenCalledWith('compare');
  });
});
