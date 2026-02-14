import { useState } from "react";
import {
  type RawgGame,
  type CreateGameInput,
  searchRawg,
  createGame,
} from "../hooks/useBackend";

/** Lifecycle callbacks for the add-game modal. */
interface AddGameModalProps {
  onClose: () => void;
  onCreated: () => void;
}

const PLATFORMS = ["PC", "Switch", "PS4", "PS5", "Xbox", "Emulator"];
const STATUSES = ["Backlog", "Playing", "Completed", "Wishlist"];

/**
 * Modal for manual game creation with optional RAWG metadata matching.
 *
 * On successful creation it refreshes the parent view through `onCreated`.
 */
export function AddGameModal({ onClose, onCreated }: AddGameModalProps) {
  const [title, setTitle] = useState("");
  const [platform, setPlatform] = useState("PC");
  const [status, setStatus] = useState("Backlog");
  const [exePath, setExePath] = useState("");
  const [rawgResults, setRawgResults] = useState<RawgGame[]>([]);
  const [selectedRawgId, setSelectedRawgId] = useState<number | null>(null);
  const [searching, setSearching] = useState(false);

  const handleSearchRawg = async () => {
    if (!title.trim()) return;
    setSearching(true);
    try {
      const results = await searchRawg(title);
      setRawgResults(results);
    } catch (e) {
      console.error("RAWG search failed:", e);
    } finally {
      setSearching(false);
    }
  };

  const handleSubmit = async () => {
    if (!title.trim()) return;
    const input: CreateGameInput = {
      title: title.trim(),
      platform,
      status,
      rawg_id: selectedRawgId,
      exe_path: exePath || null,
    };
    try {
      await createGame(input);
      onCreated();
      onClose();
    } catch (e) {
      console.error("Failed to create game:", e);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <h2>Add New Game</h2>

        <div className="form-group">
          <input
            type="text"
            placeholder="Game Title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
          />
        </div>

        <div className="form-row">
          <select value={platform} onChange={(e) => setPlatform(e.target.value)}>
            {PLATFORMS.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </select>

          <select value={status} onChange={(e) => setStatus(e.target.value)}>
            {STATUSES.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <input
            type="text"
            placeholder="Path to .exe (PC games only)"
            value={exePath}
            onChange={(e) => setExePath(e.target.value)}
          />
        </div>

        <button
          className="btn-secondary"
          onClick={handleSearchRawg}
          disabled={searching || !title.trim()}
        >
          {searching ? "Searching..." : "Search RAWG Database"}
        </button>

        {rawgResults.length > 0 && (
          <div className="rawg-results">
            <h3>Select Match:</h3>
            {rawgResults.map((r) => (
              <label
                key={r.id}
                className={`rawg-result ${selectedRawgId === r.id ? "selected" : ""}`}
              >
                <input
                  type="radio"
                  name="rawg_id"
                  checked={selectedRawgId === r.id}
                  onChange={() => setSelectedRawgId(r.id)}
                />
                {r.background_image && (
                  <img src={r.background_image} alt={r.name} />
                )}
                <div>
                  <strong>{r.name}</strong>
                  {r.released && <small> ({r.released})</small>}
                </div>
              </label>
            ))}
          </div>
        )}

        <div className="modal-actions">
          <button className="btn-primary" onClick={handleSubmit}>
            Add Game
          </button>
          <button className="btn-cancel" onClick={onClose}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
