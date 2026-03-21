import { open, save } from '@tauri-apps/plugin-dialog';
import { Component, createSignal, onMount, Show } from 'solid-js';
import { commands } from '../bindings';
import { useAppStore } from '../store/AppStore';

const SettingsScreen: Component = () => {
  const [state, actions] = useAppStore();
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);
  const [exportStatus, setExportStatus] = createSignal<'idle' | 'busy' | 'done' | 'error'>('idle');
  const [exportError, setExportError] = createSignal<string | null>(null);
  const [importStatus, setImportStatus] = createSignal<'idle' | 'busy' | 'done' | 'error'>('idle');
  const [importMessage, setImportMessage] = createSignal<string | null>(null);

  onMount(() => {
    void actions.loadSplitCount();
  });

  const handleExport = async () => {
    setExportStatus('busy');
    setExportError(null);
    const path = await save({
      defaultPath: 'ledger-backup.json',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (!path) {
      setExportStatus('idle');
      return;
    }
    const result = await commands.exportDb(path);
    if (result.status === 'ok') {
      setExportStatus('done');
      setTimeout(() => setExportStatus('idle'), 4000);
    } else {
      setExportError(result.error);
      setExportStatus('error');
    }
  };

  const handleImport = async () => {
    if (!window.confirm('This will replace all your data. Continue?')) return;
    setImportStatus('busy');
    setImportMessage(null);
    const selected = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (!selected) {
      setImportStatus('idle');
      return;
    }
    const path = typeof selected === 'string' ? selected : selected[0];
    const result = await commands.importDb(path);
    if (result.status === 'ok') {
      const { transactions, filters } = result.data;
      setImportMessage(`Imported ${String(transactions)} transactions, ${String(filters)} filters`);
      setImportStatus('done');
      await Promise.all([
        actions.loadTransactions(),
        actions.loadFilters(),
        actions.loadSplitCount(),
      ]);
      setTimeout(() => {
        setImportStatus('idle');
        setImportMessage(null);
      }, 3000);
    } else {
      setImportMessage(result.error);
      setImportStatus('error');
    }
  };

  const handleChange = async (value: string) => {
    const n = parseInt(value, 10);
    if (isNaN(n) || n < 1) return;
    setError(null);
    await actions.updateSplitCount(n);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <main class="screen">
      <h1 class="screen-title">Settings</h1>
      <Show when={error()}>{(err) => <div class="msg-error">{err()}</div>}</Show>
      <div
        class="date-field"
        style={{ 'align-items': 'flex-start', 'flex-direction': 'column', gap: '6px' }}
      >
        <label style={{ 'font-size': '14px', 'font-weight': '500', color: 'var(--text)' }}>
          Split count
        </label>
        <input
          class="input-date"
          type="number"
          min="1"
          value={state.splitCount}
          onChange={(e) => {
            void handleChange(e.currentTarget.value);
          }}
        />
        <span style={{ 'font-size': '13px', color: 'var(--text-2)' }}>
          Number of people splitting expenses. Report amounts will be divided by this value.
        </span>
        <Show when={saved()}>
          <span class="msg-success" style={{ 'margin-bottom': '0' }}>
            Saved
          </span>
        </Show>
      </div>
      <hr style={{ border: 'none', 'border-top': '1px solid var(--border)', margin: '8px 0' }} />
      <div
        class="date-field"
        style={{ 'align-items': 'flex-start', 'flex-direction': 'column', gap: '6px' }}
      >
        <h2
          style={{ 'font-size': '14px', 'font-weight': '600', color: 'var(--text)', margin: '0' }}
        >
          Data
        </h2>
        <span style={{ 'font-size': '13px', color: 'var(--text-2)' }}>
          Export or import all transactions, filters, and settings as a JSON file.
        </span>
        <div style={{ display: 'flex', gap: '8px', 'margin-top': '4px' }}>
          <button
            class="btn btn-primary"
            onClick={() => void handleExport()}
            disabled={exportStatus() === 'busy'}
          >
            {exportStatus() === 'busy' ? 'Exporting…' : 'Export data'}
          </button>
          <button
            class="btn btn-primary"
            onClick={() => void handleImport()}
            disabled={importStatus() === 'busy'}
          >
            {importStatus() === 'busy' ? 'Importing…' : 'Import data'}
          </button>
        </div>
        <Show when={exportStatus() === 'done'}>
          <span class="msg-success" style={{ 'margin-bottom': '0' }}>
            Exported successfully
          </span>
        </Show>
        <Show when={exportStatus() === 'error'}>
          <span class="msg-error">{exportError()}</span>
        </Show>
        <Show when={importStatus() === 'done'}>
          <span class="msg-success" style={{ 'margin-bottom': '0' }}>
            {importMessage()}
          </span>
        </Show>
        <Show when={importStatus() === 'error'}>
          <span class="msg-error">{importMessage()}</span>
        </Show>
      </div>
    </main>
  );
};

export default SettingsScreen;
