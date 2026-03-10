import { Component } from 'solid-js';
import { For } from 'solid-js';

export type Tab = 'transactions' | 'filters' | 'report';

type NavProps = {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
};

const TABS: { id: Tab; label: string }[] = [
  { id: 'transactions', label: 'Transactions' },
  { id: 'filters', label: 'Filters' },
  { id: 'report', label: 'Report' },
];

const Nav: Component<NavProps> = (props) => {
  return (
    <nav>
      <For each={TABS}>
        {(tab) => (
          <button
            type="button"
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
