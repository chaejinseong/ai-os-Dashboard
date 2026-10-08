import { useEffect, useState } from 'react';
import { api, type Settings as SettingsData, type Stats } from '../api';

export default function Settings({ settings, onSaved }: {
  settings: SettingsData;
  onSaved: () => void;
}) {
  const [form, setForm] = useState(settings);
  const [key, setKey] = useState('');
  const [stats, setStats] = useState<Stats | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  useEffect(() => setForm(settings), [settings]);
  useEffect(() => { void api.stats().then(setStats).catch(e => setMessage(String(e))); }, []);

  async function saveSettings(event: React.FormEvent) {
    event.preventDefault();
    setBusy(true); setMessage('');
    try {
      await api.saveSettings(form);
      setStats(await api.stats());
      setMessage('저장했습니다');
      onSaved();
    } catch (error) { setMessage(String(error)); }
    finally { setBusy(false); }
  }
  async function saveKey(event: React.FormEvent) {
    event.preventDefault();
    setBusy(true); setMessage('');
    try {
      await api.saveAnthropicKey(key);
      setKey('');
      setStats(await api.stats());
      setMessage('API 키를 Keychain에 저장했습니다');
    } catch (error) { setMessage(String(error)); }
    finally { setBusy(false); }
  }

  return <>
    <h1>설정</h1>
    <form className="card settings" onSubmit={saveSettings}>
      <label>동기화 주기 (분)<input type="number" min={1} max={1440} required value={form.sync_minutes}
        onChange={event => setForm({ ...form, sync_minutes: Number(event.target.value) })}/></label>
      <label>메일 미응답 기준 (일)<input type="number" min={1} max={365} required value={form.mail_unanswered_days}
        onChange={event => setForm({ ...form, mail_unanswered_days: Number(event.target.value) })}/></label>
      <label>AI 모델<input required value={form.ai_model}
        onChange={event => setForm({ ...form, ai_model: event.target.value })}/></label>
      <p className="muted">모델을 바꾸면 아래 입력·출력 가격도 직접 바꾸세요.</p>
      <label>월 비용 상한 (USD)<input type="number" min={0} step="any" required value={form.ai_monthly_cap_usd}
        onChange={event => setForm({ ...form, ai_monthly_cap_usd: Number(event.target.value) })}/></label>
      <label>입력 100만 토큰 가격 (USD)<input type="number" min={0} step="any" required value={form.ai_price_in_per_mtok}
        onChange={event => setForm({ ...form, ai_price_in_per_mtok: Number(event.target.value) })}/></label>
      <label>출력 100만 토큰 가격 (USD)<input type="number" min={0} step="any" required value={form.ai_price_out_per_mtok}
        onChange={event => setForm({ ...form, ai_price_out_per_mtok: Number(event.target.value) })}/></label>
      <button className="primary" disabled={busy}>설정 저장</button>
    </form>
    <form className="card settings" onSubmit={saveKey}>
      <h2>Anthropic API 키</h2>
      <p className="muted">{stats?.key_last4 ? `저장된 키: ••••${stats.key_last4}` : '저장된 키가 없습니다'}</p>
      <label>API 키<input type="password" autoComplete="off" required value={key}
        onChange={event => setKey(event.target.value)}/></label>
      <button disabled={busy || !key.trim()}>API 키 저장</button>
    </form>
    <section className="card settings ai-stats">
      <h2>AI 사용 현황</h2>
      <p>이번 달 비용 <strong>${stats?.month_cost_usd.toFixed(4) ?? '0.0000'}</strong> / ${stats?.cap_usd.toFixed(2) ?? form.ai_monthly_cap_usd.toFixed(2)}</p>
      <progress max={stats && stats.cap_usd > 0 ? stats.cap_usd : 1}
        value={stats ? Math.min(stats.month_cost_usd, stats.cap_usd || 1) : 0} />
      <p className="muted">입력 {stats?.month_input_tokens ?? 0} · 출력 {stats?.month_output_tokens ?? 0} 토큰</p>
      <p>최근 2주 👍 비율 <strong>{stats && stats.feedback_14d.up + stats.feedback_14d.down > 0
        ? `${Math.round(stats.feedback_14d.ratio * 100)}%` : '데이터 없음'}</strong>
        <span className="muted"> (👍 {stats?.feedback_14d.up ?? 0} / 👎 {stats?.feedback_14d.down ?? 0})</span></p>
    </section>
    <p role="status">{message}</p>
  </>;
}
