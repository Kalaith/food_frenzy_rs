use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CUSTOMER_TYPES_JSON: &str = include_str!("../assets/data/customer_types.json");
const DISH_TYPES_JSON: &str = include_str!("../assets/data/dish_types.json");
const UPGRADES_JSON: &str = include_str!("../assets/data/upgrades.json");
const RECIPES_JSON: &str = include_str!("../assets/data/recipes.json");
const ACHIEVEMENTS_JSON: &str = include_str!("../assets/data/achievements.json");
const GAME_BALANCE_JSON: &str = include_str!("../assets/data/game_balance.json");

pub const STATION_COLORS: [&str; 4] = ["blue", "green", "yellow", "red"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomerSpecialTraits {
    #[serde(default)]
    pub low_appetite: bool,
    #[serde(default)]
    pub can_wander: bool,
    #[serde(default)]
    pub multiplies_on_process: bool,
    #[serde(default)]
    pub fast_spoilage: bool,
    #[serde(default)]
    pub can_steal_food: bool,
    #[serde(default)]
    pub can_eat_waste: bool,
    #[serde(default)]
    pub high_yield: bool,
    #[serde(default)]
    pub throws_food: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerType {
    pub id: String,
    pub name: String,
    pub preferred_dishes: Vec<String>,
    pub base_deliciousness: f32,
    pub description: String,
    #[serde(default)]
    pub initially_unlocked: bool,
    #[serde(default)]
    pub profile_tier: u32,
    #[serde(default)]
    pub unlock_cost: HashMap<String, i64>,
    #[serde(default)]
    pub special_traits: Option<CustomerSpecialTraits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DishType {
    pub color: String,
    pub name: String,
    pub cook_time_ms: f32,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upgrade {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub base_cost: i64,
    pub cost: i64,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub max_level: u32,
    pub cost_growth: f64,
    #[serde(default)]
    pub purchased: bool,
    #[serde(default)]
    pub effects: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub ingredients: HashMap<String, i64>,
    #[serde(default)]
    pub customer_type: Option<String>,
    pub unlocked: bool,
    pub unlock_condition: String,
    pub profit_multiplier: f64,
    pub base_value: i64,
    pub capacity_bonus: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub progress: i64,
    pub max_progress: i64,
    pub reward: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBalance {
    pub customer_spawn_interval: f32,
    pub initial_spawn_delays: Vec<f32>,
    pub base_satisfaction_gain: f32,
    pub preferred_satisfaction_gain: f32,
    pub max_satisfaction_per_type: f32,
    pub max_feeding_capacity_bonus: i64,
    pub vip_deliciousness_threshold: f32,
    pub vip_satisfaction_threshold: f32,
    pub base_score_multiplier: f64,
    pub preferred_dish_score_multiplier: f64,
    pub vip_points_per_deliciousness: f64,
    pub max_customers: u32,
    pub overfeed_multiplier: f32,
    pub customer_patience_time: f32,
    pub trait_tick_interval: f32,
    pub satisfaction_decay_interval: f32,
    pub prestige_score_requirement: i64,
    pub regular_ingredient_name: String,
    pub satisfaction_decay_rate: f32,
    pub overfeed_multiplier_for_low_appetite: f32,
    pub special_table_process_time: f32,
    pub cooking_slots_limit: usize,
    pub starting_regular_ingredients: i64,
    #[serde(default = "default_dish_bill_value")]
    pub dish_bill_value: i64,
    #[serde(default = "default_preferred_bill_multiplier")]
    pub preferred_bill_multiplier: f64,
    #[serde(default = "default_satisfied_tip_rate")]
    pub satisfied_tip_rate: f64,
    #[serde(default = "default_content_dwell_ms")]
    pub content_dwell_ms: f32,
    #[serde(default = "default_min_courses")]
    pub min_courses: u32,
    #[serde(default = "default_max_courses")]
    pub max_courses: u32,
    #[serde(default = "default_visits_until_ready")]
    pub visits_until_ready: u32,
}

fn default_min_courses() -> u32 {
    1
}

fn default_max_courses() -> u32 {
    3
}

fn default_visits_until_ready() -> u32 {
    5
}

fn default_dish_bill_value() -> i64 {
    6
}

fn default_preferred_bill_multiplier() -> f64 {
    2.0
}

fn default_satisfied_tip_rate() -> f64 {
    0.5
}

fn default_content_dwell_ms() -> f32 {
    4_000.0
}

impl Default for GameBalance {
    fn default() -> Self {
        Self {
            customer_spawn_interval: 15_000.0,
            initial_spawn_delays: vec![4_000.0, 11_000.0],
            base_satisfaction_gain: 8.0,
            preferred_satisfaction_gain: 12.0,
            max_satisfaction_per_type: 40.0,
            max_feeding_capacity_bonus: 80,
            vip_deliciousness_threshold: 3.0,
            vip_satisfaction_threshold: 120.0,
            base_score_multiplier: 1.0,
            preferred_dish_score_multiplier: 2.0,
            vip_points_per_deliciousness: 140.0,
            max_customers: 2,
            overfeed_multiplier: 1.5,
            customer_patience_time: 135_000.0,
            trait_tick_interval: 8_000.0,
            satisfaction_decay_interval: 3_000.0,
            prestige_score_requirement: 50_000,
            regular_ingredient_name: "regular".to_string(),
            satisfaction_decay_rate: 0.5,
            overfeed_multiplier_for_low_appetite: 1.25,
            special_table_process_time: 3_000.0,
            cooking_slots_limit: 3,
            starting_regular_ingredients: -1,
            dish_bill_value: 6,
            preferred_bill_multiplier: 2.0,
            satisfied_tip_rate: 0.5,
            content_dwell_ms: 2_000.0,
            min_courses: 1,
            max_courses: 3,
            visits_until_ready: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub customer_types: Vec<CustomerType>,
    pub dish_types: Vec<DishType>,
    pub upgrades: Vec<Upgrade>,
    pub recipes: Vec<Recipe>,
    pub achievements: Vec<Achievement>,
    pub balance: GameBalance,
}

impl GameData {
    pub fn load() -> Self {
        Self {
            customer_types: parse_or_fallback(CUSTOMER_TYPES_JSON, "[]"),
            dish_types: parse_or_fallback(DISH_TYPES_JSON, "[]"),
            upgrades: parse_or_fallback(UPGRADES_JSON, "[]"),
            recipes: parse_or_fallback(RECIPES_JSON, "[]"),
            achievements: parse_or_fallback(ACHIEVEMENTS_JSON, "[]"),
            balance: parse_or_fallback(GAME_BALANCE_JSON, "{}"),
        }
    }

    pub fn customer_type_by_id(&self, id: &str) -> Option<&CustomerType> {
        self.customer_types.iter().find(|item| item.id == id)
    }

    pub fn dish_type_by_color(&self, color: &str) -> Option<&DishType> {
        self.dish_types.iter().find(|item| item.color == color)
    }
}

fn parse_or_fallback<T>(embedded: &str, fallback: &str) -> T
where
    T: DeserializeOwned + Default,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = data_file_path("game_data", embedded);
        if let Ok(raw) = std::fs::read_to_string(path) {
            if let Ok(value) = serde_json::from_str::<T>(&raw) {
                return value;
            }
        }
    }

    serde_json::from_str::<T>(embedded)
        .unwrap_or_else(|_| serde_json::from_str::<T>(fallback).unwrap_or_else(|_| T::default()))
}

#[cfg(not(target_arch = "wasm32"))]
fn data_file_path(filename: &str, embedded: &str) -> String {
    let _ = embedded;
    format!("assets/data/{filename}.json")
}

// `parse_or_fallback` deliberately degrades at runtime; these tests are the
// loud counterpart so a broken or drifted JSON asset fails CI instead of
// silently shipping empty content.
#[cfg(test)]
mod tests {
    use super::*;

    fn parsed() -> GameData {
        GameData {
            customer_types: serde_json::from_str(CUSTOMER_TYPES_JSON)
                .expect("customer_types.json must parse"),
            dish_types: serde_json::from_str(DISH_TYPES_JSON).expect("dish_types.json must parse"),
            upgrades: serde_json::from_str(UPGRADES_JSON).expect("upgrades.json must parse"),
            recipes: serde_json::from_str(RECIPES_JSON).expect("recipes.json must parse"),
            achievements: serde_json::from_str(ACHIEVEMENTS_JSON)
                .expect("achievements.json must parse"),
            balance: serde_json::from_str(GAME_BALANCE_JSON).expect("game_balance.json must parse"),
        }
    }

    fn meat_key(customer_type_id: &str) -> String {
        // Must match the key minted in `gameplay.rs` when a guest is processed.
        format!("{customer_type_id}-meat")
    }

    fn assert_unique_ids(kind: &str, ids: &[&str]) {
        let mut seen = std::collections::HashSet::new();
        for id in ids {
            assert!(seen.insert(*id), "duplicate {kind} id: {id}");
        }
    }

    #[test]
    fn all_embedded_assets_parse_strictly() {
        let data = parsed();
        assert!(!data.customer_types.is_empty(), "no customer types");
        assert!(!data.dish_types.is_empty(), "no dish types");
        assert!(!data.upgrades.is_empty(), "no upgrades");
        assert!(!data.recipes.is_empty(), "no recipes");
        assert!(!data.achievements.is_empty(), "no achievements");
    }

    #[test]
    fn ids_are_unique_across_each_asset() {
        let data = parsed();
        assert_unique_ids(
            "customer type",
            &data
                .customer_types
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
        );
        assert_unique_ids(
            "upgrade",
            &data
                .upgrades
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
        );
        assert_unique_ids(
            "recipe",
            &data
                .recipes
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
        );
        assert_unique_ids(
            "achievement",
            &data
                .achievements
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn dish_types_cover_exactly_the_station_colors() {
        let data = parsed();
        let colors: Vec<&str> = data
            .dish_types
            .iter()
            .map(|dish| dish.color.as_str())
            .collect();
        assert_eq!(colors.len(), STATION_COLORS.len());
        for color in STATION_COLORS {
            assert!(colors.contains(&color), "missing dish for station {color}");
        }
        for dish in &data.dish_types {
            assert!(
                dish.cook_time_ms > 0.0,
                "{}: cook time must be > 0",
                dish.color
            );
            assert!(
                !dish.examples.is_empty(),
                "{}: needs example names",
                dish.color
            );
        }
    }

    #[test]
    fn customer_types_reference_real_dishes_and_meats() {
        let data = parsed();
        let valid_ingredients: Vec<String> = data
            .customer_types
            .iter()
            .map(|item| meat_key(&item.id))
            .chain(std::iter::once(
                data.balance.regular_ingredient_name.clone(),
            ))
            .collect();

        for customer_type in &data.customer_types {
            assert!(
                !customer_type.preferred_dishes.is_empty(),
                "{}: needs at least one preferred dish",
                customer_type.id
            );
            for dish_color in &customer_type.preferred_dishes {
                assert!(
                    data.dish_type_by_color(dish_color).is_some(),
                    "{}: unknown preferred dish {dish_color}",
                    customer_type.id
                );
            }
            for ingredient in customer_type.unlock_cost.keys() {
                assert!(
                    valid_ingredients.contains(ingredient),
                    "{}: unlock cost references unknown ingredient {ingredient}",
                    customer_type.id
                );
            }
            assert!(
                customer_type.initially_unlocked || !customer_type.unlock_cost.is_empty(),
                "{}: locked type must have an unlock cost or it is unreachable",
                customer_type.id
            );
        }
    }

    #[test]
    fn recipes_reference_real_customer_types_and_meats() {
        let data = parsed();
        let valid_ingredients: Vec<String> = data
            .customer_types
            .iter()
            .map(|item| meat_key(&item.id))
            .chain(std::iter::once(
                data.balance.regular_ingredient_name.clone(),
            ))
            .collect();

        for recipe in &data.recipes {
            assert!(
                !recipe.ingredients.is_empty(),
                "{}: no ingredients",
                recipe.id
            );
            for ingredient in recipe.ingredients.keys() {
                assert!(
                    valid_ingredients.contains(ingredient),
                    "{}: unknown ingredient {ingredient}",
                    recipe.id
                );
            }
            if let Some(customer_type) = &recipe.customer_type {
                assert!(
                    data.customer_type_by_id(customer_type).is_some(),
                    "{}: unknown customer type {customer_type}",
                    recipe.id
                );
            }
            assert!(
                recipe.base_value > 0,
                "{}: base value must be > 0",
                recipe.id
            );
        }
    }

    #[test]
    fn upgrade_effects_only_use_keys_the_engine_reads() {
        // Keep in sync with the `get_effect` call sites in engine.rs/gameplay.rs;
        // an unread effect key is a balance change that silently does nothing.
        const KNOWN_EFFECTS: [&str; 9] = [
            "capacity_gain_multiplier",
            "combo_multiplier",
            "cook_time_multiplier",
            "max_customers_bonus",
            "meat_yield_multiplier",
            "patience_multiplier",
            "recipe_value_multiplier",
            "satisfaction_decay_multiplier",
            "spawn_interval_multiplier",
        ];
        let data = parsed();
        for upgrade in &data.upgrades {
            assert!(!upgrade.effects.is_empty(), "{}: no effects", upgrade.id);
            for key in upgrade.effects.keys() {
                assert!(
                    KNOWN_EFFECTS.contains(&key.as_str()),
                    "{}: effect key {key} is never read by the engine",
                    upgrade.id
                );
            }
            assert!(
                upgrade.max_level >= 1,
                "{}: max level must be >= 1",
                upgrade.id
            );
            assert!(
                upgrade.cost_growth >= 1.0,
                "{}: cost growth below 1.0 makes upgrades cheaper over time",
                upgrade.id
            );
        }
    }

    #[test]
    fn achievements_and_balance_are_sane() {
        let data = parsed();
        for achievement in &data.achievements {
            assert!(
                achievement.max_progress > 0,
                "{}: max progress must be > 0",
                achievement.id
            );
        }
        let balance = &data.balance;
        assert!(balance.max_customers >= 1);
        assert!(balance.visits_until_ready >= 1);
        assert!(balance.min_courses >= 1);
        assert!(balance.min_courses <= balance.max_courses);
        assert!(balance.max_courses as usize <= STATION_COLORS.len());
        assert!(balance.prestige_score_requirement > 0);
        assert!(balance.special_table_process_time > 0.0);
    }
}
