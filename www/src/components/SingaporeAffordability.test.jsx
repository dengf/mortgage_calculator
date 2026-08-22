import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import SingaporeAffordability from './SingaporeAffordability';
import { I18nProvider } from '../i18n';

function mockWasm(overrides = {}) {
  return {
    calculate_sg_affordability: vi.fn(() => ({
      max_price: 1_200_000,
      max_loan: 900_000,
      binding_constraint: 'tdsr',
      max_monthly_instalment: 6600,
      assessment_rate_percent: 4,
      ltv_percent: 75,
      extended_tenure: false,
      deposit: 300_000,
      min_cash_required: 60_000,
      bsd: 32_600,
      absd: 0,
      cash_and_cpf_at_completion: 332_600,
      error: null,
    })),
    list_scenarios: vi.fn(async () => ({ scenarios: [], error: null })),
    ...overrides,
  };
}

const renderPanel = (wasmModule) =>
  render(
    <I18nProvider initialLocale="en">
      <SingaporeAffordability wasmModule={wasmModule} />
    </I18nProvider>,
  );

describe('SingaporeAffordability', () => {
  it('asks Singapore questions, not American ones', async () => {
    renderPanel(mockWasm());
    // The US model's vocabulary must not appear here at all — that was the
    // bug: SG users were shown DTI, HOA and a flat property-tax rate.
    const body = document.body.textContent;
    expect(body).not.toMatch(/debt-to-income|DTI|HOA/i);
    expect(await screen.findByText(/^Cash available$/i)).toBeInTheDocument();
    expect(screen.getByText(/Housing loans outstanding/i)).toBeInTheDocument();
  });

  it('reaches the Singapore binding with the loan-count and residency inputs', async () => {
    const wasmModule = mockWasm();
    renderPanel(wasmModule);
    await waitFor(() => expect(wasmModule.calculate_sg_affordability).toHaveBeenCalled());

    const args = wasmModule.calculate_sg_affordability.mock.calls.at(-1)[0];
    expect(args).toMatchObject({
      residency: 'Citizen',
      property_count: '1st',
      outstanding_housing_loans: 0,
      is_hdb_or_ec: false,
    });
    expect(args).not.toHaveProperty('max_dti_percent');
    expect(args).not.toHaveProperty('monthly_hoa');
  });

  it('names the rule that caps the budget', async () => {
    renderPanel(mockWasm());
    expect(await screen.findByText(/Limited by TDSR/i)).toBeInTheDocument();
  });

  it('spells out the minimum cash portion, which CPF cannot cover', async () => {
    renderPanel(mockWasm());
    expect(await screen.findByText(/must be cash, not CPF/i)).toBeInTheDocument();
  });

  it('asks for cash and CPF separately, since they are not interchangeable', async () => {
    const wasmModule = mockWasm();
    renderPanel(wasmModule);
    await waitFor(() => expect(wasmModule.calculate_sg_affordability).toHaveBeenCalled());

    const args = wasmModule.calculate_sg_affordability.mock.calls.at(-1)[0];
    expect(args).toHaveProperty('cash_available');
    expect(args).toHaveProperty('cpf_oa_available');
    expect(args).not.toHaveProperty('funds_available');
  });

  it('splits fixed from variable income, because MAS haircuts the variable half', async () => {
    const wasmModule = mockWasm();
    renderPanel(wasmModule);
    await waitFor(() => expect(wasmModule.calculate_sg_affordability).toHaveBeenCalled());

    const args = wasmModule.calculate_sg_affordability.mock.calls.at(-1)[0];
    expect(args).toHaveProperty('fixed_monthly_income');
    expect(args).toHaveProperty('variable_monthly_income');
    expect(args).not.toHaveProperty('gross_monthly_income');
  });

  it('warns that the FTA remission is claimed, not automatic', async () => {
    renderPanel(mockWasm());
    const select = await screen.findByDisplayValue('Citizen');
    await userEvent.selectOptions(select, 'FTA');
    // A buyer who assumes it applies at the counter will be short the whole
    // foreigner ABSD on completion day.
    expect(
      await screen.findByText(/claimed from IRAS, not applied automatically/i),
    ).toBeInTheDocument();
  });

  it('degrades to a message rather than crashing on a cached wasm without the binding', async () => {
    // A returning visitor can hold a wasm build from before this binding
    // existed; calling a missing export would take down the whole tab.
    const { calculate_sg_affordability, ...withoutBinding } = mockWasm();
    expect(() => renderPanel(withoutBinding)).not.toThrow();
  });
});
