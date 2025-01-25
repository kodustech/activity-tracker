import { listen } from '@tauri-apps/api/event';
import { get } from 'svelte/store';
import { selectedDate, dateRange } from './stores';

export async function setupEventListeners() {
    // Escuta eventos para mostrar estatísticas de um dia específico
    await listen('show_stats', (event) => {
        const date = event.payload as string;
        selectedDate.set(new Date(date));
    });

    // Escuta eventos para mostrar estatísticas de um período
    await listen('show_range', (event) => {
        const range = event.payload as { start: string; end: string };
        dateRange.set({
            start: new Date(range.start),
            end: new Date(range.end)
        });
    });
} 