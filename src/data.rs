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
