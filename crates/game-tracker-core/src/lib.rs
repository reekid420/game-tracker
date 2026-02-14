//! Shared business logic for the Game Tracker desktop app.
//!
//! This crate contains:
//! - typed data models shared across backend layers
//! - SQLx database access helpers
//! - RAWG metadata client
//! - icon extraction and cover download utilities
//! - launcher indexers (Steam/Epic)
//! - `GameService`, the orchestration layer used by Tauri commands

/// Database access helpers for the `games` table and statistics queries.
pub mod db;
/// Image/icon helpers for local executable icons and remote cover downloads.
pub mod icon_extract;
/// Launcher-specific game discovery modules.
pub mod indexers;
/// Shared DTOs and persisted model types.
pub mod models;
/// RAWG API client and response types.
pub mod rawg;
/// High-level service layer that coordinates CRUD, enrichment, and indexing.
pub mod service;
