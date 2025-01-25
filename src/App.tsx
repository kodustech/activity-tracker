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
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900 p-4">
      <div className="max-w-4xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
            {currentView === 'activities' ? 'Activity Tracker' : 'Settings'}
          </h1>
          <button
            onClick={() => setCurrentView(currentView === 'activities' ? 'settings' : 'activities')}
            className="px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700"
          >
            {currentView === 'activities' ? 'Settings' : 'Back to Activities'}
          </button>
        </div>

        {currentView === 'activities' ? (
          <div className="space-y-4">
            <input
              type="date"
              value={selectedDate}
              onChange={(e) => setSelectedDate(e.target.value)}
              className="w-full p-2 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
            />
            
            {!loading && !error && (
              <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4 space-y-2">
                <div className="flex justify-between items-center">
                  <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Daily Summary</h2>
                  <span className="text-sm text-gray-500 dark:text-gray-400">
                    {new Date(selectedDate).toLocaleDateString()}
                  </span>
                </div>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <p className="text-sm text-gray-500 dark:text-gray-400">Total Time</p>
                    <p className="text-lg font-semibold text-gray-900 dark:text-white">
                      {formatDuration(stats.total_time)}
                    </p>
                  </div>
                  <div>
                    <p className="text-sm text-gray-500 dark:text-gray-400">Productive Time</p>
                    <p className="text-lg font-semibold text-green-600 dark:text-green-400">
                      {formatDuration(stats.productive_time)}
                    </p>
                  </div>
                </div>
              </div>
            )}
            
            {loading ? (
              <div className="text-center py-4">Loading...</div>
            ) : error ? (
              <div className="text-red-500 text-center py-4">{error}</div>
            ) : activities.length === 0 ? (
              <div className="text-center py-4">No activities found for this date.</div>
            ) : (
              <div className="space-y-4">
                {stats.top_applications.map((app) => (
                  <div key={app.application} className="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center space-x-2">
                        <span className="font-semibold text-gray-900 dark:text-white">
                          {app.application}
                        </span>
                        {app.category && (
                          <span
                            className="px-2 py-1 text-xs rounded text-white"
                            style={{ backgroundColor: app.category.color }}
                          >
                            {app.category.name}
                          </span>
                        )}
                      </div>
                      <span className="text-gray-600 dark:text-gray-300">
                        {formatDuration(app.total_duration)}
                      </span>
                    </div>
                    
                    <div className="mt-2 text-sm text-gray-500 dark:text-gray-400">
                      {new Date(app.activities[0].start_time).toLocaleTimeString()} -{' '}
                      {new Date(app.activities[app.activities.length - 1].end_time).toLocaleTimeString()}
                    </div>

                    <div className="mt-3 space-y-2">
                      {app.activities.map((activity) => (
                        <div 
                          key={activity.id}
                          className="text-sm pl-4 py-1 border-l-2 border-gray-200 dark:border-gray-700"
                        >
                          <div className="text-gray-700 dark:text-gray-300">{activity.title}</div>
                          <div className="text-gray-500 dark:text-gray-400">
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
