# Game Tracker

Game Tracker is a Windows-first Tauri 2 desktop app for tracking games across PC, Switch, PS4, and emulators. It supports manual entry, RAWG metadata enrichment, and automatic Steam/Epic discovery.

## Current Architecture

The active application path is:
- `ui/` (React + TypeScript frontend)
- `src-tauri/` (Tauri runtime and command API)
- `crates/game-tracker-core/` (shared Rust business logic)

Legacy Axum/HTMX assets (`src/`, `templates/`, `static/`) are still in the repository for transition/reference but are not the primary desktop runtime path.

## Tech Stack

- **Desktop shell:** Tauri v2
- **Frontend:** React 19, TypeScript, Vite
- **Backend/core:** Rust, SQLx, SQLite, Tokio
- **Integrations:** RAWG API, Steam (`steamlocate`), Epic manifest parsing
- **Assets:** Windows `.exe` icon extraction via `exeico`

## Prerequisites

- Rust toolchain (stable) with `cargo`
- Node.js 18+ and `pnpm`
- WebView2 runtime on Windows (normally already present with modern Edge)

## Environment Setup

1. Install dependencies:
   ```bash
   pnpm install
   ```
2. Copy `.env.example` to `.env` and set values as needed.

Environment variables:

| Variable | Purpose |
| --- | --- |
| `RAWG_API_KEY` | Optional but recommended for RAWG search/details enrichment |
| `DATABASE_URL` | Legacy Axum path setting; desktop app uses app-data SQLite path |

## Run and Build

Desktop development:

```bash
pnpm tauri:dev
```

Desktop production build:

```bash
pnpm tauri:build
```

Other useful commands:

| Command | Description |
| --- | --- |
| `pnpm dev` | Run Vite frontend only |
| `pnpm build` | Build frontend bundle |
| `pnpm lint` | Run ESLint |
| `cargo check --workspace` | Type-check Rust workspace crates |
| `cargo test --workspace` | Run Rust tests (including indexer tests) |

Notes:
- `src-tauri/tauri.conf.json` runs frontend dev/build automatically via `beforeDevCommand` and `beforeBuildCommand`.
- Migrations are executed by app startup (`sqlx::migrate!`) for the desktop runtime.

## Project Structure

```text
game-tracker/
├── Cargo.toml                      # Workspace root (members: src-tauri, core crate)
├── package.json                    # pnpm scripts for Vite + Tauri
├── vite.config.ts                  # Vite config used by Tauri dev/build
├── migrations/                     # SQLite schema migrations
├── src-tauri/                      # Tauri v2 backend
│   ├── src/lib.rs                  # Runtime setup, state creation, command registration
│   ├── src/commands.rs             # Frontend-callable command API
│   └── tauri.conf.json             # Build/dev window and bundling config
├── crates/game-tracker-core/       # Shared Rust logic
│   └── src/
│       ├── models.rs               # Domain models + command DTOs
│       ├── db.rs                   # SQLx queries
│       ├── rawg.rs                 # RAWG client
│       ├── icon_extract.rs         # Cover download + exe icon extraction
│       ├── service.rs              # GameService orchestration
│       └── indexers/               # Steam + Epic discovery
├── ui/                             # React app
│   ├── hooks/useBackend.ts         # Typed invoke wrappers
│   └── components/                 # Library, Stats, Add modal, Index panel
├── src/                            # Legacy Axum backend (transition)
├── templates/                      # Legacy Askama templates (transition)
└── static/                         # Legacy static assets (transition)
```

## Data Flow

```text
React UI (ui/) -> invoke() -> Tauri commands (src-tauri/src/commands.rs)
-> GameService (crates/game-tracker-core/src/service.rs)
-> db/rawg/indexers/icon helpers
```

## Auto-Indexing

`Index Games` scans supported launchers and upserts by `(source, source_id)`:

- **Steam:** discovered via `steamlocate` libraries/apps
- **Epic:** parsed from `.item` manifests under ProgramData

Upsert behavior avoids duplicates and refreshes install/executable paths when they change.

## Migrations

- `20260211000000_initial_schema.sql`: base `games` and `play_sessions` schema
- `20260213000000_add_source_fields.sql`: launcher source tracking (`source`, `source_id`, `install_path`)

## Notes on PLAN.md

`PLAN.md` documents the original Axum/HTMX implementation path and should be treated as historical planning context. The source of truth for current runtime architecture is the Tauri + React + core-crate code path described above.

## Contributing

- Keep docs and code in sync when changing commands, models, or architecture paths.
- Prefer updates to inline Rust/TypeScript docs and this README instead of adding extra markdown files.
- Run `cargo check --workspace`, `cargo test --workspace`, and `pnpm lint` before opening a PR.
