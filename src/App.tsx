import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { Settings } from './Settings';
import { Analytics } from './Analytics';
import './index.css';
import { DailyStats, WindowActivity } from './types/activity';

type View = 'activities' | 'settings' | 'analytics';

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

function App() {
  const [activities, setActivities] = useState<WindowActivity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedDate, setSelectedDate] = useState(new Date().toISOString().split('T')[0]);
  const [currentView, setCurrentView] = useState<View>('activities');
  const [stats, setStats] = useState<DailyStats>({
    total_time: 0,
    productive_time: 0,
    goal_percentage: 0,
    idle_time: 0,
    top_applications: [],
    activities: []
  });

  const fetchActivities = async (date: string) => {
    try {
      setLoading(true);
      setError(null);
      const rfc3339Date = new Date(date).toISOString();
      const result = await invoke<DailyStats>('get_daily_stats', { date: rfc3339Date });
      
      console.log('📊 Daily Stats:', {
        total: formatDuration(result.total_time),
        productive: formatDuration(result.productive_time),
        idle: formatDuration(result.idle_time),
        apps: result.top_applications.length
      });

      // Log de cada aplicativo
      result.top_applications.forEach(app => {
        console.log(`📱 ${app.application}:`, {
          total: formatDuration(app.total_duration),
          idle: app.idle_duration ? formatDuration(app.idle_duration) : '0m',
          activities: app.activities.length,
          category: app.category?.name
        });

        // Log de atividades idle
        const idleActivities = app.activities.filter(a => a.is_idle);
        if (idleActivities.length > 0) {
          console.log(`  🔍 Idle Activities (${idleActivities.length}):`, 
            idleActivities.map(a => ({
              time: `${new Date(a.start_time).toLocaleTimeString()} - ${new Date(a.end_time).toLocaleTimeString()}`,
              title: a.title
            }))
          );
        }
      });

      setActivities(result.activities);
      setStats(result);
    } catch (err) {
      console.error('Error fetching activities:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch activities');
    } finally {
      setLoading(false);
    }
  };

  let refreshInterval: NodeJS.Timeout;

  useEffect(() => {
    fetchActivities(selectedDate);

    // Auto-refresh a cada 30 segundos se estiver visualizando o dia atual
    const isToday = selectedDate === new Date().toISOString().split('T')[0];

    if (isToday) {
      refreshInterval = setInterval(() => {
        console.log("🔄 Auto-refreshing activities...");
        fetchActivities(selectedDate);
      }, 30000); // 30 segundos
    }

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

    // Cleanup
    return () => {
      if (refreshInterval) {
        clearInterval(refreshInterval);
      }
    };
  }, [selectedDate]);

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--text-primary)]">
      <div className="max-w-6xl mx-auto p-6">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-2xl font-semibold">
            {currentView === 'activities' ? 'Activity Tracker' : 
             currentView === 'settings' ? 'Settings' : 'Analytics'}
          </h1>
          <div className="flex gap-4">
            <button
              onClick={() => setCurrentView('analytics')}
              className={`btn-secondary ${currentView === 'analytics' ? 'bg-[var(--accent)] text-white' : ''}`}
            >
              Analytics
            </button>
            <button
              onClick={() => setCurrentView(currentView === 'activities' ? 'settings' : 'activities')}
              className="btn-secondary"
            >
              {currentView === 'activities' ? 'Settings' : 'Back to Activities'}
            </button>
          </div>
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
                <div className="grid grid-cols-3 gap-6">
                  <div>
                    <p className="text-[var(--text-secondary)] mb-1">Total Time</p>
                    <p className="text-xl font-medium">
                      {formatDuration(stats.total_time)}
                    </p>
                  </div>
                  <div>
                    <p className="text-[var(--text-secondary)] mb-1">Productive Time</p>
                    <p className="text-xl font-medium text-[var(--success)]">
                      {formatDuration(stats.productive_time)} ({stats.goal_percentage}%)
                    </p>
                  </div>
                  <div>
                    <p className="text-[var(--text-secondary)] mb-1">Idle Time</p>
                    <p className="text-xl font-medium text-[var(--warning)]">
                      {formatDuration(stats.idle_time)}
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
                      <div className="flex items-center gap-4">
                        {app.idle_duration && app.idle_duration > 0 && (
                          <span className="text-[var(--warning)] text-sm">
                            Idle: {formatDuration(app.idle_duration)}
                          </span>
                        )}
                        <span className="text-[var(--text-secondary)]">
                          {formatDuration(app.total_duration)}
                        </span>
                      </div>
                    </div>

                    <div className="mt-2 text-sm text-[var(--text-secondary)]">
                      {new Date(app.activities[0].start_time).toLocaleTimeString()} -{' '}
                      {new Date(app.activities[app.activities.length - 1].end_time).toLocaleTimeString()}
                    </div>

                    <div className="mt-4 space-y-3">
                      {app.activities.map((activity) => (
                        <div 
                          key={`${activity.start_time}-${activity.title}`}
                          className={`pl-4 border-l border-[var(--border)] text-sm ${
                            activity.is_idle ? 'opacity-60' : ''
                          }`}
                        >
                          <div className="flex items-center gap-2">
                            <div className="text-[var(--text-primary)]">{activity.title}</div>
                            {activity.is_idle && (
                              <span className="text-[var(--warning)] text-xs">idle</span>
                            )}
                          </div>
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
        ) : currentView === 'settings' ? (
          <Settings />
        ) : (
          <Analytics />
        )}
      </div>
    </div>
  );
}

export default App;
