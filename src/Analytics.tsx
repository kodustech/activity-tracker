import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { formatDuration } from './utils/timeUtils';
import { DailyStats } from './types/activity';

type TimePeriod = 'day' | 'week' | 'month';

interface CategoryAnalytics {
  name: string;
  color: string;
  totalTime: number;
  percentage: number;
}

export function Analytics() {
  const [period, setPeriod] = useState<TimePeriod>('day');
  const [selectedDate, setSelectedDate] = useState(new Date());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [stats, setStats] = useState<DailyStats | null>(null);
  const [categoryStats, setCategoryStats] = useState<CategoryAnalytics[]>([]);

  const fetchStats = async () => {
    try {
      setLoading(true);
      setError(null);
      
      let result: DailyStats;
      const date = selectedDate.toISOString();
      
      switch (period) {
        case 'week':
          result = await invoke<DailyStats>('get_weekly_stats', { date });
          break;
        case 'month':
          result = await invoke<DailyStats>('get_monthly_stats', { date });
          break;
        default:
          result = await invoke<DailyStats>('get_daily_stats', { date });
      }
      
      setStats(result);

      // Process category statistics
      const categoryMap = new Map<string, CategoryAnalytics>();
      let totalTime = 0;

      result.top_applications.forEach((app) => {
        if (app.category) {
          const existing = categoryMap.get(app.category.name) || {
            name: app.category.name,
            color: app.category.color,
            totalTime: 0,
            percentage: 0,
          };
          existing.totalTime += app.total_duration;
          totalTime += app.total_duration;
          categoryMap.set(app.category.name, existing);
        }
      });

      // Calculate percentages
      const categories = Array.from(categoryMap.values()).map((cat) => ({
        ...cat,
        percentage: totalTime > 0 ? (cat.totalTime / totalTime) * 100 : 0,
      }));

      // Sort by total time
      categories.sort((a, b) => b.totalTime - a.totalTime);
      setCategoryStats(categories);
    } catch (err) {
      console.error('Error fetching stats:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch stats');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStats();
  }, [selectedDate, period]);

  const handlePeriodChange = (newPeriod: TimePeriod) => {
    setPeriod(newPeriod);
    // TODO: Implement period-specific date handling
  };

  const handleDateChange = (date: Date) => {
    setSelectedDate(date);
  };

  if (loading) {
    return <div className="text-center py-8 text-[var(--text-secondary)]">Loading...</div>;
  }

  if (error) {
    return <div className="text-center py-8 text-[var(--error)]">{error}</div>;
  }

  return (
    <div className="max-w-6xl mx-auto p-6">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-2xl font-semibold">Time Analytics</h1>
      </div>

      <div className="card mb-6">
        <div className="flex items-center gap-4 mb-6">
          <div className="flex rounded-lg overflow-hidden border border-[var(--border)]">
            {(['day', 'week', 'month'] as TimePeriod[]).map((p) => (
              <button
                key={p}
                onClick={() => handlePeriodChange(p)}
                className={`px-4 py-2 text-sm font-medium ${
                  period === p
                    ? 'bg-[var(--accent)] text-white'
                    : 'hover:bg-[var(--surface-secondary)]'
                }`}
              >
                {p.charAt(0).toUpperCase() + p.slice(1)}
              </button>
            ))}
          </div>
          <input
            type="date"
            value={selectedDate.toISOString().split('T')[0]}
            onChange={(e) => handleDateChange(new Date(e.target.value))}
            className="px-3 py-2 rounded border border-[var(--border)] bg-transparent"
          />
        </div>

        <div className="space-y-6">
          {categoryStats.map((category) => (
            <div key={category.name} className="space-y-2">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: category.color }}
                  />
                  <span className="font-medium">{category.name}</span>
                </div>
                <div className="text-[var(--text-secondary)]">
                  {formatDuration(category.totalTime)}
                </div>
              </div>
              <div className="h-2 bg-[var(--surface-secondary)] rounded-full overflow-hidden">
                <div
                  className="h-full rounded-full transition-all duration-300"
                  style={{
                    width: `${category.percentage}%`,
                    backgroundColor: category.color,
                  }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      {stats && (
        <div className="grid grid-cols-3 gap-6">
          <div className="card">
            <h3 className="text-[var(--text-secondary)] mb-2">Total Time</h3>
            <p className="text-2xl font-semibold">{formatDuration(stats.total_time)}</p>
          </div>
          <div className="card">
            <h3 className="text-[var(--text-secondary)] mb-2">Productive Time</h3>
            <p className="text-2xl font-semibold text-[var(--success)]">
              {formatDuration(stats.productive_time)}
            </p>
          </div>
          <div className="card">
            <h3 className="text-[var(--text-secondary)] mb-2">Goal Progress</h3>
            <p className="text-2xl font-semibold">{stats.goal_percentage}%</p>
          </div>
        </div>
      )}
    </div>
  );
} 