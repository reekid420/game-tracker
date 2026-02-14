//! Steam game auto-indexer using the `steamlocate` crate.

use crate::models::DiscoveredGame;

/// Scan all Steam library folders and return discovered games.
pub fn scan_steam_games() -> Result<Vec<DiscoveredGame>, Box<dyn std::error::Error + Send + Sync>> {
    let steam_dir = steamlocate::SteamDir::locate()?;

    let mut games = Vec::new();

    for library in steam_dir.libraries()?.filter_map(|l| l.ok()) {
        for app in library.apps().filter_map(|a| a.ok()) {
            let name = match &app.name {
                Some(n) if !n.is_empty() => n.clone(),
                _ => continue,
            };

            let app_dir = library.resolve_app_dir(&app);
            let install_path = app_dir.to_string_lossy().to_string();

            // Try to find a main .exe in the install directory
            let exe_path = find_main_exe(&install_path);

            games.push(DiscoveredGame {
                title: name,
                platform: "PC".to_string(),
                exe_path,
                install_path: Some(install_path),
                source: "steam".to_string(),
                source_id: app.app_id.to_string(),
            });
        }
    }

    Ok(games)
}

/// Best-effort: look for a common exe in the root of the install dir.
fn find_main_exe(install_path: &str) -> Option<String> {
    let path = std::path::Path::new(install_path);
    if !path.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(path).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("exe") {
            let name = p.file_name()?.to_string_lossy().to_lowercase();
            // Skip common non-game executables
            if name.contains("unins")
                || name.contains("redist")
                || name.contains("setup")
                || name.contains("crash")
                || name.contains("ue4prereq")
                || name.contains("dxsetup")
            {
                continue;
            }
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}
