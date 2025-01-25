import { invoke } from '@tauri-apps/api/tauri';
import { WindowActivity, DailyStats } from '../types/activity';

export async function getActivitiesBetween(
    startDate: Date,
    endDate: Date
): Promise<WindowActivity[]> {
    return invoke('get_activities', {
        range: {
            start: startDate.toISOString(),
            end: endDate.toISOString(),
        },
    });
}

export async function getDailyStats(date: Date): Promise<DailyStats> {
    return invoke('get_daily_stats', {
        date: date.toISOString(),
    });
}

// Lista de aplicativos considerados não produtivos
const unproductiveApps = new Set([
    'Finder',
    'System Settings',
    'System Preferences',
    'Notification Center',
    'Dock',
    'Spotlight',
    'Menu Bar',
]);

function isUnproductiveApp(appName: string): boolean {
    return unproductiveApps.has(appName);
} 