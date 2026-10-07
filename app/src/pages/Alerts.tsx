import { openItem, type MissedItem } from '../api';
export default function Alerts({ items }: {
    items: MissedItem[];
}) { return <><h1>알림 <small>{items.length}건</small></h1><section className="card missed-card">{items.map(m => <button className={`item-row source-${m.item.source}`} key={`${m.item.source}:${m.item.id}`} onClick={() => void openItem(m.item)}><span className="eyebrow">{m.item.source}</span><div><strong>{m.item.title}</strong><p className="muted">{m.item.subtitle}</p></div><span className="danger">{m.reason}</span></button>)}{!items.length && <p className="empty large">놓친 업무가 없습니다</p>}</section></>; }
