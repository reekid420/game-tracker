import { useState } from "react";
import { type IndexResult, indexNow } from "../hooks/useBackend";

/** Controls for running Steam/Epic discovery and showing summary results. */
export function IndexPanel() {
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<IndexResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleIndex = async () => {
    setRunning(true);
    setError(null);
    setResult(null);
    try {
      const res = await indexNow();
      setResult(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="index-panel">
      <h2>Game Indexing</h2>
      <p className="index-description">
        Automatically discover installed games from Steam and Epic Games
        Launcher. Discovered games are added to your library with "Backlog"
        status. Existing entries are updated if the install path changed.
      </p>

      <button
        className="btn-primary index-btn"
        onClick={handleIndex}
        disabled={running}
      >
        {running ? "Scanning..." : "Scan Now"}
      </button>

      {result && (
        <div className="index-result success">
          <p>
            Discovered <strong>{result.discovered}</strong> games,{" "}
            <strong>{result.upserted}</strong> added/updated in library.
          </p>
        </div>
      )}

      {error && (
        <div className="index-result error">
          <p>Indexing error: {error}</p>
        </div>
      )}

      <div className="index-sources">
        <h3>Supported Sources</h3>
        <ul>
          <li>
            <strong>Steam</strong> — Reads library folders and app manifests
          </li>
          <li>
            <strong>Epic Games</strong> — Parses install manifests from
            ProgramData
          </li>
        </ul>
      </div>
    </div>
  );
}
