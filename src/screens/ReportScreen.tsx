import { Component, createSignal, For, Show } from 'solid-js';
import { commands, type ReportRow } from '../bindings';
import { useAppStore } from '../store/AppStore';
import TransactionTable from '../components/TransactionTable';

const amountClass = (amount: number) =>
  amount > 0 ? 'amount-positive' : amount < 0 ? 'amount-negative' : 'amount-neutral';

const ReportScreen: Component = () => {
  const [state, actions] = useAppStore();
  const [generating, setGenerating] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [copied, setCopied] = createSignal(false);

  const handleGenerateReport = async () => {
    setError(null);
    setGenerating(true);
    try {
      const result = await commands.generateReport(state.dateFrom, state.dateTo);
      if (result.status === 'ok') {
        actions.setReportOutput(result.data);
      } else {
        setError(result.error);
      }
    } finally {
      setGenerating(false);
    }
  };

  const handleCopy = () => {
    const text = state.reportOutput?.text;
    if (!text) return;
    void navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  const handleDateFromChange = (value: string) => {
    actions.setDateRange(value || null, state.dateTo);
  };

  const handleDateToChange = (value: string) => {
    actions.setDateRange(state.dateFrom, value || null);
  };

  return (
    <main class="screen">
      <h2 class="screen-title">Report</h2>

      <div class="report-controls">
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
        <button
          class="btn btn-primary"
          onClick={() => {
            void handleGenerateReport();
          }}
          disabled={generating()}
        >
          {generating() ? 'Generating…' : 'Generate Report'}
        </button>
      </div>

      <Show when={error()}>{(err) => <div class="msg-error">✕ {err()}</div>}</Show>

      <Show when={state.reportOutput}>
        {(output) => (
          <>
            <p class="section-title">Breakdown</p>
            <div class="report-breakdown">
              <For each={output().rows}>
                {(row: ReportRow) => (
                  <details class="report-details">
                    <summary>
                      <span class="summary-name">{row.filter_name}</span>
                      <span class="summary-date">{row.last_date}</span>
                      <span class={`summary-amount ${amountClass(row.total_amount)}`}>
                        ${row.total_amount.toFixed(2)}
                      </span>
                      <span class="summary-chevron">›</span>
                    </summary>
                    <div class="detail-content">
                      <TransactionTable transactions={row.transactions} />
                    </div>
                  </details>
                )}
              </For>
            </div>

            <p class="section-title">Output</p>
            <div class="report-output-section">
              <textarea class="report-textarea" readonly rows={10}>
                {output().text}
              </textarea>
              <div class="report-output-actions">
                <button
                  class={`btn ${copied() ? 'btn-success' : 'btn-secondary'}`}
                  onClick={handleCopy}
                >
                  {copied() ? '✓ Copied' : 'Copy to Clipboard'}
                </button>
              </div>
            </div>
          </>
        )}
      </Show>
    </main>
  );
};

export default ReportScreen;
