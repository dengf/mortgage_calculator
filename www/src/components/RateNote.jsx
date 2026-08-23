import React, { useMemo } from 'react';
import { toRateTypeDto } from '../rate';
import { useI18n } from '../i18n';

/**
 * What a quoted rate assumed, shown beside the figures it produced.
 *
 * A rate quoted over 3M SORA, SOFR or Prime is a snapshot: the index is
 * published by someone else, it moves daily, and this app ships no market
 * feed — so every amount on the tab is exact *given* a number that was held
 * still. Saying so is the difference between an illustration and a claim.
 *
 * Whether there is anything to say is not decided here. `rate_note` asks
 * `RateType::floating_base` and returns the sentence as a code plus its
 * values, so the tabs and the printed document say the same thing in the
 * reader's language. See CLAUDE.md.
 */
export default function RateNote({ wasmModule, rate }) {
  const { t } = useI18n();
  const note = useMemo(() => {
    if (!wasmModule?.rate_note) return null;
    try {
      return wasmModule.rate_note(toRateTypeDto(rate));
    } catch {
      // A note is a courtesy on top of a working tab. If the boundary call
      // fails the figures are still right, and blanking the page over a
      // missing caveat would be the worse outcome.
      return null;
    }
  }, [wasmModule, rate]);

  if (!note) return null;

  return (
    <p className="rate-note" role="note">
      {t(note.code, note.params) || note.text}
    </p>
  );
}
