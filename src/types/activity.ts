export interface WindowActivity {
    id: number;
    title: string;
    application: string;
    start_time: string; // ISO string
    end_time: string; // ISO string
    is_browser: boolean;
    url?: string;
}

export interface Category {
    id: string;
    name: string;
    color: string;
    is_productive: boolean;
}

export interface ApplicationStats {
    application: string;
    total_duration: number; // em segundos
    activities: WindowActivity[];
    category?: Category;
}

export interface DailyStats {
    total_time: number; // em segundos
    productive_time: number; // em segundos
    goal_percentage: number;
    top_applications: ApplicationStats[];
    activities: WindowActivity[];
} 