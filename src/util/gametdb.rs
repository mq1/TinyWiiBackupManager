use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Cache state for GameTDB instance
enum CacheState {
    /// Not yet attempted to load
    NotLoaded,
    /// Successfully loaded and cached
    Loaded(Arc<GameTDB>),
    /// Attempted to load but failed (file doesn't exist or parse error)
    Failed,
}

static GAMETDB_INSTANCE: Mutex<CacheState> = Mutex::new(CacheState::NotLoaded);

#[derive(Debug, Deserialize)]
struct WiiTDBFile {
    #[serde(rename = "game")]
    games: Vec<GameEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // These fields are accessed through getter methods and will be used in future phases
pub struct GameEntry {
    #[serde(rename = "@name")]
    pub name: String,
    pub id: String,
    pub region: Option<String>,
    pub languages: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub rating: Option<Rating>,
    pub input: Option<Input>,
    #[serde(rename = "wi-fi")]
    pub wifi: Option<WiFi>,
    #[serde(default, rename = "locale")]
    pub locales: Vec<LocaleInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Used through get_rating() method
pub struct Rating {
    #[serde(rename = "@type")]
    pub rating_type: Option<String>,
    #[serde(rename = "@value")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Used through get_players() method
pub struct Input {
    #[serde(rename = "@players")]
    pub players: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Will be used in future phases
pub struct WiFi {
    #[serde(rename = "@players")]
    pub players: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // synopsis is used through get_synopsis() method
pub struct LocaleInfo {
    #[serde(rename = "@lang")]
    pub lang: String,
    pub title: String,
    pub synopsis: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GameTDB {
    games: HashMap<String, GameEntry>,
}

/// Clear the cached GameTDB instance to force reload on next access
pub fn clear_cache() {
    let mut cache = GAMETDB_INSTANCE.lock().unwrap();
    *cache = CacheState::NotLoaded;
    log::info!("GameTDB cache cleared");
}

impl GameTDB {
    pub fn load(path: &Path) -> Result<Self> {
        log::info!("Loading GameTDB from: {:?}", path);

        let file = BufReader::new(File::open(path)?);
        let db: WiiTDBFile = quick_xml::de::from_reader(file)?;

        let mut games = HashMap::new();
        for game in db.games {
            let id = game.id.clone();

            // Insert with full ID (either 4 or 6 characters)
            games.insert(id.clone(), game.clone());

            // For 6-char IDs, also index by first 4 characters for compatibility
            // This allows looking up games by their 4-char disc ID
            if id.len() == 6 {
                let id4 = id[..4].to_string();
                games.entry(id4).or_insert(game);
            }
        }

        log::info!("Loaded {} unique game IDs from GameTDB", games.len());
        Ok(Self { games })
    }

    pub fn load_from_base_dir(base_dir: &Path) -> Option<Arc<Self>> {
        let mut cache = GAMETDB_INSTANCE.lock().unwrap();

        match &*cache {
            CacheState::Loaded(db) => {
                // Already loaded, return a clone of the Arc
                Some(Arc::clone(db))
            }
            CacheState::Failed => {
                // Previously failed to load, don't try again
                None
            }
            CacheState::NotLoaded => {
                // First attempt to load
                let wiitdb_path = base_dir.join("apps/usbloader_gx/wiitdb.xml");
                if wiitdb_path.exists() {
                    match Self::load(&wiitdb_path) {
                        Ok(db) => {
                            log::info!("GameTDB loaded successfully from {:?}", wiitdb_path);
                            let arc_db = Arc::new(db);
                            *cache = CacheState::Loaded(Arc::clone(&arc_db));
                            Some(arc_db)
                        }
                        Err(e) => {
                            log::error!("Failed to load GameTDB: {}", e);
                            *cache = CacheState::Failed;
                            None
                        }
                    }
                } else {
                    log::debug!("GameTDB not found at {:?}", wiitdb_path);
                    *cache = CacheState::Failed;
                    None
                }
            }
        }
    }

