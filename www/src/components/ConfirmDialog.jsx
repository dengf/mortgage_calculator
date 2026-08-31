import React, { useCallback, useRef, useState } from 'react';
import { useI18n } from '../i18n';

/**
 * A promise-based replacement for `window.confirm()`.
 *
 * `window.confirm` isn't available in every context this app renders in
 * (this project's own browser-preview tooling included), and a native
 * browser dialog can't be styled or translated. `useConfirm` returns an
 * async `confirm(message, confirmLabel)` function and the dialog element
 * to render once at the app root; every destructive action awaits it
 * before acting.
 */
export function useConfirm() {
  const [state, setState] = useState(null); // { message, confirmLabel, resolve }
  const resolverRef = useRef(null);

  // `confirmLabel` names the action being confirmed -- a generic "OK"
  // reads fine for nothing here, since both call sites (Replace on
  // import, Clear all data) are actively misleading under any shared
  // default, so every caller names its own verb.
  const confirm = useCallback((message, confirmLabel) => {
    return new Promise((resolve) => {
      resolverRef.current = resolve;
      setState({ message, confirmLabel });
    });
  }, []);

  const answer = useCallback((value) => {
    resolverRef.current?.(value);
    resolverRef.current = null;
    setState(null);
  }, []);

  const dialog = state ? (
    <ConfirmDialogView message={state.message} confirmLabel={state.confirmLabel} onAnswer={answer} />
  ) : null;

  return [confirm, dialog];
}

function ConfirmDialogView({ message, confirmLabel, onAnswer }) {
  const { t } = useI18n();
  return (
    <div className="confirm-backdrop" role="presentation" onClick={() => onAnswer(false)}>
      <div
        className="confirm-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-describedby="confirm-message"
        onClick={(e) => e.stopPropagation()}
      >
        <p className="confirm-message" id="confirm-message">
          {message}
        </p>
        <div className="confirm-actions">
          <button className="secondary-button" onClick={() => onAnswer(false)}>
            {t('confirm.cancel')}
          </button>
          <button className="danger-button" onClick={() => onAnswer(true)} autoFocus>
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
