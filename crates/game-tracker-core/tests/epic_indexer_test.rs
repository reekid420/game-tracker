//! Tests for Epic Games indexer manifest parsing.

use game_tracker_core::indexers::epic::scan_epic_games_from;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn test_scan_epic_discovers_games_only() {
    let games = scan_epic_games_from(&fixtures_dir()).expect("scan should succeed");

    // Should find 2 games, not the DLC (bIsApplication = false)
    assert_eq!(games.len(), 2, "Expected 2 games, got {}", games.len());

    let titles: Vec<&str> = games.iter().map(|g| g.title.as_str()).collect();
    assert!(titles.contains(&"Test Game One"));
    assert!(titles.contains(&"Another Great Game"));
}

#[test]
fn test_epic_game_fields() {
    let games = scan_epic_games_from(&fixtures_dir()).expect("scan should succeed");

    let game1 = games.iter().find(|g| g.title == "Test Game One").unwrap();
    assert_eq!(game1.source, "epic");
    assert_eq!(game1.source_id, "TestGameOne123");
    assert_eq!(game1.platform, "PC");
    assert_eq!(
        game1.install_path.as_deref(),
        Some("C:\\Games\\TestGameOne")
    );
    // exe_path won't exist on disk in test, so it should be None
    assert!(game1.exe_path.is_none());
}

#[test]
fn test_epic_nonexistent_dir_returns_empty() {
    let games =
        scan_epic_games_from(&PathBuf::from("C:\\nonexistent\\path\\12345"))
            .expect("scan should succeed even for missing dir");
    assert!(games.is_empty());
}
