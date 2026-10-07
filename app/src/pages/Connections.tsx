import { useEffect, useState } from 'react';
import { api, type SourceStatus } from '../api';
export default function Connections({ statuses, onChange }: {
    statuses: SourceStatus[];
    onChange: () => void;
}) {
    const [hasClient, setHasClient] = useState(false), [id, setId] = useState(''), [secret, setSecret] = useState(''), [busy, setBusy] = useState(false), [error, setError] = useState('');
    const status = statuses.find(s => s.source === 'gmail');
    useEffect(() => { void api.hasClient().then(setHasClient).catch(e => setError(String(e))); }, []);
    async function run(action: () => Promise<void>) { setBusy(true); setError(''); try {
        await action();
        onChange();
    }
    catch (e) {
        setError(String(e));
    }
    finally {
        setBusy(false);
    } }
    return <><h1>연결</h1>{error && <p className="error-banner" role="alert">{error}</p>}<section className="card connection-card"><h2>GOOGLE <span className={`dot ${status?.error ? 'error' : status?.connected ? 'live' : ''}`}/></h2><p>Gmail · Calendar 읽기 전용 연결</p><p className="muted">Google Cloud의 Desktop app 클라이언트 정보를 입력하세요. Keychain에 저장됩니다.</p>{!status?.connected && <form onSubmit={e => { e.preventDefault(); void run(async () => { await api.saveClient(id, secret); setId(''); setSecret(''); setHasClient(true); }); }}><label>클라이언트 ID<input autoComplete="off" value={id} onChange={e => setId(e.target.value)} required disabled={busy}/></label><label>클라이언트 secret<input type="password" autoComplete="off" value={secret} onChange={e => setSecret(e.target.value)} required disabled={busy}/></label><button disabled={busy || !id.trim() || !secret.trim()}>클라이언트 정보 저장</button>{hasClient && <small className="muted">저장된 클라이언트 정보가 있습니다</small>}</form>}<div className="connection-details"><div>Gmail <span>{status?.connected ? '연결됨' : '연결 안 됨'}</span></div><div>Calendar <span>준비 중 · 읽기 권한은 함께 요청</span></div><div>마지막 동기화 <span>{status?.last_synced ? new Date(status.last_synced).toLocaleString('ko-KR') : '아직 없음'}</span></div></div>{status?.error && <p className="danger">{status.error}</p>}<div className="actions"><button className="primary" disabled={busy || !hasClient} onClick={() => void run(api.connect)}>{busy ? '처리 중…' : status?.connected ? '다시 로그인' : 'Google 연결'}</button>{status?.connected && <button disabled={busy} onClick={() => void run(api.disconnect)}>연결 해제</button>}</div>{busy && <p className="muted">로그인은 브라우저에서 진행하며 최대 5분 기다립니다.</p>}</section><div className="card-grid placeholders">{['Notion', 'GitHub'].map(name => <section className="card" key={name}><h2>{name}</h2><p className="empty">준비 중</p></section>)}</div></>;
}
