import { useEffect, useState } from 'react';
import { api, type SourceStatus } from '../api';

type Card = 'google' | 'notion' | 'github';

export default function Connections({ statuses, onChange }: {
  statuses: SourceStatus[];
  onChange: () => void;
}) {
  const [hasClient, setHasClient] = useState(false);
  const [id, setId] = useState('');
  const [secret, setSecret] = useState('');
  const [busy, setBusy] = useState<Set<Card>>(() => new Set());
  const [error, setError] = useState('');
  const [notionToken, setNotionToken] = useState('');
  const [notionDatabase, setNotionDatabase] = useState('');
  const [githubToken, setGithubToken] = useState('');
  const [githubLogin, setGithubLogin] = useState<string | null>(null);
  const googleSources = ['gmail', 'calendar'].map(source => statuses.find(s => s.source === source));
  const notion = statuses.find(status => status.source === 'notion');
  const github = statuses.find(status => status.source === 'github');
  const connected = googleSources.some(s => s?.connected);
  const hasError = googleSources.some(s => s?.error);

  useEffect(() => {
    void api.hasClient().then(setHasClient).catch(e => setError(String(e)));
  }, []);
  useEffect(() => {
    if (github?.connected) {
      void api.githubLogin().then(setGithubLogin).catch(e => setError(String(e)));
    } else {
      setGithubLogin(null);
    }
  }, [github?.connected]);

  async function run(card: Card, action: () => Promise<void>) {
    setBusy(current => new Set(current).add(card));
    setError('');
    try { await action(); onChange(); }
    catch (e) { setError(String(e)); }
    finally {
      setBusy(current => {
        const next = new Set(current);
        next.delete(card);
        return next;
      });
    }
  }

  return <>
    <h1>연결</h1>
    {error && <p className="error-banner" role="alert">{error}</p>}
    <section className="card connection-card">
      <h2>GOOGLE <span className={`dot ${hasError ? 'error' : connected ? 'live' : ''}`} /></h2>
      <p>Gmail · Calendar 읽기 전용 연결</p>
      <p className="muted">Google Cloud의 Desktop app 클라이언트 정보를 입력하세요. Keychain에 저장됩니다.</p>
      {!connected && <form onSubmit={e => {
        e.preventDefault();
        void run('google', async () => {
          await api.saveClient(id, secret);
          setId(''); setSecret(''); setHasClient(true);
        });
      }}>
        <label>클라이언트 ID
          <input autoComplete="off" value={id} onChange={e => setId(e.target.value)} required disabled={busy.has('google')} />
        </label>
        <label>클라이언트 secret
          <input type="password" autoComplete="off" value={secret} onChange={e => setSecret(e.target.value)} required disabled={busy.has('google')} />
        </label>
        <button disabled={busy.has('google') || !id.trim() || !secret.trim()}>클라이언트 정보 저장</button>
        {hasClient && <small className="muted">저장된 클라이언트 정보가 있습니다</small>}
      </form>}
      {googleSources.map((status, index) => <div className="connection-details" key={index}>
        <div>{['Gmail', 'Calendar'][index]}
          <span>{status?.syncing ? '동기화 중' : status?.connected ? '연결됨' : '연결 안 됨'}</span>
        </div>
        <div>마지막 동기화
          <span>{status?.last_synced ? new Date(status.last_synced).toLocaleString('ko-KR') : '아직 없음'}</span>
        </div>
        {status?.error && <p className="danger">{status.error}</p>}
      </div>)}
      <div className="actions">
        <button className="primary" disabled={busy.has('google') || !hasClient} onClick={() => void run('google', api.connect)}>
          {busy.has('google') ? '처리 중…' : connected ? '다시 로그인' : 'Google 연결'}
        </button>
        {connected && <button disabled={busy.has('google')} onClick={() => void run('google', api.disconnect)}>연결 해제</button>}
      </div>
      {busy.has('google') && <p className="muted">로그인은 브라우저에서 진행하며 최대 5분 기다립니다.</p>}
    </section>
    <div className="card-grid placeholders">
      <section className="card connection-card">
        <h2>NOTION <span className={`dot ${notion?.error ? 'error' : notion?.connected ? 'live' : ''}`} /></h2>
        <p>프로젝트 플래너 · 읽기 전용 연결</p>
        {!notion?.connected && <form onSubmit={event => {
          event.preventDefault();
          void run('notion', async () => {
            await api.saveNotion(notionToken, notionDatabase);
            setNotionToken('');
            setNotionDatabase('');
          });
        }}>
          <label>내부 통합 토큰
            <input type="password" autoComplete="off" value={notionToken}
              onChange={event => setNotionToken(event.target.value)} required disabled={busy.has('notion')} />
          </label>
          <label>데이터베이스 URL 또는 ID
            <input autoComplete="off" value={notionDatabase}
              onChange={event => setNotionDatabase(event.target.value)} required disabled={busy.has('notion')} />
          </label>
          <button disabled={busy.has('notion') || !notionToken.trim() || !notionDatabase.trim()}>Notion 연결</button>
        </form>}
        <div className="connection-details">
          <div>상태 <span>{notion?.syncing ? '동기화 중' : notion?.connected ? '연결됨' : '연결 안 됨'}</span></div>
          <div>마지막 동기화 <span>{notion?.last_synced ? new Date(notion.last_synced).toLocaleString('ko-KR') : '아직 없음'}</span></div>
        </div>
        {notion?.error && <p className="danger">{notion.error}</p>}
        {notion?.connected && <div className="actions">
          <button disabled={busy.has('notion')} onClick={() => void run('notion', api.disconnectNotion)}>연결 해제</button>
        </div>}
      </section>
      <section className="card connection-card">
        <h2>GITHUB <span className={`dot ${github?.error ? 'error' : github?.connected ? 'live' : ''}`} /></h2>
        <p>리뷰 요청 · 내 PR · 할당 이슈 읽기 전용 연결</p>
        <p className="muted">Fine-grained token이 접근할 수 있는 저장소만 검색됩니다. Metadata, Pull requests, Issues 읽기 권한을 선택하세요.</p>
        {!github?.connected && <form onSubmit={event => {
          event.preventDefault();
          void run('github', async () => {
            await api.saveGithub(githubToken);
            setGithubToken('');
            setGithubLogin(await api.githubLogin());
          });
        }}>
          <label>개인 액세스 토큰
            <input type="password" autoComplete="off" value={githubToken}
              onChange={event => setGithubToken(event.target.value)} required disabled={busy.has('github')} />
          </label>
          <button disabled={busy.has('github') || !githubToken.trim()}>GitHub 연결</button>
        </form>}
        <div className="connection-details">
          <div>상태 <span>{github?.syncing ? '동기화 중' : github?.connected ? `연결됨 · ${githubLogin ?? '로그인 확인 중'}` : '연결 안 됨'}</span></div>
          <div>마지막 동기화 <span>{github?.last_synced ? new Date(github.last_synced).toLocaleString('ko-KR') : '아직 없음'}</span></div>
        </div>
        {github?.error && <p className="danger">{github.error}</p>}
        {github?.connected && <div className="actions">
          <button disabled={busy.has('github')} onClick={() => void run('github', async () => { await api.disconnectGithub(); setGithubLogin(null); })}>연결 해제</button>
        </div>}
      </section>
    </div>
  </>;
}
