import { useEffect, useState, useCallback } from "react";
import { type Game, listGames, searchGames, filterGames } from "../hooks/useBackend";
import { GameCard } from "./GameCard";
import { AddGameModal } from "./AddGameModal";

const STATUSES = ["", "Playing", "Completed", "Backlog", "Wishlist"];

/**
 * Library view for browsing, searching, filtering, and creating games.
 *
 * Data refresh is debounced so typing/filter changes do not spam backend calls.
 */
export function Library() {
  const [games, setGames] = useState<Game[]>([]);
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState("");
  const [showAddModal, setShowAddModal] = useState(false);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      if (search.trim()) {
        setGames(await searchGames(search));
      } else if (statusFilter) {
        setGames(await filterGames(statusFilter));
      } else {
        setGames(await listGames());
      }
    } catch (e) {
      console.error("Failed to load games:", e);
    } finally {
      setLoading(false);
    }
  }, [search, statusFilter]);

  useEffect(() => {
    // Keep search responsive while limiting backend request frequency.
    const timer = setTimeout(refresh, 300);
    return () => clearTimeout(timer);
  }, [refresh]);

  return (
    <div className="library">
      <div className="controls">
        <input
          type="search"
          className="search-input"
          placeholder="Search games..."
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setStatusFilter("");
          }}
        />

        <select
          className="filter-select"
          value={statusFilter}
          onChange={(e) => {
            setStatusFilter(e.target.value);
            setSearch("");
          }}
        >
          <option value="">All</option>
          {STATUSES.filter(Boolean).map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </select>

        <button className="btn-primary" onClick={() => setShowAddModal(true)}>
          + Add Game
        </button>
      </div>

      {loading ? (
        <p className="loading-text">Loading...</p>
      ) : games.length === 0 ? (
        <p className="empty-text">
          No games found. Add some or run the indexer!
        </p>
      ) : (
        <div className="game-grid">
          {games.map((game) => (
            <GameCard key={game.id} game={game} onUpdate={refresh} />
          ))}
        </div>
      )}

      {showAddModal && (
        <AddGameModal
          onClose={() => setShowAddModal(false)}
          onCreated={refresh}
        />
      )}
    </div>
  );
}
