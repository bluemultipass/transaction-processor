import { Component, createSignal, For, Show } from 'solid-js';
import { commands, type ReportRow } from '../bindings';
import { useAppStore } from '../store/AppStore';
import TransactionTable from '../components/TransactionTable';

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
    <main>
      <h2>Report</h2>

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

      <div>
        <button
          onClick={() => {
            void handleGenerateReport();
          }}
          disabled={generating()}
        >
          {generating() ? 'Generating…' : 'Generate Report'}
        </button>
      </div>

      <Show when={error()}>{(err) => <p>Error: {err()}</p>}</Show>

      <Show when={state.reportOutput}>
        {(output) => (
          <>
            <h3>Breakdown</h3>
            <For each={output().rows}>
              {(row: ReportRow) => (
                <details>
                  <summary>
                    {row.filter_name} — {row.last_date} — ${row.total_amount.toFixed(2)}
                  </summary>
                  <TransactionTable transactions={row.transactions} />
                </details>
              )}
            </For>

            <h3>Output</h3>
            <div>
              <textarea readonly rows={10} style={{ width: '100%', 'font-family': 'monospace' }}>
                {output().text}
              </textarea>
            </div>
            <div>
              <button onClick={handleCopy}>{copied() ? 'Copied!' : 'Copy to Clipboard'}</button>
            </div>
          </>
        )}
      </Show>
    </main>
  );
};

export default ReportScreen;
