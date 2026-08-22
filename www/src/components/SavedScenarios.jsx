import React, { useCallback, useEffect, useState } from 'react';
import { useI18n } from '../i18n';

export default function SavedScenarios({ wasmModule, calculatorKind, getCurrentInputs, onLoad }) {
  const { t } = useI18n();
  const [scenarios, setScenarios] = useState([]);
  const [isSaving, setIsSaving] = useState(false);
  const [nameInput, setNameInput] = useState('');
  const [error, setError] = useState(null);

  const refresh = useCallback(async () => {
    if (!wasmModule) return;
    const result = await wasmModule.list_scenarios(calculatorKind);
    if (result.error) {
      setError(result.error);
      return;
    }
    setError(null);
    setScenarios(result.scenarios);
  }, [wasmModule, calculatorKind]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const handleSave = async () => {
    const name = nameInput.trim();
    if (!name) return;
    const result = await wasmModule.save_scenario({
      calculator: calculatorKind,
      name,
      inputs_json: JSON.stringify(getCurrentInputs()),
      id: undefined,
    });
    if (result.error) {
      setError(result.error);
      return;
    }
    setNameInput('');
    setIsSaving(false);
    refresh();
  };

  const handleLoad = async (id) => {
    const result = await wasmModule.load_scenario(id);
    if (result.error) {
      setError(result.error);
      return;
    }
    onLoad(JSON.parse(result.scenario.inputs_json));
  };

  const handleDelete = async (id) => {
    await wasmModule.delete_scenario(id);
    refresh();
  };

  return (
    <div className="saved-scenarios">
      <div className="saved-scenarios-header">
        <h3>{t('saved.title')}</h3>
        {isSaving ? (
          <div className="save-form">
            <input
              value={nameInput}
              onChange={(e) => setNameInput(e.target.value)}
              placeholder={t('saved.namePlaceholder')}
              autoFocus
              onKeyDown={(e) => e.key === 'Enter' && handleSave()}
            />
            <button className="link-button" onClick={handleSave}>
              {t('saved.save')}
            </button>
            <button className="link-button" onClick={() => setIsSaving(false)}>
              {t('saved.cancel')}
            </button>
          </div>
        ) : (
          <button className="link-button" onClick={() => setIsSaving(true)}>
            {t('saved.saveAs')}
          </button>
        )}
      </div>

      {error && <div className="error">{error}</div>}

      {scenarios.length === 0 ? (
        <p className="saved-scenarios-empty">{t('saved.empty')}</p>
      ) : (
        <ul className="saved-scenarios-list">
          {scenarios.map((s) => (
            <li key={s.id}>
              <span className="scenario-name">{s.name}</span>
              <span className="scenario-date">{new Date(s.created_at).toLocaleDateString()}</span>
              <button className="link-button" onClick={() => handleLoad(s.id)}>
                {t('saved.load')}
              </button>
              <button className="link-button danger" onClick={() => handleDelete(s.id)}>
                {t('saved.delete')}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
