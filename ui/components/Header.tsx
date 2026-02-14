/** Props for the app header navigation. */
interface HeaderProps {
  currentView: string;
  onNavigate: (view: "library" | "stats" | "indexing") => void;
}

/** App header with primary view navigation controls. */
export function Header({ currentView, onNavigate }: HeaderProps) {
  return (
    <header className="header">
      <h1 className="header-title">Game Tracker</h1>
      <nav className="header-nav">
        <button
          className={`nav-btn ${currentView === "library" ? "active" : ""}`}
          onClick={() => onNavigate("library")}
        >
          Library
        </button>
        <button
          className={`nav-btn ${currentView === "stats" ? "active" : ""}`}
          onClick={() => onNavigate("stats")}
        >
          Stats
        </button>
        <button
          className={`nav-btn ${currentView === "indexing" ? "active" : ""}`}
          onClick={() => onNavigate("indexing")}
        >
          Index Games
        </button>
      </nav>
    </header>
  );
}
