import { Component } from 'solid-js';
import { For } from 'solid-js';

export type Tab = 'transactions' | 'filters' | 'report' | 'settings';

type NavProps = {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
};

const TABS: { id: Tab; label: string }[] = [
  { id: 'transactions', label: 'Transactions' },
  { id: 'filters', label: 'Filters' },
  { id: 'report', label: 'Report' },
  { id: 'settings', label: 'Settings' },
];

const Nav: Component<NavProps> = (props) => {
  return (
    <nav class="app-nav">
      <span class="nav-logo">
        <span class="nav-logo-dot" />
        Ledger
      </span>
      <For each={TABS}>
        {(tab) => (
          <button
            type="button"
            class="nav-tab"
            aria-current={props.activeTab === tab.id ? 'page' : undefined}
            onClick={() => {
              props.onTabChange(tab.id);
            }}
          >
            {tab.label}
          </button>
        )}
      </For>
    </nav>
  );
};

export default Nav;
