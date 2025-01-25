export interface WindowActivity {
    id: number;
    title: string;
    application: string;
    start_time: string; // ISO string
    end_time: string; // ISO string
    is_browser: boolean;
    url?: string;
    is_idle: boolean;
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
    idle_duration?: number; // tempo total em idle
}

export interface DailyStats {
    total_time: number; // em segundos
    productive_time: number; // em segundos
    goal_percentage: number;
    idle_time: number; // tempo total em idle
    top_applications: ApplicationStats[];
    activities: WindowActivity[];
} 