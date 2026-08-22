import React, { useState } from 'react';
import { render } from '@testing-library/react';
import { DEFAULT_SCENARIO } from '../scenario';

/**
 * Renders a scenario-driven panel with real state behind it.
 *
 * These panels are controlled — App owns the scenario so it can be shared
 * across tabs. Handing them a no-op setter would make them silently
 * read-only, and every interaction test would pass while proving nothing.
 */
export function renderControlled(Panel, props = {}, initial = DEFAULT_SCENARIO) {
  function Harness() {
    const [scenario, setScenario] = useState(initial);
    return <Panel {...props} scenario={scenario} onScenarioChange={setScenario} />;
  }
  return render(<Harness />);
}
