//! Epic Games Store auto-indexer.
//!
//! Reads `.item` JSON manifest files from:
//! `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\`

use crate::models::DiscoveredGame;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Fields we care about from an Epic `.item` manifest.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EpicManifest {
    display_name: Option<String>,
    install_location: Option<String>,
    launch_executable: Option<String>,
    app_name: Option<String>,
    #[serde(default, rename = "bIsApplication")]
    b_is_application: bool,
}

/// Default manifest directory on Windows.
fn default_manifests_dir() -> PathBuf {
    PathBuf::from(r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests")
}

/// Scan Epic Games manifests and return discovered games.
pub fn scan_epic_games() -> Result<Vec<DiscoveredGame>, Box<dyn std::error::Error + Send + Sync>> {
    scan_epic_games_from(&default_manifests_dir())
}

/// Scan Epic Games manifests from a specific directory (useful for testing).
pub fn scan_epic_games_from(
    manifests_dir: &Path,
) -> Result<Vec<DiscoveredGame>, Box<dyn std::error::Error + Send + Sync>> {
    let mut games = Vec::new();

    if !manifests_dir.is_dir() {
        tracing::warn!("Epic manifests directory not found: {:?}", manifests_dir);
        return Ok(games);
    }

    for entry in std::fs::read_dir(manifests_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }

        match parse_manifest(&path) {
            Ok(Some(game)) => games.push(game),
            Ok(None) => {} // not a game (e.g. engine, tool)
            Err(e) => {
                tracing::warn!("Failed to parse Epic manifest {:?}: {}", path, e);
            }
        }
    }

    Ok(games)
}

fn parse_manifest(
    path: &Path,
) -> Result<Option<DiscoveredGame>, Box<dyn std::error::Error + Send + Sync>> {
    let contents = std::fs::read_to_string(path)?;
    let manifest: EpicManifest = serde_json::from_str(&contents)?;

    // Skip non-application entries (DLC, engine components)
    if !manifest.b_is_application {
        return Ok(None);
    }

    let title = match manifest.display_name {
        Some(ref n) if !n.is_empty() => n.clone(),
        _ => return Ok(None),
    };

    let app_name = match manifest.app_name {
        Some(ref n) if !n.is_empty() => n.clone(),
        _ => return Ok(None),
    };

    let install_path = manifest.install_location.clone();

    // Build full exe path from install location + launch executable
    let exe_path = match (&manifest.install_location, &manifest.launch_executable) {
        (Some(install), Some(launch)) => {
            let full = PathBuf::from(install).join(launch);
            if full.exists() {
                Some(full.to_string_lossy().to_string())
            } else {
                None
            }
        }
        _ => None,
    };

    Ok(Some(DiscoveredGame {
        title,
        platform: "PC".to_string(),
        exe_path,
        install_path,
        source: "epic".to_string(),
        source_id: app_name,
    }))
}
