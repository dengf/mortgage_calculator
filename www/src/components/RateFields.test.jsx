import React, { useState } from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import RateFields from './RateFields';
import { I18nProvider } from '../i18n';
import { DEFAULT_RATE } from '../rate';

function Harness({ initial = DEFAULT_RATE, onRate }) {
  const [rate, setRate] = useState(initial);
  onRate?.(rate);
  return (
    <I18nProvider initialLocale="en">
      <RateFields
        rate={rate}
        onChange={(next) => {
          setRate(next);
          onRate?.(next);
        }}
        wasmModule={{ rate_note: () => null }}
      />
    </I18nProvider>
  );
}

describe('asking what a step-up is quoted over', () => {
  it('asks only of a package that steps up', () => {
    // A floating quote is benchmark-based by construction and a fixed one
    // has no base. Offering the choice there would imply it is a choice.
    render(<Harness initial={{ ...DEFAULT_RATE, kind: 'fixed' }} />);
    expect(screen.queryByLabelText('Base rate floats')).toBeNull();

    render(<Harness initial={{ ...DEFAULT_RATE, kind: 'floating' }} />);
    expect(screen.queryByLabelText('Base rate floats')).toBeNull();
  });

  it('offers the choice on a package, checked', () => {
    render(<Harness initial={{ ...DEFAULT_RATE, kind: 'reverting' }} />);
    expect(screen.getByLabelText('Base rate floats')).toBeChecked();
  });

  it('remembers the answer', async () => {
    let latest = null;
    render(
      <Harness initial={{ ...DEFAULT_RATE, kind: 'reverting' }} onRate={(r) => (latest = r)} />,
    );

    await userEvent.click(screen.getByLabelText('Base rate floats'));

    expect(latest.baseFloats).toBe(false);
    expect(screen.getByLabelText('Base rate floats')).not.toBeChecked();
  });

  it('keeps the answer when the kind is switched away and back', async () => {
    // The form holds every field of every shape for exactly this reason:
    // a user who says their package is quoted off a board rate, glances at
    // a fixed quote and comes back should not have to say it again.
    let latest = null;
    render(
      <Harness
        initial={{ ...DEFAULT_RATE, kind: 'reverting', baseFloats: false }}
        onRate={(r) => (latest = r)}
      />,
    );

    await userEvent.click(screen.getByRole('button', { name: 'Fixed' }));
    await userEvent.click(screen.getByRole('button', { name: 'Steps up' }));

    expect(latest.baseFloats).toBe(false);
  });
});
