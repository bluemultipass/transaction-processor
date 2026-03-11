import { describe, it, expect, afterEach } from 'vitest';
import { render, screen, cleanup } from '@solidjs/testing-library';
import TransactionTable from './TransactionTable';
import type { Transaction } from '../bindings';

afterEach(cleanup);

const makeTransaction = (overrides: Partial<Transaction> = {}): Transaction => ({
  id: 1,
  date: '2024-01-15',
  description: 'Test Store',
  amount: 10.99,
  accounted: false,
  ...overrides,
});

describe('TransactionTable', () => {
  it('shows fallback when no transactions', () => {
    render(() => <TransactionTable transactions={[]} />);
    expect(screen.getByText('No transactions found.')).toBeInTheDocument();
  });

  it('renders transaction rows correctly', () => {
    const transactions = [
      makeTransaction({
        id: 1,
        date: '2024-01-15',
        description: 'AMAZON',
        amount: 19.99,
        accounted: false,
      }),
      makeTransaction({
        id: 2,
        date: '2024-01-16',
        description: 'WALMART',
        amount: 5.0,
        accounted: true,
      }),
    ];
    render(() => <TransactionTable transactions={transactions} />);

    expect(screen.getByText('AMAZON')).toBeInTheDocument();
    expect(screen.getByText('$19.99')).toBeInTheDocument();
    expect(screen.getByText('WALMART')).toBeInTheDocument();
    expect(screen.getByText('$5.00')).toBeInTheDocument();
    expect(screen.getByText('2024-01-15')).toBeInTheDocument();
    expect(screen.getByText('2024-01-16')).toBeInTheDocument();
    expect(screen.getByText('Yes')).toBeInTheDocument();
    expect(screen.getAllByText('No').length).toBeGreaterThan(0);
  });

  it('renders table headers', () => {
    render(() => <TransactionTable transactions={[makeTransaction()]} />);
    expect(screen.getAllByText('Date').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Description').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Amount').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Accounted').length).toBeGreaterThan(0);
  });
});
