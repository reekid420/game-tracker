import { useState } from "react";
import { Header } from "./components/Header";
import { Library } from "./components/Library";
import { Stats } from "./components/Stats";
import { IndexPanel } from "./components/IndexPanel";

/** Top-level views rendered inside the desktop app shell. */
type View = "library" | "stats" | "indexing";

/** Root React component that routes between primary app views. */
export default function App() {
  const [view, setView] = useState<View>("library");

  return (
    <div className="app">
      <Header currentView={view} onNavigate={setView} />
      <main className="main-content">
        {view === "library" && <Library />}
        {view === "stats" && <Stats />}
        {view === "indexing" && <IndexPanel />}
      </main>
    </div>
  );
}
