import type { Summary } from '../../api';

export default function BoardCard({ data }: {
  data: Summary['board'];
}) {
  const total = data.today + data.this_week + data.later;
  return <section className="card">
    <h2>BOARD <span>{total}</span></h2>
    <div className="board">
      {[data.today, data.this_week, data.later].map((count, i) =>
        <div key={i}>
          <strong>{count}</strong><span>{['오늘', '이번 주', '나중에'][i]}</span>
          <div className="track"><div style={{
            width: `${total ? count / total * 100 : 0}%`,
            background: ['var(--accent)', 'var(--success)', 'var(--warning)'][i]
          }}/></div>
        </div>
      )}
    </div>
    {!total && <p className="empty">예정된 업무가 없습니다</p>}
  </section>;
}
