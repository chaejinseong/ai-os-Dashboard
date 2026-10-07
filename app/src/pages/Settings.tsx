import { useState, useEffect } from 'react';
import { api, type Settings as SettingsData } from '../api';
export default function Settings({ settings, onSaved }: {
    settings: SettingsData;
    onSaved: () => void;
}) { const [form, setForm] = useState(settings), [busy, setBusy] = useState(false), [message, setMessage] = useState(''); useEffect(() => setForm(settings), [settings.sync_minutes, settings.mail_unanswered_days]); return <><h1>설정</h1><form className="card settings" onSubmit={e => { e.preventDefault(); setBusy(true); setMessage(''); void api.saveSettings(form).then(() => { setMessage('저장했습니다'); onSaved(); }).catch(e => setMessage(String(e))).finally(() => setBusy(false)); }}><label>동기화 주기 (분)<input type="number" min={1} max={1440} required value={form.sync_minutes} onChange={e => setForm({ ...form, sync_minutes: Number(e.target.value) })}/></label><label>메일 미응답 기준 (일)<input type="number" min={1} max={365} required value={form.mail_unanswered_days} onChange={e => setForm({ ...form, mail_unanswered_days: Number(e.target.value) })}/></label><button className="primary" disabled={busy}>저장</button><p role="status">{message}</p></form></>; }
