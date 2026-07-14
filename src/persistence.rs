//! Cross-platform persistence helpers for the game.

use crate::state::{GameState, GuestState, ProgressionState, Timers};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Bump when the save schema changes; `migrate` upgrades older versions.
/// v1: pre-review-roadmap shape (bare-string plated dishes, no tutorial /
///     day-cycle / regulars / specialization state).
/// v2: the Phase 1-3 systems landed; new fields fill via serde defaults and
///     `PlatedDish` reads the legacy bare-string form.
const SAVE_VERSION: u32 = 2;
const GAME_NAME: &str = "feast_frenzy";
#[cfg(target_arch = "wasm32")]
const SAVE_KEY: &str = "feast-frenzy-save.json";
#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILE_NAME: &str = "feast_frenzy.json";
#[cfg(not(target_arch = "wasm32"))]
const TEST_SAVE_PATH_ENV: &str = "feast_FRENZY_TEST_SAVE_PATH";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodFrenzySave {
    /// 0 = saves from before the version field existed.
    #[serde(default)]
    pub version: u32,
    pub game_state: GameState,
    pub progression_state: ProgressionState,
    pub guest_state: GuestState,
    pub timers: Timers,
    pub selected_station: Option<String>,
}

pub fn save_game(
    game_state: &GameState,
    progression_state: &ProgressionState,
    guest_state: &GuestState,
    timers: &Timers,
    selected_station: &Option<String>,
) -> Result<(), String> {
    let snapshot = FoodFrenzySave {
        version: SAVE_VERSION,
        game_state: game_state.clone(),
        progression_state: progression_state.clone(),
        guest_state: guest_state.clone(),
        timers: timers.clone(),
        selected_station: selected_station.clone(),
    };

    save_json(&snapshot)
}

pub fn load_game() -> Result<Option<FoodFrenzySave>, String> {
    if !save_exists() {
        return Ok(None);
    }

    let mut save = load_json::<FoodFrenzySave>()?;
    migrate(&mut save);
    Ok(Some(save))
}

/// Upgrade an older save in place. Most version gaps are absorbed by serde
/// defaults and `PlatedDish`'s legacy deserializer; anything that needs an
/// explicit fixup gets a version arm here.
fn migrate(save: &mut FoodFrenzySave) {
    if save.version < 2 {
        // v0/v1 saves predate the day cycle: their run effectively resumes at
        // the start of a fresh service day rather than mid-ledger.
        save.game_state.day_cycle.summary_pending = false;
        save.game_state.day_cycle.elapsed_ms = 0.0;
    }
    save.version = SAVE_VERSION;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_path() -> Result<PathBuf, String> {
    macroquad_toolkit::persistence::get_webhatchery_game_app_path(
        GAME_NAME,
        SAVE_FILE_NAME,
        Some(TEST_SAVE_PATH_ENV),
    )
    .ok_or_else(|| "Failed to resolve save directory".to_string())
}

fn save_json<T: Serialize>(value: &T) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::persistence::save_json_key(GAME_NAME, SAVE_KEY, value)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = save_path()?;
        macroquad_toolkit::persistence::save_json_atomic(&path, value)
            .map_err(|error| format!("Failed to write save file '{}': {error}", path.display()))
    }
}

fn load_json<T: serde::de::DeserializeOwned>() -> Result<T, String> {
    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::persistence::load_json_key(GAME_NAME, SAVE_KEY)
            .or_else(|_| load_legacy_browser_save())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = save_path()?;
        macroquad_toolkit::persistence::load_json(&path)
            .map_err(|error| format!("Failed to read save file '{}': {error}", path.display()))
    }
}

#[cfg(target_arch = "wasm32")]
fn load_legacy_browser_save<T: serde::de::DeserializeOwned>() -> Result<T, String> {
    let serialized = macroquad_toolkit::wasm_storage::storage_get(SAVE_KEY)
        .ok_or_else(|| format!("No browser save data found for '{SAVE_KEY}'."))?;
    let save = serde_json::from_str(&serialized)
        .map_err(|error| format!("Failed to parse save data: {error}"))?;
    let _ = macroquad_toolkit::persistence::save_string_key(GAME_NAME, SAVE_KEY, &serialized);
    Ok(save)
}

