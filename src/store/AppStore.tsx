import { createContext, useContext, ParentComponent } from 'solid-js';
import { createStore } from 'solid-js/store';
import { commands, type Filter, type ReportOutput, type Transaction } from '../bindings';

type AppState = {
  transactions: Transaction[];
  filters: Filter[];
  dateFrom: string | null;
  dateTo: string | null;
  reportOutput: ReportOutput | null;
  splitCount: number;
};

type AppActions = {
  loadTransactions: () => Promise<void>;
  loadFilters: () => Promise<void>;
  setDateRange: (dateFrom: string | null, dateTo: string | null) => void;
  setReportOutput: (output: ReportOutput | null) => void;
  loadSplitCount: () => Promise<void>;
  updateSplitCount: (n: number) => Promise<void>;
};

type AppStoreContextValue = [AppState, AppActions];

const AppStoreContext = createContext<AppStoreContextValue | undefined>(undefined);

export const AppStoreProvider: ParentComponent = (props) => {
  const [state, setState] = createStore<AppState>({
    transactions: [],
    filters: [],
    dateFrom: null,
    dateTo: null,
    reportOutput: null,
    splitCount: 2,
  });

  const actions: AppActions = {
    loadTransactions: async () => {
      const result = await commands.listTransactions(state.dateFrom, state.dateTo);
      if (result.status === 'ok') setState('transactions', result.data);
    },

    loadFilters: async () => {
      const result = await commands.listFilters();
      if (result.status === 'ok') setState('filters', result.data);
    },

    setDateRange: (dateFrom, dateTo) => {
      setState('dateFrom', dateFrom);
      setState('dateTo', dateTo);
    },

    setReportOutput: (output) => {
      setState('reportOutput', output);
    },

    loadSplitCount: async () => {
      const result = await commands.getSplitCount();
      if (result.status === 'ok') setState('splitCount', result.data);
    },

    updateSplitCount: async (n: number) => {
      const result = await commands.setSplitCount(n);
      if (result.status === 'ok') setState('splitCount', n);
    },
  };

  return (
    <AppStoreContext.Provider value={[state, actions]}>{props.children}</AppStoreContext.Provider>
  );
};

export function useAppStore(): AppStoreContextValue {
  const ctx = useContext(AppStoreContext);
  if (ctx === undefined) {
    throw new Error('useAppStore must be used within AppStoreProvider');
  }
  return ctx;
}
