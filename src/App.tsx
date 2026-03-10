import { createSignal } from 'solid-js';
import { AppStoreProvider } from './store/AppStore';
import Nav, { type Tab } from './components/Nav';
import TransactionsScreen from './screens/TransactionsScreen';
import FiltersScreen from './screens/FiltersScreen';
import ReportScreen from './screens/ReportScreen';

function App() {
  const [activeTab, setActiveTab] = createSignal<Tab>('transactions');

  return (
    <AppStoreProvider>
      <Nav activeTab={activeTab()} onTabChange={setActiveTab} />
      {activeTab() === 'transactions' && <TransactionsScreen />}
      {activeTab() === 'filters' && <FiltersScreen />}
      {activeTab() === 'report' && <ReportScreen />}
    </AppStoreProvider>
  );
}

export default App;
