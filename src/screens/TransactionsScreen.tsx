import { Component, createSignal, onMount, Show } from 'solid-js';
import { open } from '@tauri-apps/plugin-dialog';
import { commands, type ImportResult } from '../bindings';
import { useAppStore } from '../store/AppStore';
import TransactionTable from '../components/TransactionTable';

const TransactionsScreen: Component = () => {
  const [state, actions] = useAppStore();
  const [importResult, setImportResult] = createSignal<ImportResult | null>(null);
  const [importError, setImportError] = createSignal<string | null>(null);
  const [importing, setImporting] = createSignal(false);

  onMount(() => {
    void actions.loadTransactions();
  });

  const handleImport = async () => {
    setImportResult(null);
    setImportError(null);

    const selected = await open({
      multiple: true,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });

    if (selected === null || selected.length === 0) return;

    setImporting(true);
    try {
      const result = await commands.importTransactions(selected);
      if (result.status === 'ok') {
        setImportResult(result.data);
        await actions.loadTransactions();
      } else {
        setImportError(result.error);
      }
    } finally {
      setImporting(false);
    }
  };

  const handleDateFromChange = (value: string) => {
    actions.setDateRange(value || null, state.dateTo);
    void actions.loadTransactions();
  };

  const handleDateToChange = (value: string) => {
    actions.setDateRange(state.dateFrom, value || null);
    void actions.loadTransactions();
  };

  return (
    <main>
      <h2>Transactions</h2>

      <div>
        <button
          onClick={() => {
            void handleImport();
          }}
          disabled={importing()}
        >
          {importing() ? 'Importing…' : 'Import CSV'}
        </button>
      </div>

      <Show when={importResult()}>
        {(result) => <p>Imported {result().imported} transaction(s).</p>}
      </Show>

      <Show when={importError()}>{(error) => <p>Error: {error()}</p>}</Show>

      <div>
        <label>
          From:{' '}
          <input
            type="date"
            value={state.dateFrom ?? ''}
            onInput={(e) => {
              handleDateFromChange(e.currentTarget.value);
            }}
          />
        </label>
        {'  '}
        <label>
          To:{' '}
          <input
            type="date"
            value={state.dateTo ?? ''}
            onInput={(e) => {
              handleDateToChange(e.currentTarget.value);
            }}
          />
        </label>
      </div>

      <TransactionTable transactions={state.transactions} />
    </main>
  );
};

export default TransactionsScreen;
