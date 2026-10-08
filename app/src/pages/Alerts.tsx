import { useState } from 'react';
import { api, openItem, type MissedItem } from '../api';

export default function Alerts({ items }: { items: MissedItem[] }) {
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
  return <><h1>알림 <small>{visible.length}건</small></h1>
    <section className="card missed-card">
      {visible.map(m => <div className={`item-row source-${m.item.source}`} key={`${m.item.source}:${m.item.id}`}>
        <button className="alert-open" onClick={() => void openItem(m.item)}>
          <span className="eyebrow">{m.item.source}</span>
          <span><strong>{m.item.title}</strong><small className="muted">{m.item.subtitle}</small>
            {m.summary && <small className="ai-summary">{m.summary}</small>}</span>
          <span className="danger">{m.reason}</span>
        </button>
        <div className="vote-actions">
          <button aria-label={`${m.item.title} 도움 됨`} onClick={() => void vote(m, 1)}>👍</button>
          <button aria-label={`${m.item.title} 도움 안 됨`} onClick={() => void vote(m, -1)}>👎</button>
        </div>
      </div>)}
      {!visible.length && <p className="empty large">놓친 업무가 없습니다</p>}
    </section>
  </>;
}
