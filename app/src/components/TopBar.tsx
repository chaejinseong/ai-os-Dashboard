import { Bell, Search } from 'lucide-react';
import { useEffect, useState, type RefObject } from 'react';
import { api, type Page, type Stats } from '../api';
export default function TopBar({ page, query, onQuery, onAlerts, missed, inputRef }: {
    page: Page;
    query: string;
    onQuery: (q: string) => void;
    onAlerts: () => void;
    missed: number;
    inputRef: RefObject<HTMLInputElement | null>;
}) {
    const [stats, setStats] = useState<Stats | null>(null);
    useEffect(() => {
        let active = true;
        let unlisten: (() => void) | undefined;
        const refresh = () => { void api.stats().then(value => { if (active) setStats(value); }).catch(() => {}); };
        refresh();
        void api.onUpdate(refresh).then(fn => { if (active) unlisten = fn; else fn(); });
        return () => { active = false; unlisten?.(); };
    }, []);
    return <div className="topbar"><span>워크스페이스 <span className="muted">/</span> <strong>{page}</strong></span>{stats?.cap_reached && <span className="ai-cap-banner">AI 판정 중지 (월 상한)</span>}<label className="search"><Search size={16}/><input ref={inputRef} value={query} onChange={e => onQuery(e.target.value)} placeholder="업무 검색" aria-label="업무 제목 검색"/><kbd>⌘K</kbd></label><button className="icon-button" aria-label={`알림 ${missed}건`} onClick={onAlerts}><Bell size={18}/><span className="badge">{missed}</span></button></div>;
}
