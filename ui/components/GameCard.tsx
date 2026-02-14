import { type Game, updateGameStatus, deleteGame } from "../hooks/useBackend";

/** Props for rendering and mutating a single game entry. */
interface GameCardProps {
  game: Game;
  onUpdate: () => void;
}

const STATUSES = ["Backlog", "Playing", "Completed", "Wishlist"];

/** Card UI for one game with status update and delete actions. */
export function GameCard({ game, onUpdate }: GameCardProps) {
  const handleStatusChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    await updateGameStatus(game.id, e.target.value);
    onUpdate();
  };

  const handleDelete = async () => {
    if (confirm(`Delete "${game.title}"?`)) {
      await deleteGame(game.id);
      onUpdate();
    }
  };

  // Prefer cover art, then extracted icon path, otherwise show placeholder.
  const coverSrc = game.cover_url || game.icon_path || null;

  return (
    <div className="game-card">
      {coverSrc ? (
        <img className="game-card-img" src={coverSrc} alt={game.title} />
      ) : (
        <div className="game-card-no-img">🎮</div>
      )}

      <h3 className="game-card-title">{game.title}</h3>
      <span className="platform-badge">{game.platform}</span>

      {game.source && (
        <span className="source-badge">{game.source}</span>
      )}

      <select
        className="status-select"
        value={game.status}
        onChange={handleStatusChange}
      >
        {STATUSES.map((s) => (
          <option key={s} value={s}>
            {s}
          </option>
        ))}
      </select>

      {game.description && (
        <p className="game-card-desc">{game.description}</p>
      )}

      <p className="game-card-playtime">{game.playtime_hours}h played</p>

      <button className="delete-btn" onClick={handleDelete}>
        Delete
      </button>
    </div>
  );
}
