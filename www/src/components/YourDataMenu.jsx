import React, { useRef, useState } from 'react';
import { useI18n } from '../i18n';
import { EXPORT_FORMAT, readBackup } from '../backup';
import { useConfirm } from './ConfirmDialog';

/**
 * "Your data" as a nav-level dropdown rather than a tab -- it isn't a
 * page of its own, just three one-shot actions (export/import/clear)
 * that apply to every saved scenario across every calculator, regardless
 * of which tab happens to be open. Living in the nav means it's reachable
 * without a tab switch losing whatever the person was looking at, and it
 * rides along on the sticky-on-mobile nav for free.
 *
 * Ported from budget_planner's YourDataMenu with its fixes already in
 * place rather than rediscovered: the file input lives outside the
 * fixed-position dialog (see the comment on it below), and a successful
 * import or clear closes the dropdown instead of leaving it open.
 */
export default function YourDataMenu({ wasmModule, onDataChanged }) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [importResult, setImportResult] = useState(null);
  const fileInputRef = useRef(null);
  const [confirm, confirmDialog] = useConfirm();

  const exportData = async () => {
    const result = await wasmModule.list_scenarios(undefined);
    const payload = {
      format: EXPORT_FORMAT,
      exported_at: new Date().toISOString(),
      scenarios: result.scenarios ?? [],
    };
    const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mortgage-calculator-scenarios-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const onImportFile = (e) => {
    const file = e.target.files?.[0];
    // Reset immediately so picking the same file twice still fires a change.
    e.target.value = '';
    if (!file) return;
    const reader = new FileReader();
    reader.onload = async () => {
      let payload;
      try {
        payload = JSON.parse(String(reader.result ?? ''));
      } catch {
        setImportResult({ error: t('err.badImportFile') });
        return;
      }
      const backup = readBackup(payload);
      if (!backup.ok) {
        setImportResult({ error: t(backup.reason) });
        return;
      }

      const proceed = await confirm(
        t('data.importConfirm', { count: backup.count }),
        t('confirm.replace'),
      );
      if (!proceed) return;

      const clearResult = await wasmModule.clear_all_scenarios();
      if (clearResult.error) {
        setImportResult({ error: clearResult.error });
        return;
      }
      for (const scenario of backup.scenarios) {
        const saveResult = await wasmModule.save_scenario({
          calculator: scenario.calculator,
          name: scenario.name,
          inputs_json: scenario.inputs_json,
          id: scenario.id,
          created_at: scenario.created_at,
        });
        // Stop rather than pressing on into the rest of the file -- the
        // store is already cleared at this point, so what's imported so
        // far is genuinely what's there, not a false "done" over a partial
        // restore.
        if (saveResult.error) {
          setImportResult({ error: saveResult.error });
          return;
        }
      }
      onDataChanged?.();
      setOpen(false);
    };
    reader.readAsText(file);
  };

  const onClearAll = async () => {
    const proceed = await confirm(t('data.clearConfirm'), t('data.clearAll'));
    if (!proceed) return;
    const result = await wasmModule.clear_all_scenarios();
    if (result.error) {
      setImportResult({ error: result.error });
      return;
    }
    onDataChanged?.();
    setOpen(false);
  };

  const openMenu = () => {
    setImportResult(null);
    setOpen(true);
  };

  return (
    <div className="data-menu">
      <button
        type="button"
        className="app-tab data-menu-trigger"
        aria-haspopup="true"
        aria-expanded={open}
        onClick={openMenu}
      >
        {t('data.title')}
        <svg className="data-menu-caret" width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <path
            d="M1.5 3.5L5 7L8.5 3.5"
            stroke="currentColor"
            strokeWidth="1.5"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>

      {/* Deliberately NOT inside .data-menu-dialog (position: fixed) --
          iOS Safari/Chrome (both WebKit) can silently fail to open the
          native file picker for an <input type="file"> nested inside a
          fixed-position ancestor. Living here, in normal flow, and
          triggered via ref from a plain button in the dialog sidesteps
          that. */}
      <input
        ref={fileInputRef}
        type="file"
        accept="application/json,.json"
        onChange={onImportFile}
        style={{ display: 'none' }}
      />

      {open && (
        <div className="data-menu-backdrop" role="presentation" onClick={() => setOpen(false)}>
          <div
            className="data-menu-dialog"
            role="dialog"
            aria-modal="true"
            aria-label={t('data.title')}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="data-menu-header">
              <span className="data-menu-title">{t('data.title')}</span>
              <button
                type="button"
                className="data-menu-close"
                aria-label={t('data.close')}
                onClick={() => setOpen(false)}
              >
                ×
              </button>
            </div>
            <p className="data-menu-hint">{t('data.exportHint')}</p>

            <div className="data-menu-actions">
              <button
                type="button"
                className="secondary-button"
                onClick={async () => {
                  await exportData();
                  setOpen(false);
                }}
              >
                {t('data.export')}
              </button>
              <button
                type="button"
                className="secondary-button"
                onClick={() => fileInputRef.current?.click()}
              >
                {t('data.import')}
              </button>
              <button type="button" className="danger-button" onClick={onClearAll}>
                {t('data.clearAll')}
              </button>
            </div>

            {importResult?.error && (
              <p className="error" role="alert">
                {importResult.error}
              </p>
            )}
          </div>
        </div>
      )}

      {confirmDialog}
    </div>
  );
}
