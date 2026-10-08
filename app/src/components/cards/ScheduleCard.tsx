import { Calendar } from 'lucide-react';
import { openItem, type Item } from '../../api';

export default function ScheduleCard({ items }: { items: Item[] }) {
  return <section className="card schedule">
    <h2>SCHEDULE <span>오늘 · 내일</span></h2>
    {items.map(item => <button className="schedule-row" key={`${item.source}:${item.id}`} onClick={() => void openItem(item)}>
      <Calendar size={18} />
      <time>
        {item.meta.all_day === true
          ? `${new Date(item.timestamp).toLocaleDateString('ko-KR', { month: 'short', day: 'numeric' })} · 종일`
          : new Date(item.timestamp).toLocaleString('ko-KR', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
      </time>
      <strong>{item.title}</strong>
      {item.meta.my_response === 'needsAction' && <span className="badge">응답 필요</span>}
    </button>)}
    {!items.length && <div className="empty">오늘·내일 일정이 없습니다</div>}
  </section>;
}
