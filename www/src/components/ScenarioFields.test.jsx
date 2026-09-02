import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import ScenarioFields from './ScenarioFields';
import { scenarioBindings } from '../test/wasm';
import { DEFAULT_SCENARIO } from '../scenario';
import { makeFormatMoney } from '../currency';

function renderFields(props = {}) {
  return render(
    <ScenarioFields
      wasmModule={scenarioBindings()}
      scenario={DEFAULT_SCENARIO}
      onChange={vi.fn()}
      money="$"
      formatMoney={makeFormatMoney('US')}
      {...props}
    />,
  );
}

describe('ScenarioFields', () => {
  it('renders the fields directly with no toggle when not collapsible', () => {
    renderFields();

    expect(screen.getByText('Home price')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Loan details' })).not.toBeInTheDocument();
  });

  it('starts folded away when collapsible, showing a summary instead of the fields', () => {
    renderFields({ collapsible: true });

    expect(screen.queryByText('Home price')).not.toBeInTheDocument();
    const toggle = screen.getByRole('button', { name: /Loan details/ });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    // Enough of the loan to know which one this is without opening it.
    expect(toggle).toHaveTextContent('$500,000.00');
  });

  it('expands to show the fields on click, and collapses again on a second click', async () => {
    renderFields({ collapsible: true });

    await userEvent.click(screen.getByRole('button', { name: /Loan details/ }));

    expect(screen.getByText('Home price')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Loan details' })).toHaveAttribute(
      'aria-expanded',
      'true',
    );

    await userEvent.click(screen.getByRole('button', { name: 'Loan details' }));

    expect(screen.queryByText('Home price')).not.toBeInTheDocument();
  });
});
