import { useState } from 'react';
import { Check, Clock, Mail } from 'lucide-react';
import { api, openItem, type MissedItem } from '../../api';

export default function MissedCard({ items, onAlerts }: {
  items: MissedItem[];
  onAlerts: () => void;
}) {
  const [hidden, setHidden] = useState<Set<string>>(new Set());
  const visible = items.filter(m => !hidden.has(`${m.item.source}:${m.item.id}`));
  async function vote(m: MissedItem, value: 1 | -1) {
    const id = `${m.item.source}:${m.item.id}`;
    if (value === -1) setHidden(previous => new Set(previous).add(id));
    try { await api.setFeedback(m.item.source, m.item.id, value); }
    catch (error) {
      if (value === -1) setHidden(previous => { const next = new Set(previous); next.delete(id); return next; });
      window.alert(String(error));
    }
  }
  return <section className="card missed-card">
    <h2>MISSED <span className="badge">{visible.length}</span>
      <button className="icon-button" aria-label="모든 알림 보기" onClick={onAlerts}><Check size={17}/></button>
    </h2>
    {visible.slice(0, 5).map(m => <div key={`${m.item.source}:${m.item.id}`} className={`missed-row source-${m.item.source}`}>
      <button className="missed-open" onClick={() => void openItem(m.item)}>
        <Mail size={17}/><span className="truncate">{m.item.title}{m.summary && <small className="ai-summary">{m.summary}</small>}</span>
        <small><Clock size={12}/>{Math.max(0, Math.floor((Date.now() - Date.parse(m.since)) / 86400000))}일</small>
      </button>
      <div className="vote-actions">
        <button aria-label={`${m.item.title} 도움 됨`} onClick={() => void vote(m, 1)}>👍</button>
        <button aria-label={`${m.item.title} 도움 안 됨`} onClick={() => void vote(m, -1)}>👎</button>
      </div>
    </div>)}
    {!visible.length && <div className="empty large">놓친 업무가 없습니다</div>}
  </section>;
}
