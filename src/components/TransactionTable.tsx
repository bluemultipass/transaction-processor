import { Component, For, Show } from 'solid-js';
import { type Transaction } from '../bindings';

type Props = {
  transactions: Transaction[];
};

const amountClass = (amount: number) =>
  amount > 0 ? 'amount-positive' : amount < 0 ? 'amount-negative' : 'amount-neutral';

const TransactionTable: Component<Props> = (props) => {
  return (
    <Show
      when={props.transactions.length > 0}
      fallback={<p class="empty-state">No transactions found.</p>}
    >
      <div class="table-wrapper">
        <table class="data-table">
          <thead>
            <tr>
              <th>Date</th>
              <th>Description</th>
              <th class="col-right">Amount</th>
              <th class="col-center">Accounted</th>
            </tr>
          </thead>
          <tbody>
            <For each={props.transactions}>
              {(tx) => (
                <tr>
                  <td class="col-muted">{tx.date}</td>
                  <td>{tx.description}</td>
                  <td class={`col-right ${amountClass(tx.amount)}`}>${tx.amount.toFixed(2)}</td>
                  <td class="col-center">
                    {tx.accounted ? (
                      <span class="badge-yes">Yes</span>
                    ) : (
                      <span class="badge-no">No</span>
                    )}
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
    </Show>
  );
};

export default TransactionTable;
