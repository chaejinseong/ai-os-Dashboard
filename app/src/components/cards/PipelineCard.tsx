import { RefreshCw, TriangleAlert } from 'lucide-react';
import type { Summary } from '../../api';

export default function PipelineCard({ data, onSync, busy }: {
  data: Summary['pipeline'];
  onSync: () => void;
  busy: boolean;
}) {
  const missed = data.missed_by_stage.reduce((a, b) => a + b, 0);
  const max = Math.max(...data.missed_by_stage);
  return <section className="card">
    <h2>PIPELINE <span className="danger"><TriangleAlert size={13}/> {missed}</span>
      <button className="icon-button" onClick={onSync} disabled={busy} aria-label="지금 동기화">
        <RefreshCw size={16} className={busy ? 'spin' : ''}/>
      </button>
    </h2>
    {[data.requested, data.in_progress, data.review, data.done].map((count, i) =>
      <div key={i} className={`pipeline-row ${max > 0 && data.missed_by_stage[i] === max ? 'highlight' : ''}`}>
        <span className={`stage stage-${i}`}/>
        <span>{['요청', '진행', '검토', '완료'][i]}</span>
        <strong>{count}</strong>
      </div>
    )}
  </section>;
}
