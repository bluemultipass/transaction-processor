import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, cleanup } from '@solidjs/testing-library';
import userEvent from '@testing-library/user-event';
import { AppStoreProvider } from '../store/AppStore';
import ReportScreen from './ReportScreen';

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

const renderReportScreen = () =>
  render(() => (
    <AppStoreProvider>
      <ReportScreen />
    </AppStoreProvider>
  ));

describe('ReportScreen', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    mocks.listFilters.mockResolvedValue({ status: 'ok', data: [] });
    mocks.listTransactions.mockResolvedValue({ status: 'ok', data: [] });
  });

  it('renders Generate Report button', () => {
    renderReportScreen();
    expect(screen.getByRole('button', { name: 'Generate Report' })).toBeInTheDocument();
  });

  it('calls generateReport command when button is clicked', async () => {
    mocks.generateReport.mockResolvedValue({ status: 'ok', data: { rows: [], text: '' } });

    const user = userEvent.setup();
    renderReportScreen();

    await user.click(screen.getByRole('button', { name: 'Generate Report' }));

    await waitFor(() => {
      expect(mocks.generateReport).toHaveBeenCalledWith(null, null);
    });
  });

  it('displays report output after generation', async () => {
    const reportOutput = {
      rows: [
        {
          filter_name: 'Amazon',
          last_date: '2024-01-15',
          total_amount: 42.5,
          transactions: [],
        },
      ],
      text: 'Amazon\t2024-01-15\t$42.50',
    };
    mocks.generateReport.mockResolvedValue({ status: 'ok', data: reportOutput });

    const user = userEvent.setup();
    renderReportScreen();

    await user.click(screen.getByRole('button', { name: 'Generate Report' }));

    await waitFor(() => {
      expect(screen.getByText('Breakdown')).toBeInTheDocument();
    });
    // The summary row uses " — " as separator; the textarea uses tabs
    expect(screen.getByText(/Amazon.*—.*2024-01-15.*—.*\$42\.50/)).toBeInTheDocument();
    const textarea = screen.getByRole('textbox');
    expect(textarea).toHaveValue('Amazon\t2024-01-15\t$42.50');
  });

  it('shows error message when report generation fails', async () => {
    mocks.generateReport.mockResolvedValue({ status: 'error', error: 'Database error' });

    const user = userEvent.setup();
    renderReportScreen();

    await user.click(screen.getByRole('button', { name: 'Generate Report' }));

    await waitFor(() => {
      expect(screen.getByText('Error: Database error')).toBeInTheDocument();
    });
  });
});
