import React, { useEffect, useState } from 'react';
import { DailyStats } from '../types/activity';
import { getDailyStats } from '../services/activityService';
import { formatDuration } from '../utils/timeUtils';

export function Dashboard() {
    const [stats, setStats] = useState<DailyStats | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        async function loadStats() {
            try {
                const data = await getDailyStats(new Date());
                setStats(data);
            } catch (err) {
                setError(err instanceof Error ? err.message : 'Failed to load stats');
            } finally {
                setLoading(false);
            }
        }

        loadStats();
    }, []);

    if (loading) return <div className="loading">Loading...</div>;
    if (error) return <div className="error">{error}</div>;
    if (!stats) return null;

    return (
        <div className="dashboard">
            <header className="dashboard-header">
                <h1>Today's Activity</h1>
                <div className="stats-summary">
                    <div className="stat-card">
                        <h3>Total Time</h3>
                        <p>{formatDuration(stats.total_time)}</p>
                    </div>
                    <div className="stat-card">
                        <h3>Productive Time</h3>
                        <p>{formatDuration(stats.productive_time)}</p>
                    </div>
                </div>
            </header>

            <section className="top-applications">
                <h2>Top Applications</h2>
                <div className="app-list">
                    {stats.top_applications.map((app) => (
                        <div key={app.application} className="app-card">
                            <div className="app-info">
                                <h3>{app.application}</h3>
                                <p>{formatDuration(app.total_duration)}</p>
                            </div>
                            <div className="app-bar">
                                <div
                                    className="app-bar-fill"
                                    style={{
                                        width: `${(app.total_duration / stats.total_time) * 100}%`,
                                    }}
                                />
                            </div>
                        </div>
                    ))}
                </div>
            </section>

            <section className="activity-timeline">
                <h2>Timeline</h2>
                <div className="timeline">
                    {stats.activities.map((activity, index) => (
                        <div key={index} className="timeline-item">
                            <div className="time">
                                {new Date(activity.start_time).toLocaleTimeString()}
                            </div>
                            <div className="activity-info">
                                <h4>{activity.application}</h4>
                                <p>{activity.title}</p>
                            </div>
                        </div>
                    ))}
                </div>
            </section>
        </div>
    );
} 