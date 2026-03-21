import { Component, createSignal, onMount, Show } from 'solid-js';
import { useAppStore } from '../store/AppStore';

const SettingsScreen: Component = () => {
  const [state, actions] = useAppStore();
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);

  onMount(() => {
    void actions.loadSplitCount();
  });

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
    </main>
  );
};

export default SettingsScreen;