    pub fn get_game(&self, id: &str) -> Option<&GameEntry> {
        // Try exact match first
        self.games.get(id).or_else(|| {
            // If ID is longer than 6, try first 6 characters
            if id.len() > 6 {
                self.games.get(&id[..6])
            } else if id.len() > 4 {
                // If ID is 5 characters, try first 4
                self.games.get(&id[..4])
            } else {
                None
            }
        })
    }

    pub fn get_title(&self, id: &str, locale: Option<&str>) -> Option<String> {
        let game = self.get_game(id)?;

        // Try specified locale
        if let Some(locale_str) = locale
            && let Some(locale_info) = game.locales.iter().find(|l| l.lang == locale_str)
            && !locale_info.title.is_empty()
        {
            return Some(locale_info.title.clone());
        }

        // Try English as fallback
        if let Some(locale_info) = game.locales.iter().find(|l| l.lang == "EN")
            && !locale_info.title.is_empty()
        {
            return Some(locale_info.title.clone());
        }

        // Try any available locale
        if let Some(locale_info) = game.locales.first()
            && !locale_info.title.is_empty()
        {
            return Some(locale_info.title.clone());
        }

        // Fallback to game name
        if !game.name.is_empty() {
            Some(game.name.clone())
        } else {
            None
        }
    }

    #[allow(dead_code)] // Will be used for game info display
    pub fn get_synopsis(&self, id: &str, locale: Option<&str>) -> Option<String> {
        let game = self.get_game(id)?;

        // Try specified locale
        if let Some(locale_str) = locale
            && let Some(locale_info) = game.locales.iter().find(|l| l.lang == locale_str)
            && let Some(ref synopsis) = locale_info.synopsis
            && !synopsis.is_empty()
        {
            return Some(synopsis.clone());
        }

        // Try English as fallback
        if let Some(locale_info) = game.locales.iter().find(|l| l.lang == "EN")
            && let Some(ref synopsis) = locale_info.synopsis
            && !synopsis.is_empty()
        {
            return Some(synopsis.clone());
        }

        // Try any available locale
        for locale_info in &game.locales {
            if let Some(ref synopsis) = locale_info.synopsis
                && !synopsis.is_empty()
            {
                return Some(synopsis.clone());
            }
        }

        None
    }

    #[allow(dead_code)] // Will be used for game info display
    pub fn get_developer(&self, id: &str) -> Option<String> {
        self.get_game(id)?.developer.clone()
    }

    #[allow(dead_code)] // Will be used for game info display
    pub fn get_publisher(&self, id: &str) -> Option<String> {
        self.get_game(id)?.publisher.clone()
    }

    #[allow(dead_code)] // Will be used for game info display
    pub fn get_genre(&self, id: &str) -> Option<String> {
        self.get_game(id)?.genre.clone()
    }

    #[allow(dead_code)] // Will be used for game info display
    pub fn get_rating(&self, id: &str) -> Option<(String, String)> {
        let game = self.get_game(id)?;
        if let Some(ref rating) = game.rating
            && let (Some(rating_type), Some(value)) = (&rating.rating_type, &rating.value)
            && !rating_type.is_empty()
            && !value.is_empty()
        {
            return Some((rating_type.clone(), value.clone()));
        }
        None
    }

    #[allow(dead_code)] // Will be used for game info display
    pub fn get_players(&self, id: &str) -> Option<u8> {
        self.get_game(id)?.input.as_ref()?.players
    }

    #[allow(dead_code)] // Useful for debugging and stats
    pub fn game_count(&self) -> usize {
        // Count unique game entries (not including duplicates for 4/6 char IDs)
        self.games
            .values()
            .filter(|g| g.id.len() == 6) // Count only full IDs
            .count()
    }

    #[allow(dead_code)] // May be useful for testing or future hot-reload features
    pub fn clear_cache() {
        // This would only be used in tests or when reloading
        // Since we use OnceLock, we can't actually clear it in production
        // But this is here for API completeness
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_wiitdb_path() -> PathBuf {
        PathBuf::from("wiitdb.xml")
    }

    #[test]
    fn test_load_wiitdb() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found in project root");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");
        assert!(
            db.game_count() > 10000,
            "Expected at least 10000 games, got {}",
            db.game_count()
        );
    }

