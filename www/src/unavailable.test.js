import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createUnavailableModule } from './unavailable';

beforeEach(() => {
  vi.spyOn(console, 'warn').mockImplementation(() => {});
});

const CALCULATIONS = [
  'calculate_payment',
  'calculate_amortization_schedule',
  'calculate_extra_payment_impact',
  'calculate_affordability',
  'calculate_refinance',
  'calculate_comparison',
  'calculate_singapore',
  'calculate_united_states',
  'calculate_sg_affordability',
];

describe('the module used when the engine cannot be loaded', () => {
  it.each(CALCULATIONS)('%s reports the failure instead of computing', (name) => {
    // This used to be a second implementation of the mortgage math in
    // JavaScript. A browser that failed to instantiate wasm got a plausible
    // answer from code no test in this repo covers -- one that could differ
    // from the Rust core without anything saying so.
    const result = createUnavailableModule()[name]({});

    expect(result.error).toBeTruthy();
    expect(result.error_message.code).toBe('err.engineUnavailable');
    expect(result.payment).toBeUndefined();
    expect(result.rows).toBeUndefined();
  });

  it('offers no rate presets rather than inventing US benchmarks', () => {
    expect(createUnavailableModule().get_common_rate_presets()).toEqual([]);
  });

  it('leaves the derived-figure bindings absent for their callers to handle', () => {
    // Stubbing these would re-create in JavaScript exactly what was moved out
    // of it. Their callers already cope with a module that cannot answer.
    const m = createUnavailableModule();
    for (const name of [
      'detect_region',
      'summarize_scenario',
      'down_payment_for_percent',
      'describe_duration',
    ]) {
      expect(m[name]).toBeUndefined();
    }
  });

  it('still saves and loads scenarios, since storage is not a mortgage rule', async () => {
    const m = createUnavailableModule();
    await m.init_storage();

    const saved = await m.save_scenario({
      calculator: 'payment',
      name: 'A run',
      inputs_json: '{"homePrice":500000}',
    });
    const listed = await m.list_scenarios('payment');
    const loaded = await m.load_scenario(saved.id);

    expect(listed.scenarios).toHaveLength(1);
    expect(JSON.parse(loaded.scenario.inputs_json).homePrice).toBe(500000);

    await m.delete_scenario(saved.id);
    expect((await m.list_scenarios('payment')).scenarios).toHaveLength(0);
  });

  it('does not hand one calculator another calculator saved runs', async () => {
    const m = createUnavailableModule();
    await m.save_scenario({ calculator: 'payment', name: 'p', inputs_json: '{}' });

    expect((await m.list_scenarios('refinance')).scenarios).toHaveLength(0);
  });

  it('clears all saved scenarios, since YourDataMenu calls this unconditionally', async () => {
    const m = createUnavailableModule();
    await m.save_scenario({ calculator: 'payment', name: 'p', inputs_json: '{}' });
    await m.save_scenario({ calculator: 'refinance', name: 'r', inputs_json: '{}' });

    const result = await m.clear_all_scenarios();

    expect(result.error).toBeNull();
    expect((await m.list_scenarios()).scenarios).toHaveLength(0);
  });
});
