import { Component, createSignal, For, onMount, Show } from 'solid-js';
import { open } from '@tauri-apps/plugin-dialog';
import { commands, type ImportResult, type PendingTransaction } from '../bindings';
import { useAppStore } from '../store/AppStore';
import TransactionTable from '../components/TransactionTable';

const TransactionsScreen: Component = () => {
  const [state, actions] = useAppStore();
  const [importResult, setImportResult] = createSignal<ImportResult | null>(null);
  const [importError, setImportError] = createSignal<string | null>(null);
  const [importing, setImporting] = createSignal(false);
  const [pendingTransactions, setPendingTransactions] = createSignal<PendingTransaction[]>([]);
  const [skippedIndices, setSkippedIndices] = createSignal<Set<number>>(new Set());
  const [confirming, setConfirming] = createSignal(false);

  onMount(() => {
    void actions.loadTransactions();
  });

  const doConfirmImport = async (transactions: PendingTransaction[]) => {
    const result = await commands.confirmImport(transactions);
    if (result.status === 'ok') {
      setImportResult(result.data);
      setPendingTransactions([]);
      setSkippedIndices(new Set<number>());
      await actions.loadTransactions();
    } else {
      setImportError(result.error);
    }
  };

  const handleImport = async () => {
    setImportResult(null);
    setImportError(null);
    setPendingTransactions([]);
    setSkippedIndices(new Set<number>());

    const selected = await open({
      multiple: true,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });

    if (selected === null || selected.length === 0) return;

    setImporting(true);
    try {
      const result = await commands.previewImport(selected);
      if (result.status === 'ok') {
        const preview = result.data.transactions;
        const hasDuplicates = preview.some((t) => t.is_possible_duplicate);
        if (!hasDuplicates) {
          // No duplicates — insert immediately without asking the user.
          await doConfirmImport(preview);
        } else {
          // Show review UI: pre-check duplicate rows as "skip".
          setPendingTransactions(preview);
          setSkippedIndices(
            new Set<number>(preview.flatMap((t, i) => (t.is_possible_duplicate ? [i] : []))),
          );
        }
      } else {
        setImportError(result.error);
      }
    } finally {
      setImporting(false);
    }
  };

  const toggleSkip = (index: number) => {
    setSkippedIndices((prev) => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  };

  const handleConfirm = async () => {
    const toInsert = pendingTransactions().filter((_, i) => !skippedIndices().has(i));
    setConfirming(true);
    try {
      await doConfirmImport(toInsert);
    } finally {
      setConfirming(false);
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

  const includeCount = () => pendingTransactions().length - skippedIndices().size;

  return (
    <main>
      <h2>Transactions</h2>

      <div>
        <button
          onClick={() => {
            void handleImport();
          }}
          disabled={importing() || pendingTransactions().length > 0}
        >
          {importing() ? 'Importing…' : 'Import CSV'}
        </button>
      </div>

      <Show when={importResult()}>
        {(result) => <p>Imported {result().imported} transaction(s).</p>}
      </Show>

      <Show when={importError()}>{(error) => <p>Error: {error()}</p>}</Show>

      <Show when={pendingTransactions().length > 0}>
        <div>
          <h3>Review before importing</h3>
          <p>
            {includeCount()} of {pendingTransactions().length} transaction(s) will be imported.
            <Show when={skippedIndices().size > 0}>
              {' '}
              {skippedIndices().size} marked as duplicate and will be skipped.
            </Show>
          </p>
          <table>
            <thead>
              <tr>
                <th>Skip</th>
                <th>Date</th>
                <th>Description</th>
                <th>Amount</th>
                <th>Note</th>
              </tr>
            </thead>
            <tbody>
              <For each={pendingTransactions()}>
                {(tx, i) => (
                  <tr>
                    <td>
                      <input
                        type="checkbox"
                        checked={skippedIndices().has(i())}
                        onChange={() => {
                          toggleSkip(i());
                        }}
                      />
                    </td>
                    <td>{tx.date}</td>
                    <td>{tx.description}</td>
                    <td>${tx.amount.toFixed(2)}</td>
                    <td>{tx.is_possible_duplicate ? 'Possible duplicate' : ''}</td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
          <button
            onClick={() => {
              void handleConfirm();
            }}
            disabled={confirming() || includeCount() === 0}
          >
            {confirming() ? 'Importing…' : `Import ${includeCount()} transaction(s)`}
          </button>
          {'  '}
          <button
            onClick={() => {
              setPendingTransactions([]);
            }}
          >
            Cancel
          </button>
        </div>
      </Show>

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
