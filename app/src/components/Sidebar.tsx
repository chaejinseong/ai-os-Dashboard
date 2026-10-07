import { LayoutDashboard, Link, ListTodo, Bell, Settings, Mail, Calendar, BookOpen, Github } from 'lucide-react';
import { sources, type Page, type SourceStatus } from '../api';
const icons = [Mail, Calendar, BookOpen, Github];
export default function Sidebar({ page, onPage, statuses, work, missed }: {
    page: Page;
    onPage: (p: Page) => void;
    statuses: SourceStatus[];
    work: number;
    missed: number;
}) {
    return <aside className="sidebar"><div className="eyebrow">워크스페이스</div><nav>{([{ name: '개요', icon: LayoutDashboard }, { name: '연결', icon: Link }, { name: '업무', icon: ListTodo, count: work }, { name: '알림', icon: Bell, count: missed }] as const).map(({ name, icon: Icon, ...rest }) => <button key={name} className={page === name ? 'selected' : ''} onClick={() => onPage(name)}><Icon size={18}/>{name}{'count' in rest && <span className={name === '알림' ? 'badge' : 'count'}>{rest.count}</span>}</button>)}</nav><div className="eyebrow sources-label">SOURCES</div>{sources.map((s, index) => { const status = statuses.find(v => v.source === s.id); const Icon = icons[index]; return <button className="source" key={s.id} onClick={() => onPage('연결')}><Icon size={16}/>{s.name}<span className={`dot ${status?.error ? 'error' : status?.connected ? 'live' : ''}`}/></button>; })}<button className="account" onClick={() => onPage('설정')}><span className="avatar">나</span>내 계정<Settings size={17}/></button></aside>;
}
