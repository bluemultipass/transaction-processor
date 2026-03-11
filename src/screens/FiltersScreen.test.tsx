import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, cleanup } from '@solidjs/testing-library';
import userEvent from '@testing-library/user-event';
import { AppStoreProvider } from '../store/AppStore';
import FiltersScreen from './FiltersScreen';

afterEach(cleanup);

const mocks = vi.hoisted(() => ({
  listFilters: vi.fn(),
  createFilter: vi.fn(),
  deleteFilter: vi.fn(),
  updateFilter: vi.fn(),
  listTransactions: vi.fn(),
  importTransactions: vi.fn(),
  generateReport: vi.fn(),
}));

vi.mock('../bindings', () => ({
  commands: mocks,
}));

const renderFiltersScreen = () =>
  render(() => (
    <AppStoreProvider>
      <FiltersScreen />
    </AppStoreProvider>
  ));

describe('FiltersScreen', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.listTransactions.mockResolvedValue({ status: 'ok', data: [] });
    mocks.listFilters.mockResolvedValue({ status: 'ok', data: [] });
  });

  it("shows 'No filters yet.' when filter list is empty", async () => {
    renderFiltersScreen();
    await waitFor(() => {
      expect(screen.getByText('No filters yet.')).toBeInTheDocument();
    });
  });

  it('adds a filter when the form is submitted', async () => {
    const newFilter = { id: 1, name: 'Amazon', pattern: 'AMAZON' };
    mocks.listFilters
      .mockResolvedValueOnce({ status: 'ok', data: [] })
      .mockResolvedValueOnce({ status: 'ok', data: [newFilter] });
    mocks.createFilter.mockResolvedValue({ status: 'ok', data: newFilter });

    const user = userEvent.setup();
    renderFiltersScreen();

    await waitFor(() => {
      expect(screen.getByText('No filters yet.')).toBeInTheDocument();
    });

    await user.type(screen.getByPlaceholderText('Name'), 'Amazon');
    await user.type(screen.getByPlaceholderText('Pattern (e.g. AMAZON)'), 'AMAZON');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    await waitFor(() => {
      expect(mocks.createFilter).toHaveBeenCalledWith('Amazon', 'AMAZON');
    });
    await waitFor(() => {
      expect(screen.getByText('Amazon')).toBeInTheDocument();
    });
    expect(screen.getByText('AMAZON')).toBeInTheDocument();
  });

  it('deletes a filter when Delete is clicked', async () => {
    const filter = { id: 1, name: 'Groceries', pattern: 'WHOLE FOODS' };
    mocks.listFilters
      .mockResolvedValueOnce({ status: 'ok', data: [filter] })
      .mockResolvedValueOnce({ status: 'ok', data: [] });
    mocks.deleteFilter.mockResolvedValue({ status: 'ok', data: null });

    const user = userEvent.setup();
    renderFiltersScreen();

    await waitFor(() => {
      expect(screen.getByText('Groceries')).toBeInTheDocument();
    });

    await user.click(screen.getByRole('button', { name: 'Delete' }));

    await waitFor(() => {
      expect(mocks.deleteFilter).toHaveBeenCalledWith(1);
    });
    await waitFor(() => {
      expect(screen.queryByText('Groceries')).not.toBeInTheDocument();
    });
  });
});
