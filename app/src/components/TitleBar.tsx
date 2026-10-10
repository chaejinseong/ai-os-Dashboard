import type { SourceStatus } from '../api';

export default function TitleBar({ statuses, minutes, online, now }: {
  statuses: SourceStatus[];
  minutes: number;
  online: boolean;
  now: number;
}) {
  const connected = statuses.filter(s => s.connected);
  const state = !online ? 'offline'
    : statuses.some(s => s.error) ? 'error'
      : statuses.some(s => s.syncing) ? 'syncing'
        : connected.length > 0 && connected.every(s => s.last_synced && now - Date.parse(s.last_synced) <= minutes * 60000)
          ? 'live' : 'offline';
  return <header className="titlebar">
    <strong><span className="brand">✳</span> AI 운영체제</strong>
    <span className={`connection-state ${state}`}><span className={`dot ${state}`}/>{state}</span>
  </header>;
}
