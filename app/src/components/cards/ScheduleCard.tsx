import { Calendar } from 'lucide-react';
import { openItem, type Item } from '../../api';
export default function ScheduleCard({ items }: {
    items: Item[];
}) { return <section className="card schedule"><h2>SCHEDULE <span>오늘 · 내일</span></h2>{items.map(item => <button className="schedule-row" key={`${item.source}:${item.id}`} onClick={() => void openItem(item)}><Calendar size={18}/><time>{new Date(item.timestamp).toLocaleString('ko-KR', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}</time><strong>{item.title}</strong>{item.meta.response_status === 'needsAction' && <span className="badge">응답 필요</span>}</button>)}{!items.length && <div className="empty">Calendar를 연결하면 일정이 표시됩니다</div>}</section>; }
