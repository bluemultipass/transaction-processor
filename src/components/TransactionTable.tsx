import { Component, For, Show } from 'solid-js';
import { type Transaction } from '../bindings';

type Props = {
  transactions: Transaction[];
};

const TransactionTable: Component<Props> = (props) => {
  return (
    <Show when={props.transactions.length > 0} fallback={<p>No transactions found.</p>}>
      <table>
        <thead>
          <tr>
            <th>Date</th>
            <th>Description</th>
            <th>Amount</th>
            <th>Accounted</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.transactions}>
            {(tx) => (
              <tr>
                <td>{tx.date}</td>
                <td>{tx.description}</td>
                <td>${tx.amount.toFixed(2)}</td>
                <td>{tx.accounted ? 'Yes' : 'No'}</td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </Show>
  );
};

export default TransactionTable;
