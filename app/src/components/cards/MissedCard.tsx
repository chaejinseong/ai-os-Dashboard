import { Check, Clock, Mail } from 'lucide-react';
import { openItem, type MissedItem } from '../../api';
export default function MissedCard({ items, onAlerts }: {
    items: MissedItem[];
    onAlerts: () => void;
}) { return <section className="card missed-card"><h2>MISSED <span className="badge">{items.length}</span><button className="icon-button" aria-label="모든 알림 보기" onClick={onAlerts}><Check size={17}/></button></h2>{items.slice(0, 5).map(m => <button key={`${m.item.source}:${m.item.id}`} className={`missed-row source-${m.item.source}`} onClick={() => void openItem(m.item)}><Mail size={17}/><span className="truncate">{m.item.title}</span><small><Clock size={12}/>{Math.max(0, Math.floor((Date.now() - Date.parse(m.since)) / 86400000))}일</small></button>)}{!items.length && <div className="empty large">놓친 업무가 없습니다</div>}</section>; }
