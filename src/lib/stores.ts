import { writable } from 'svelte/store';

export const selectedDate = writable<Date>(new Date());

export const dateRange = writable<{
    start: Date;
    end: Date;
}>({
    start: new Date(),
    end: new Date()
}); 