import type { Dashboard } from '../api';
import CompletionCard from '../components/cards/CompletionCard';
import BoardCard from '../components/cards/BoardCard';
import PipelineCard from '../components/cards/PipelineCard';
import MissedCard from '../components/cards/MissedCard';
import ScheduleCard from '../components/cards/ScheduleCard';

export default function Overview({ data, period, onPeriod, onSync, onAlerts, busy }: {
  data: Dashboard;
  period: number;
  onPeriod: (n: number) => void;
  onSync: () => void;
  onAlerts: () => void;
  busy: boolean;
}) {
  return <>
    <div className="page-heading">
      <div><div className="eyebrow">OVERVIEW</div><h1>전체 현황</h1></div>
      <div className="period">
        {[1, 7, 30].map(n => <button key={n} className={period === n ? 'selected' : ''}
          onClick={() => onPeriod(n)}>{n === 1 ? '오늘' : `${n}일`}</button>)}
      </div>
      <button className="round-badge" onClick={onAlerts}>
        {data.summary.missed_count}<small>건</small>
      </button>
    </div>
    <div className="card-grid">
      <CompletionCard data={data.summary.completion}/>
      <BoardCard data={data.summary.board}/>
      <PipelineCard data={data.summary.pipeline} onSync={onSync} busy={busy}/>
      <MissedCard items={data.missed} onAlerts={onAlerts}/>
      <ScheduleCard items={data.summary.schedule}/>
    </div>
  </>;
}
