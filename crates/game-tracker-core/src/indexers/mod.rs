//! Launcher discovery adapters.
//!
//! Each indexer scans a source-specific installation database and returns
//! normalized [`crate::models::DiscoveredGame`] entries for upsert.

/// Epic Games Store manifest indexer.
pub mod epic;
/// Steam library indexer.
pub mod steam;
