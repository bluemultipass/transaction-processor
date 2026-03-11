import { Component, createSignal, Show } from 'solid-js';

type FilterFormProps = {
  initialName?: string;
  initialPattern?: string;
  submitLabel: string;
  /** Return true on success (causes form to reset), false on failure. */
  onSubmit: (name: string, pattern: string) => Promise<boolean>;
  onCancel?: () => void;
};

const FilterForm: Component<FilterFormProps> = (props) => {
  const [name, setName] = createSignal(props.initialName ?? '');
  const [pattern, setPattern] = createSignal(props.initialPattern ?? '');
  const [submitting, setSubmitting] = createSignal(false);

  const handleSubmit = async (e: SubmitEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      const success = await props.onSubmit(name().trim(), pattern().trim());
      if (success) {
        setName('');
        setPattern('');
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form
      class="filter-form"
      onSubmit={(e) => {
        void handleSubmit(e);
      }}
    >
      <input
        class="input"
        type="text"
        placeholder="Name"
        value={name()}
        onInput={(e) => setName(e.currentTarget.value)}
        required
      />
      <input
        class="input"
        type="text"
        placeholder="Pattern (e.g. AMAZON)"
        value={pattern()}
        onInput={(e) => setPattern(e.currentTarget.value)}
        required
      />
      <div class="filter-form-actions">
        <button class="btn btn-primary btn-sm" type="submit" disabled={submitting()}>
          {submitting() ? 'Saving…' : props.submitLabel}
        </button>
        <Show when={props.onCancel !== undefined}>
          <button
            class="btn btn-secondary btn-sm"
            type="button"
            onClick={() => {
              props.onCancel?.();
            }}
          >
            Cancel
          </button>
        </Show>
      </div>
    </form>
  );
};

export default FilterForm;
