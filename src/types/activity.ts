export interface WindowActivity {
    title: string;
    application: string;
    start_time: string; // ISO string
    end_time: string; // ISO string
    is_browser: boolean;
    url?: string;
}

export interface ApplicationStats {
    application: string;
    total_duration: number; // em segundos
    activities: WindowActivity[];
}

export interface DailyStats {
    total_time: number; // em segundos
    productive_time: number; // em segundos
    top_applications: ApplicationStats[];
    activities: WindowActivity[];
} 