import { Component, createSignal, For, onMount, Show } from 'solid-js';
import { open } from '@tauri-apps/plugin-dialog';
import { commands, type ImportResult, type PendingTransaction } from '../bindings';
import { useAppStore } from '../store/AppStore';
import TransactionTable from '../components/TransactionTable';

const amountClass = (amount: number) =>
  amount > 0 ? 'amount-positive' : amount < 0 ? 'amount-negative' : 'amount-neutral';

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
    <main class="screen">
      <h2 class="screen-title">Transactions</h2>

      <div class="toolbar">
        <button
          class="btn btn-primary"
          onClick={() => {
            void handleImport();
          }}
          disabled={importing() || pendingTransactions().length > 0}
        >
          {importing() ? 'Importing…' : 'Import CSV'}
        </button>
      </div>

      <Show when={importResult()}>
        {(result) => (
          <div class="msg-success">
            ✓ Imported {result().imported} transaction{result().imported !== 1 ? 's' : ''}
          </div>
        )}
      </Show>

      <Show when={importError()}>{(error) => <div class="msg-error">✕ {error()}</div>}</Show>

      <Show when={pendingTransactions().length > 0}>
        <div class="review-panel">
          <div class="review-panel-header">
            <span class="review-panel-title">Review before importing</span>
          </div>
          <p class="review-summary">
            <strong>{includeCount()}</strong> of {pendingTransactions().length} transaction
            {pendingTransactions().length !== 1 ? 's' : ''} will be imported.
            <Show when={skippedIndices().size > 0}>
              {' '}
              <strong>{skippedIndices().size}</strong> marked as duplicate and will be skipped.
            </Show>
          </p>
          <div class="table-wrapper">
            <table class="data-table">
              <thead>
                <tr>
                  <th class="col-center">Skip</th>
                  <th>Date</th>
                  <th>Description</th>
                  <th class="col-right">Amount</th>
                  <th class="col-center">Note</th>
                </tr>
              </thead>
              <tbody>
                <For each={pendingTransactions()}>
                  {(tx, i) => (
                    <tr classList={{ 'duplicate-row': tx.is_possible_duplicate }}>
                      <td class="col-center">
                        <input
                          class="skip-checkbox"
                          type="checkbox"
                          checked={skippedIndices().has(i())}
                          onChange={() => {
                            toggleSkip(i());
                          }}
                        />
                      </td>
                      <td class="col-muted">{tx.date}</td>
                      <td>{tx.description}</td>
                      <td class={`col-right ${amountClass(tx.amount)}`}>${tx.amount.toFixed(2)}</td>
                      <td class="col-center">
                        {tx.is_possible_duplicate ? (
                          <span class="duplicate-badge">Duplicate</span>
                        ) : (
                          ''
                        )}
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
          <div class="review-actions">
            <button
              class="btn btn-primary"
              onClick={() => {
                void handleConfirm();
              }}
              disabled={confirming() || includeCount() === 0}
            >
              {confirming()
                ? 'Importing…'
                : `Import ${includeCount().toString()} transaction${includeCount() !== 1 ? 's' : ''}`}
            </button>
            <button
              class="btn btn-secondary"
              onClick={() => {
                setPendingTransactions([]);
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      </Show>

      <div class="date-range-bar">
        <label class="date-field">
          From
          <input
            class="input-date"
            type="date"
            value={state.dateFrom ?? ''}
            onInput={(e) => {
              handleDateFromChange(e.currentTarget.value);
            }}
          />
        </label>
        <label class="date-field">
          To
          <input
            class="input-date"
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
