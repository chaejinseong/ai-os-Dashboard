import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
export type Page = '개요' | '연결' | '업무' | '알림' | '설정';
export interface Item {
    id: string;
    source: string;
    kind: 'Email' | 'Event' | 'Task' | 'PullRequest' | 'Issue';
    title: string;
    subtitle: string | null;
    url: string;
    timestamp: string;
    due: string | null;
    done: boolean;
    stage: 'Requested' | 'InProgress' | 'Review' | 'Done' | null;
    meta: Record<string, unknown>;
}
export interface SourceStatus {
    source: string;
    connected: boolean;
    syncing: boolean;
    last_synced: string | null;
    error: string | null;
}
export interface MissedItem {
    item: Item;
    reason: string;
    since: string;
}
export interface Summary {
    completion: {
        done: number;
        left: number;
    };
    board: {
        today: number;
        this_week: number;
        later: number;
    };
    pipeline: {
        requested: number;
        in_progress: number;
        review: number;
        done: number;
        missed_by_stage: number[];
    };
    schedule: Item[];
    missed_count: number;
    open_work_count: number;
}
export interface Dashboard {
    summary: Summary;
    missed: MissedItem[];
    statuses: SourceStatus[];
}
export interface Settings {
    sync_minutes: number;
    mail_unanswered_days: number;
}
export const api = {
    dashboard: (periodDays: number) => invoke<Dashboard>('get_dashboard', { periodDays }),
    items: () => invoke<Item[]>('get_items', { filter: null }),
    sync: () => invoke<void>('sync_now'), connect: () => invoke<void>('connect_google'), disconnect: () => invoke<void>('disconnect_google'),
    saveClient: (clientId: string, clientSecret: string) => invoke<void>('save_google_client', { clientId, clientSecret }),
    hasClient: () => invoke<boolean>('has_google_client'), settings: () => invoke<Settings>('get_settings'), saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
    saveNotion: (token: string, database: string) => invoke<void>('save_notion', { token, database }),
    disconnectNotion: () => invoke<void>('disconnect_notion'),
    onUpdate: (handler: () => void) => listen('dashboard-updated', handler),
};
export async function openItem(item: Item) { try {
    const url = new URL(item.url);
    if (url.protocol !== 'https:')
        throw new Error('안전하지 않은 링크');
    await openUrl(url.toString());
}
catch {
    window.alert('링크를 열 수 없습니다');
} }
export const sources = [{ id: 'gmail', name: 'Gmail' }, { id: 'calendar', name: 'Calendar' }, { id: 'notion', name: 'Notion' }, { id: 'github', name: 'GitHub' }];