fn save_exists() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::persistence::json_key_exists(GAME_NAME, SAVE_KEY)
            || macroquad_toolkit::wasm_storage::storage_exists(SAVE_KEY)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(path) = save_path() {
            macroquad_toolkit::persistence::file_exists(path)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn can_save_and_load_round_trip() {
        let path = std::env::temp_dir().join(format!(
            "feast_frenzy_save_test_{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        std::env::set_var("feast_FRENZY_TEST_SAVE_PATH", &path);

        let data = GameData::load();
        let game_state = crate::state::GameState::new(&data);
        let progression_state = crate::state::ProgressionState::from_game_data(&data);
        let guest_state = crate::state::GuestState::new();
        let timers = crate::state::Timers::new();
        let selected = None;

        let err = save_game(
            &game_state,
            &progression_state,
            &guest_state,
            &timers,
            &selected,
        );
        assert!(err.is_ok(), "save should succeed: {err:?}");

        let loaded = load_game().unwrap_or(None);
        assert!(loaded.is_some());

        std::env::remove_var("feast_FRENZY_TEST_SAVE_PATH");
        let _ = std::fs::remove_file(path);
    }

    /// A trimmed real v1 save (pre-roadmap schema): no version-2 fields,
    /// plated dishes as bare strings, guest records without personalities.
    /// Loading it must succeed and migrate rather than wiping the run.
    #[test]
    fn legacy_v1_save_loads_and_migrates() {
        let legacy = r#"{
            "version": 1,
            "game_state": {
                "score": 420,
                "combo": 2,
                "chain": 3,
                "customers": [],
                "ingredients": { "regular": -1, "pig-meat": 3 },
                "cooking_stations": {
                    "blue": {
                        "color": "blue",
                        "is_cooking": false,
                        "remaining_ms": 0.0,
                        "dishes": ["Pickled Clover Tart"]
                    }
                },
                "special_table_busy": false,
                "special_table_timer": 0.0,
                "messages": ["Service started."],
                "next_customer_id": 7
            },
            "progression_state": {
                "currency": 55,
                "upgrades": [],
                "recipes": [],
                "achievements": [],
                "prestige_level": 0,
                "prestige_points": 0,
                "total_score": 420,
                "processed_customer_counts": { "pig": 1 },
                "processed_customer_types": ["pig"],
                "feeding_capacity_bonus": 0,
                "crafted_recipe_counts": {},
                "total_dishes_served": 9,
                "preferred_dishes_served": 4,
                "overfed_customer_count": 0,
                "customers_lost": 1,
                "unlocked_customer_types": ["pig", "sheep", "rabbit"]
            },
            "guest_state": {
                "guests": [{
                    "id": "guest-1",
                    "name": "Marnie",
                    "customer_type": "pig",
                    "visits": 4,
                    "feedings": 6,
                    "satisfied_visits": 3,
                    "processed_count": 0,
                    "last_seen_at": 0
                }]
            },
            "timers": {
                "elapsed_ms": 90000.0,
                "next_spawn_ms": 95000.0,
                "spawn_step": 2,
                "decay_accum_ms": 0.0,
                "patience_accum_ms": 0.0,
                "trait_accum_ms": 0.0,
                "save_accum_ms": 0.0
            },
            "selected_station": null
        }"#;

        let mut save: FoodFrenzySave =
            serde_json::from_str(legacy).expect("legacy v1 save must still deserialize");
        migrate(&mut save);

        assert_eq!(save.version, SAVE_VERSION);
        let station = &save.game_state.cooking_stations["blue"];
        assert_eq!(station.dishes[0].name, "Pickled Clover Tart");
        assert_eq!(station.dishes[0].age_ms, 0.0, "legacy dishes start fresh");
        assert!(save.game_state.tutorial.step_index == 0 && !save.game_state.tutorial.complete);
        assert_eq!(save.game_state.day_cycle.day, 1);
        assert!(!save.game_state.day_cycle.summary_pending);
        assert!(save.guest_state.guests[0].personality.is_none());
        assert!(save.progression_state.specialization.is_none());
        assert_eq!(save.progression_state.currency, 55);
    }
}
