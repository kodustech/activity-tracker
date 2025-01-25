import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { Settings } from './Settings';
import './index.css';

interface Activity {
  id: number;
  application: string;
  title: string;
  start_time: string;
  end_time: string;
  is_browser: boolean;
  day: string;
}

interface DailyStats {
  total_time: number;
  productive_time: number;
  top_applications: Array<{
    application: string;
    total_duration: number;
    activities: Activity[];
    category?: Category;
  }>;
  activities: Activity[];
}

interface Category {
  id: string;
  name: string;
  color: string;
  is_productive: boolean;
}

interface ApplicationStats {
  application: string;
  total_duration: number;
  activities: Activity[];
  category?: Category;
}

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

function App() {
  const [activities, setActivities] = useState<Activity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedDate, setSelectedDate] = useState(new Date().toISOString().split('T')[0]);
  const [currentView, setCurrentView] = useState<'activities' | 'settings'>('activities');
  const [stats, setStats] = useState<DailyStats>({
    total_time: 0,
    productive_time: 0,
    top_applications: [],
    activities: []
  });

  const fetchActivities = async (date: string) => {
    try {
      setLoading(true);
      setError(null);
      const rfc3339Date = new Date(date).toISOString();
      const result = await invoke<DailyStats>('get_daily_stats', { date: rfc3339Date });
      setActivities(result.activities);
      setStats(result);
    } catch (err) {
      console.error('Error fetching activities:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch activities');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchActivities(selectedDate);

    const setupListeners = async () => {
      await listen('show_stats', () => {
        setCurrentView('activities');
      });

      await listen('show_range', () => {
        setCurrentView('activities');
      });

      await listen('show_settings', () => {
        setCurrentView('settings');
      });
    };

    setupListeners();
  }, [selectedDate]);

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--text-primary)]">
      <div className="max-w-6xl mx-auto p-6">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-2xl font-semibold">
            {currentView === 'activities' ? 'Activity Tracker' : 'Settings'}
          </h1>
          <button
            onClick={() => setCurrentView(currentView === 'activities' ? 'settings' : 'activities')}
            className="btn-secondary"
          >
            {currentView === 'activities' ? 'Settings' : 'Back to Activities'}
          </button>
        </div>

        {currentView === 'activities' ? (
          <div className="space-y-6">
            <input
              type="date"
              value={selectedDate}
              onChange={(e) => setSelectedDate(e.target.value)}
              className="w-full"
            />
            
            {!loading && !error && (
              <div className="card space-y-4">
                <div className="flex justify-between items-center">
                  <h2 className="text-lg font-medium">Daily Summary</h2>
                  <span className="text-[var(--text-secondary)]">
                    {new Date(selectedDate).toLocaleDateString()}
                  </span>
                </div>
                <div className="grid grid-cols-2 gap-6">
                  <div>
                    <p className="text-[var(--text-secondary)] mb-1">Total Time</p>
                    <p className="text-xl font-medium">
                      {formatDuration(stats.total_time)}
                    </p>
                  </div>
                  <div>
                    <p className="text-[var(--text-secondary)] mb-1">Productive Time</p>
                    <p className="text-xl font-medium text-[var(--success)]">
                      {formatDuration(stats.productive_time)}
                    </p>
                  </div>
                </div>
              </div>
            )}
            
            {loading ? (
              <div className="text-center py-8 text-[var(--text-secondary)]">Loading...</div>
            ) : error ? (
              <div className="text-center py-8 text-[var(--error)]">{error}</div>
            ) : activities.length === 0 ? (
              <div className="text-center py-8 text-[var(--text-secondary)]">No activities found for this date.</div>
            ) : (
              <div className="space-y-4">
                {stats.top_applications.map((app) => (
                  <div key={app.application} className="card">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center space-x-3">
                        <span className="font-medium">
                          {app.application}
                        </span>
                        {app.category && (
                          <span
                            className="badge"
                            style={{ 
                              backgroundColor: app.category.color,
                              color: 'white',
                              opacity: 0.9
                            }}
                          >
                            {app.category.name}
                          </span>
                        )}
                      </div>
                      <span className="text-[var(--text-secondary)]">
                        {formatDuration(app.total_duration)}
                      </span>
                    </div>
                    
                    <div className="mt-2 text-sm text-[var(--text-secondary)]">
                      {new Date(app.activities[0].start_time).toLocaleTimeString()} -{' '}
                      {new Date(app.activities[app.activities.length - 1].end_time).toLocaleTimeString()}
                    </div>

                    <div className="mt-4 space-y-3">
                      {app.activities.map((activity) => (
                        <div 
                          key={activity.id}
                          className="pl-4 border-l border-[var(--border)] text-sm"
                        >
                          <div className="text-[var(--text-primary)]">{activity.title}</div>
                          <div className="text-[var(--text-secondary)]">
                            {new Date(activity.start_time).toLocaleTimeString()} -{' '}
                            {new Date(activity.end_time).toLocaleTimeString()}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ) : (
          <Settings />
        )}
      </div>
    </div>
  );
}

export default App;