    #[test]
    fn test_get_game_by_id() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        // Test with Super Mario Galaxy (RMGE01)
        let game = db.get_game("RMGE01");
        assert!(game.is_some(), "Should find Super Mario Galaxy");

        let game = game.unwrap();
        assert_eq!(game.id, "RMGE01");
        assert!(!game.locales.is_empty());
    }

    #[test]
    fn test_get_game_with_partial_id() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        // Test with 4-char ID
        let game = db.get_game("RMGE");
        assert!(game.is_some(), "Should find game with 4-char ID");

        // Test with 6-char ID
        let game = db.get_game("RMGE01");
        assert!(game.is_some(), "Should find game with 6-char ID");
    }

    #[test]
    fn test_get_title() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        // Test getting English title
        let title = db.get_title("RMGE01", Some("EN"));
        assert!(
            title.is_some(),
            "Should find English title for Super Mario Galaxy"
        );
        assert!(
            title.unwrap().contains("Mario"),
            "Title should contain 'Mario'"
        );

        // Test fallback to any available locale
        let title = db.get_title("RMGE01", Some("ZZ")); // Non-existent locale
        assert!(title.is_some(), "Should fallback to available title");
    }

    #[test]
    fn test_get_synopsis() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        // Test getting synopsis for a known game
        let synopsis = db.get_synopsis("RMGE01", Some("EN"));
        assert!(
            synopsis.is_some(),
            "Should find synopsis for Super Mario Galaxy"
        );
    }

    #[test]
    fn test_game_metadata() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        // Test various metadata fields
        let developer = db.get_developer("RMGE01");
        assert!(developer.is_some(), "Should have developer");

        let publisher = db.get_publisher("RMGE01");
        assert!(publisher.is_some(), "Should have publisher");

        let game = db
            .get_game("RMGE01")
            .expect("Should find Super Mario Galaxy");
        assert!(game.region.is_some(), "Should have region");
        assert!(game.languages.is_some(), "Should have languages");
    }

    #[test]
    fn test_multiple_locales() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        // Find a game with multiple locales (many games have multiple)
        // Using RMGE01 which should have multiple language support
        let game = db.get_game("RMGE01").expect("Should find game");
        assert!(!game.locales.is_empty(), "Should have at least one locale");

        // Check that EN locale exists for this game
        let has_en = game.locales.iter().any(|l| l.lang == "EN");
        assert!(has_en, "Should have English locale");
    }

    #[test]
    fn test_rating_info() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        // Test rating for a known game
        let rating = db.get_rating("RMGE01");
        if let Some((rating_type, value)) = rating {
            assert!(!rating_type.is_empty(), "Rating type should not be empty");
            assert!(!value.is_empty(), "Rating value should not be empty");
        }
    }

    #[test]
    fn test_nonexistent_game() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        let game = db.get_game("ZZZZZZ");
        assert!(game.is_none(), "Should not find non-existent game");

        let title = db.get_title("ZZZZZZ", Some("EN"));
        assert!(
            title.is_none(),
            "Should not find title for non-existent game"
        );
    }

    #[test]
    fn test_additional_metadata() {
        let path = get_test_wiitdb_path();
        if !path.exists() {
            eprintln!("Skipping test: wiitdb.xml not found");
            return;
        }

        let db = GameTDB::load(&path).expect("Failed to load GameTDB");

        // Test getting genre
        let genre = db.get_genre("RMGE01");
        // Genre might not be present for all games
        if genre.is_some() {
            assert!(
                !genre.unwrap().is_empty(),
                "Genre should not be empty if present"
            );
        }

        // Test getting player count
        let players = db.get_players("RMGE01");
        // Player count might not be present for all games
        if players.is_some() {
            assert!(players.unwrap() > 0, "Player count should be positive");
        }
    }
}
