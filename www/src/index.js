import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { startVersionCheck } from './version-check';
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
    return createMockModule();
  }
}

// Mock module for development/testing without a wasm-pack build. Mirrors
// the shape of the real bindings (see crates/mortgage-wasm) closely enough
// that `npm start` works before you've ever run `npm run build:wasm`.
function createMockModule() {
  console.warn('Using mock WASM module - run `npm run build:wasm` for full functionality');

  const monthlyPayment = (principal, annualRatePercent, termYears) => {
    const r = annualRatePercent / 100 / 12;
    const n = termYears * 12;
    if (r === 0) return principal / n;
    const growth = Math.pow(1 + r, n);
    return (principal * r * growth) / (growth - 1);
  };

  return {
    // Same reasoning as the Singapore rules below: the ranking between time
    // zone, language and stored preference lives in
    // mortgage-core/src/region.rs, and a second copy here could disagree
    // with it. Without the real module the app just opens in its default
    // region, which the header toggle fixes in one click.
    detect_region: () => ({ region: 'US' }),
    // Deliberately not reimplemented here. The MAS/CPF/IRAS rules live in
    // exactly one place (mortgage-calc/src/singapore.rs); mirroring the
    // TDSR/MSR ceilings, BSD tiers and ABSD matrix in JavaScript would let
    // the two drift apart and give a wrong answer that looks right. Report
    // the real reason instead — the panel renders it as an error.
    calculate_singapore: () => ({
      warnings: [],
      error: 'Singapore rules need the WASM module — run `npm run build:wasm`.',
    }),
    calculate_united_states: () => ({
      error: 'US costs need the WASM module — run `npm run build:wasm`.',
    }),
    calculate_payment: (params) => {
      const payment = monthlyPayment(
        params.principal,
        params.annual_rate_percent,
        params.term_years,
      );
      const totalPeriods = params.term_years * 12;
      const totalPaid = payment * totalPeriods;
      return {
        payment,
        total_periods: totalPeriods,
        total_paid: totalPaid,
        total_interest: totalPaid - params.principal,
        error: null,
      };
    },
    calculate_amortization_schedule: (params) => {
      const { loan, extra_payment: extraPayment } = params;
      const r = loan.annual_rate_percent / 100 / 12;
      const payment = monthlyPayment(loan.principal, loan.annual_rate_percent, loan.term_years);
      const maxPeriods = loan.term_years * 12;
      let balance = loan.principal;
      const rows = [];
      for (let period = 1; period <= maxPeriods && balance > 0; period++) {
        const interestPortion = balance * r;
        let principalPortion = payment - interestPortion + extraPayment;
        let periodPayment = payment + extraPayment;
        if (principalPortion >= balance || period === maxPeriods) {
          principalPortion = balance;
          periodPayment = balance + interestPortion;
        }
        balance = Math.max(0, balance - principalPortion);
        rows.push({
          period,
          payment: periodPayment,
          extra_payment: extraPayment,
          principal_portion: principalPortion,
          interest_portion: interestPortion,
          remaining_balance: balance,
        });
      }
      return { rows, error: null };
    },
    calculate_extra_payment_impact: function (params) {
      const baseline = this.calculate_amortization_schedule({ ...params, extra_payment: 0 }).rows;
      const withExtra = this.calculate_amortization_schedule(params).rows;
      const sumInterest = (rows) => rows.reduce((total, row) => total + row.interest_portion, 0);
      const baselineInterest = sumInterest(baseline);
      const extraInterest = sumInterest(withExtra);
      return {
        baseline_periods: baseline.length,
        payoff_periods: withExtra.length,
        periods_saved: baseline.length - withExtra.length,
        baseline_total_interest: baselineInterest,
        total_interest_with_extra: extraInterest,
        interest_saved: baselineInterest - extraInterest,
        error: null,
      };
    },
    calculate_affordability: (params) => {
      const maxHousingPayment = Math.max(
        0,
        (params.gross_monthly_income * params.max_dti_percent) / 100 - params.monthly_debts,
      );
      const r = params.annual_rate_percent / 100 / 12;
      const n = params.term_years * 12;
      const k = r === 0 ? 1 / n : (r * Math.pow(1 + r, n)) / (Math.pow(1 + r, n) - 1);
      const monthlyInsurance = params.annual_insurance / 12;
      const monthlyTaxRate = params.annual_property_tax_rate_percent / 100 / 12;
      const budget =
        maxHousingPayment -
        monthlyInsurance -
        params.monthly_hoa -
        params.down_payment * monthlyTaxRate;
      const maxLoanAmount = Math.max(0, budget / (k + monthlyTaxRate));
      const maxHomePrice = maxLoanAmount + params.down_payment;
      const maxPI = maxLoanAmount * k;
      const monthlyTax = maxHomePrice * monthlyTaxRate;
      const totalHousingCost = maxPI + monthlyTax + monthlyInsurance + params.monthly_hoa;
      return {
        max_monthly_housing_payment: maxHousingPayment,
        max_principal_and_interest: maxPI,
        max_loan_amount: maxLoanAmount,
        max_home_price: maxHomePrice,
        front_end_dti_percent: (totalHousingCost / params.gross_monthly_income) * 100,
        back_end_dti_percent:
          ((totalHousingCost + params.monthly_debts) / params.gross_monthly_income) * 100,
        error: null,
      };
    },
    calculate_refinance: (params) => {
      const currentR = params.current_annual_rate_percent / 100 / 12;
      const currentGrowth = Math.pow(1 + currentR, params.remaining_periods);
      const currentPayment =
        currentR === 0
          ? params.current_balance / params.remaining_periods
          : (params.current_balance * currentR * currentGrowth) / (currentGrowth - 1);
      const newPayment = monthlyPayment(
        params.current_balance,
        params.new_annual_rate_percent,
        params.new_term_years,
      );
      const newPeriods = params.new_term_years * 12;
      const paymentSavings = currentPayment - newPayment;
      const breakEvenPeriods =
        paymentSavings > 0 ? Math.max(1, Math.ceil(params.closing_costs / paymentSavings)) : null;
      const remainingInterestCurrent =
        currentPayment * params.remaining_periods - params.current_balance;
      const totalInterestNew = newPayment * newPeriods - params.current_balance;
      return {
        current_payment: currentPayment,
        new_payment: newPayment,
        payment_savings: paymentSavings,
        break_even_periods: breakEvenPeriods,
        remaining_interest_on_current_loan: remainingInterestCurrent,
        total_interest_on_new_loan: totalInterestNew,
        lifetime_savings: remainingInterestCurrent - totalInterestNew - params.closing_costs,
        error: null,
      };
    },
    get_common_rate_presets: () => [
      { label: '30-Year Fixed', rate_type: { kind: 'fixed', rate_percent: 6.5 }, term_years: 30 },
      { label: '20-Year Fixed', rate_type: { kind: 'fixed', rate_percent: 6.25 }, term_years: 20 },
      { label: '15-Year Fixed', rate_type: { kind: 'fixed', rate_percent: 6.0 }, term_years: 15 },
      {
        label: 'Floating: SOFR + 2.00%',
        rate_type: { kind: 'floating', base_rate_percent: 4.3, spread_percent: 2.0 },
        term_years: 30,
      },
      {
        label: 'Floating: SOFR + 2.50%',
        rate_type: { kind: 'floating', base_rate_percent: 4.3, spread_percent: 2.5 },
        term_years: 30,
      },
      {
        label: 'Floating: Prime + 0.50%',
        rate_type: { kind: 'floating', base_rate_percent: 7.5, spread_percent: 0.5 },
        term_years: 30,
      },
    ],
    calculate_comparison: (params) => {
      const effectiveRate = (rateType) =>
        rateType.kind === 'fixed'
          ? rateType.rate_percent
          : rateType.base_rate_percent + rateType.spread_percent;
      const rows = params.entries.map((entry) => {
        const ratePercent = effectiveRate(entry.rate_type);
        const payment = monthlyPayment(params.principal, ratePercent, entry.term_years);
        const totalPeriods = entry.term_years * 12;
        const totalPaid = payment * totalPeriods;
        return {
          label: entry.label,
          effective_rate_percent: ratePercent,
          term_years: entry.term_years,
          payment,
          total_periods: totalPeriods,
          total_paid: totalPaid,
          total_interest: totalPaid - params.principal,
        };
      });
      return { rows, error: null };
    },
    // In-memory scenario store: not persisted across reloads, just enough
    // to exercise the save/load UI before `npm run build:wasm` has run.
    _mockScenarios: [],
    _mockScenarioSeq: 0,
    init_storage: async function () {},
    save_scenario: async function (params) {
      const id = params.id ?? `mock-${this._mockScenarioSeq++}`;
      const scenario = {
        id,
        calculator: params.calculator,
        name: params.name,
        created_at: Date.now(),
        inputs_json: params.inputs_json,
      };
      this._mockScenarios = this._mockScenarios.filter((s) => s.id !== id);
      this._mockScenarios.push(scenario);
      return { id, error: null };
    },
    list_scenarios: async function (calculator) {
      const scenarios = this._mockScenarios
        .filter((s) => !calculator || s.calculator === calculator)
        .sort((a, b) => b.created_at - a.created_at);
      return { scenarios, error: null };
    },
    load_scenario: async function (id) {
      const scenario = this._mockScenarios.find((s) => s.id === id);
      if (!scenario) return { scenario: null, error: `scenario not found: ${id}` };
      return { scenario, error: null };
    },
    delete_scenario: async function (id) {
      this._mockScenarios = this._mockScenarios.filter((s) => s.id !== id);
      return { success: true, error: null };
    },
  };
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
