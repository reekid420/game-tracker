import { useEffect, useState } from "react";
import { type GameStats, getGameStats } from "../hooks/useBackend";

/** Statistics dashboard backed by aggregated game metrics. */
export function Stats() {
  const [stats, setStats] = useState<GameStats | null>(null);

  useEffect(() => {
    getGameStats()
      .then(setStats)
      .catch((e) => console.error("Failed to load stats:", e));
  }, []);

  if (!stats) return <p className="loading-text">Loading stats...</p>;

  return (
    <div className="stats-container">
      <h2>Library Statistics</h2>

      <div className="stats-grid">
        <div className="stat-card">
          <h2 className="stat-value">{stats.total_games}</h2>
          <p>Total Games</p>
        </div>

        <div className="stat-card">
          <h2 className="stat-value">{stats.total_playtime.toFixed(1)}h</h2>
          <p>Total Playtime</p>
        </div>

        <div className="stat-card">
          <h3>By Platform</h3>
          {stats.by_platform.map(([platform, count]) => (
            <p key={platform}>
              {platform}: <strong>{count}</strong>
            </p>
          ))}
          {stats.by_platform.length === 0 && <p className="empty-text">No data</p>}
        </div>

        <div className="stat-card">
          <h3>By Status</h3>
          {stats.by_status.map(([status, count]) => (
            <p key={status}>
              {status}: <strong>{count}</strong>
            </p>
          ))}
          {stats.by_status.length === 0 && <p className="empty-text">No data</p>}
        </div>
      </div>
    </div>
  );
}
