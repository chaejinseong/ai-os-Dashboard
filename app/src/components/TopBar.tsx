import { Bell, Search } from 'lucide-react';
import type { RefObject } from 'react';
import type { Page } from '../api';
export default function TopBar({ page, query, onQuery, onAlerts, missed, inputRef }: {
    page: Page;
    query: string;
    onQuery: (q: string) => void;
    onAlerts: () => void;
    missed: number;
    inputRef: RefObject<HTMLInputElement | null>;
}) {
    return <div className="topbar"><span>워크스페이스 <span className="muted">/</span> <strong>{page}</strong></span><label className="search"><Search size={16}/><input ref={inputRef} value={query} onChange={e => onQuery(e.target.value)} placeholder="업무 검색" aria-label="업무 제목 검색"/><kbd>⌘K</kbd></label><button className="icon-button" aria-label={`알림 ${missed}건`} onClick={onAlerts}><Bell size={18}/><span className="badge">{missed}</span></button></div>;
}
