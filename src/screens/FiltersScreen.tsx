import { Component, For, createSignal, onMount, Show } from 'solid-js';
import { commands } from '../bindings';
import { useAppStore } from '../store/AppStore';
import FilterForm from '../components/FilterForm';

const FiltersScreen: Component = () => {
  const [state, actions] = useAppStore();
  const [editingId, setEditingId] = createSignal<number | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  onMount(() => {
    void actions.loadFilters();
  });

  const handleCreate = async (name: string, pattern: string): Promise<boolean> => {
    setError(null);
    const result = await commands.createFilter(name, pattern);
    if (result.status === 'ok') {
      await actions.loadFilters();
      return true;
    } else {
      setError(result.error);
      return false;
    }
  };

  const handleUpdate = async (id: number, name: string, pattern: string): Promise<boolean> => {
    setError(null);
    const result = await commands.updateFilter(id, name, pattern);
    if (result.status === 'ok') {
      setEditingId(null);
      await actions.loadFilters();
      return true;
    } else {
      setError(result.error);
      return false;
    }
  };

  const handleDelete = async (id: number) => {
    setError(null);
    const result = await commands.deleteFilter(id);
    if (result.status === 'ok') {
      await actions.loadFilters();
    } else {
      setError(result.error);
    }
  };

  return (
    <main class="screen">
      <h2 class="screen-title">Filters</h2>

      <Show when={error()}>{(err) => <div class="msg-error">✕ {err()}</div>}</Show>

      <Show
        when={state.filters.length > 0}
        fallback={<p class="empty-state">No filters defined yet.</p>}
      >
        <div class="table-wrapper">
          <table class="data-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Pattern</th>
                <th class="col-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              <For each={state.filters}>
                {(filter) => (
                  <Show
                    when={editingId() === filter.id}
                    fallback={
                      <tr>
                        <td>{filter.name}</td>
                        <td class="col-muted">{filter.pattern}</td>
                        <td class="col-right">
                          <div class="action-buttons">
                            <button
                              class="btn btn-ghost btn-sm"
                              onClick={() => setEditingId(filter.id)}
                            >
                              Edit
                            </button>
                            <button
                              class="btn btn-danger btn-sm"
                              onClick={() => {
                                void handleDelete(filter.id);
                              }}
                            >
                              Delete
                            </button>
                          </div>
                        </td>
                      </tr>
                    }
                  >
                    <tr>
                      <td colSpan={3}>
                        <FilterForm
                          initialName={filter.name}
                          initialPattern={filter.pattern}
                          submitLabel="Save"
                          onSubmit={(name, pattern) => handleUpdate(filter.id, name, pattern)}
                          onCancel={() => setEditingId(null)}
                        />
                      </td>
                    </tr>
                  </Show>
                )}
              </For>
            </tbody>
          </table>
        </div>
      </Show>

      <p class="section-title">Add Filter</p>
      <div class="add-filter-section">
        <FilterForm submitLabel="Add" onSubmit={handleCreate} />
      </div>
    </main>
  );
};

export default FiltersScreen;
