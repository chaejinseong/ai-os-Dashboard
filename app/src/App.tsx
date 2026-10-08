import { useCallback, useEffect, useRef, useState } from 'react';
import { api, type Dashboard, type Item, type Page, type Settings as SettingsData } from './api';
import Sidebar from './components/Sidebar';
import TitleBar from './components/TitleBar';
import TopBar from './components/TopBar';
import Overview from './pages/Overview';
import Connections from './pages/Connections';
import WorkList from './pages/WorkList';
import Alerts from './pages/Alerts';
import Settings from './pages/Settings';
import './styles/tokens.css';
import './styles/app.css';

export default function App() {
  const [page, setPage] = useState<Page>('개요');
  const [period, setPeriod] = useState(7);
  const [data, setData] = useState<Dashboard | null>(null);
  const [items, setItems] = useState<Item[]>([]);
  const [settings, setSettings] = useState<SettingsData>({
    sync_minutes: 15, mail_unanswered_days: 2,
    ai_model: 'claude-haiku-4-5-20251001', ai_monthly_cap_usd: 5,
    ai_price_in_per_mtok: 1, ai_price_out_per_mtok: 5,
  });
  const [query, setQuery] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [online, setOnline] = useState(navigator.onLine);
  const [now, setNow] = useState(Date.now());
  const search = useRef<HTMLInputElement>(null);
  const generation = useRef(0);

  const refresh = useCallback(async () => {
    const request = ++generation.current;
    try {
      const [next, list, config] = await Promise.all([
        api.dashboard(period), api.items(), api.settings(),
      ]);
      if (request !== generation.current) return;
      setData(next);
      setItems(list);
      setSettings(config);
      setError('');
    } catch (e) {
      if (request === generation.current) setError(String(e));
    }
  }, [period]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void api.onUpdate(() => void refresh()).then(fn => {
      if (disposed) fn();
      else {
        unlisten = fn;
        void refresh();
      }
    }).catch(e => setError(String(e)));
    void refresh();
    return () => {
      disposed = true;
      generation.current++;
      unlisten?.();
    };
  }, [refresh]);

  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setPage('업무');
        search.current?.focus();
      }
    };
    const network = () => setOnline(navigator.onLine);
    window.addEventListener('keydown', key);
    window.addEventListener('online', network);
    window.addEventListener('offline', network);
    const timer = window.setInterval(() => setNow(Date.now()), 30000);
    return () => {
      window.removeEventListener('keydown', key);
      window.removeEventListener('online', network);
      window.removeEventListener('offline', network);
      clearInterval(timer);
    };
  }, []);

  async function sync() {
    setBusy(true);
    try {
      await api.sync();
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="app">
      <TitleBar statuses={data?.statuses ?? []} minutes={settings.sync_minutes} online={online} now={now} />
      <div className="shell">
        <Sidebar page={page} onPage={setPage} statuses={data?.statuses ?? []}
          work={data?.summary.open_work_count ?? 0} missed={data?.summary.missed_count ?? 0} />
        <div className="workspace">
          <TopBar page={page} query={query} inputRef={search}
            onQuery={q => { setQuery(q); setPage('업무'); }}
            missed={data?.summary.missed_count ?? 0} onAlerts={() => setPage('알림')} />
          <main>
            {error && <div className="error-banner" role="alert">
              {error}<button onClick={() => void refresh()}>다시 시도</button>
            </div>}
            {!data && !error && <p className="empty large">캐시를 불러오는 중…</p>}
            {page === '개요' && data && <Overview data={data} period={period} onPeriod={setPeriod}
              onAlerts={() => setPage('알림')} onSync={() => void sync()}
              busy={busy || data.statuses.some(s => s.syncing)} />}
            {page === '연결' && <Connections statuses={data?.statuses ?? []} onChange={() => void refresh()} />}
            {page === '업무' && <WorkList items={items} query={query} />}
            {page === '알림' && <Alerts items={data?.missed ?? []} />}
            {page === '설정' && <Settings settings={settings} onSaved={() => void refresh()} />}
          </main>
        </div>
      </div>
    </div>
  );
}
