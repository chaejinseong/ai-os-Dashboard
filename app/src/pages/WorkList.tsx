import { openItem, type Item } from '../api';
export default function WorkList({ items, query }: {
    items: Item[];
    query: string;
}) { const shown = items.filter(i => i.title.toLocaleLowerCase().includes(query.toLocaleLowerCase())); return <><h1>업무 <small>{shown.length}건</small></h1><section className="card">{shown.map(i => <button className="item-row" key={`${i.source}:${i.id}`} onClick={() => void openItem(i)}><span className="eyebrow">{i.source}</span><strong>{i.title}</strong><span>{i.due ? new Date(i.due).toLocaleDateString('ko-KR') : '마감 없음'}</span></button>)}{!shown.length && <p className="empty large">{query ? '검색 결과가 없습니다' : '완료되지 않은 업무가 없습니다'}</p>}</section></>; }
