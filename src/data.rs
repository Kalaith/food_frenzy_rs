use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CUSTOMER_TYPES_JSON: &str = include_str!("../assets/data/customer_types.json");
const DISH_TYPES_JSON: &str = include_str!("../assets/data/dish_types.json");
const UPGRADES_JSON: &str = include_str!("../assets/data/upgrades.json");
const RECIPES_JSON: &str = include_str!("../assets/data/recipes.json");
const ACHIEVEMENTS_JSON: &str = include_str!("../assets/data/achievements.json");
const GAME_BALANCE_JSON: &str = include_str!("../assets/data/game_balance.json");
const TUTORIAL_JSON: &str = include_str!("../assets/data/tutorial.json");
const SPECIALIZATIONS_JSON: &str = include_str!("../assets/data/specializations.json");
const TRAIT_BEHAVIORS_JSON: &str = include_str!("../assets/data/trait_behaviors.json");

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

/// What has to happen in play for a tutorial step to complete and advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TutorialTrigger {
    GuestSeated,
    CookingStarted,
    DishCarried,
    CourseServed,
    GuestDepartedFed,
    GuestReady,
    GuestProcessed,
    /// Completed by the player clicking "Got it" rather than a game event.
    Acknowledged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub body: String,
    pub trigger: TutorialTrigger,
}

/// A house style the player commits to after their first processing. Effects
/// use the same keys as upgrades and feed the same `get_effect` accumulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecializationDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub flavor: String,
    pub effects: HashMap<String, f64>,
}

/// Player-facing behavior of a special trait: how it telegraphs, what the
/// counterplay is, and the one-time hint shown on first encounter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitBehavior {
    #[serde(rename = "trait")]
    pub trait_key: String,
    pub name: String,
    pub telegraph: String,
    pub counter: String,
    pub hint: String,
    pub telegraphed: bool,
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
    /// Visits needed before a guest is Lounge-ready, indexed by profile tier
    /// (tier 1 uses index 0). Cheap early tiers land the hook fast; later
    /// tiers stay a longer investment. Falls back to `visits_until_ready`
    /// when empty.
    #[serde(default)]
    pub visits_until_ready_by_tier: Vec<u32>,
    /// Plated dishes served within this window pay the fresh bonus.
    #[serde(default = "default_dish_fresh_window_ms")]
    pub dish_fresh_window_ms: f32,
    /// Plated dishes older than this are discarded from the pass.
    #[serde(default = "default_dish_spoil_ms")]
    pub dish_spoil_ms: f32,
    #[serde(default = "default_fresh_bill_bonus_multiplier")]
    pub fresh_bill_bonus_multiplier: f64,
    /// How long a telegraphed trait warns before it resolves.
    #[serde(default = "default_trait_telegraph_ms")]
    pub trait_telegraph_ms: f32,
    /// Every Nth combo pays a streak bonus.
    #[serde(default = "default_combo_milestone_interval")]
    pub combo_milestone_interval: u32,
    #[serde(default = "default_combo_milestone_cash")]
    pub combo_milestone_cash: i64,
    /// Renown per seated guest when every table's order is complete at once.
    #[serde(default = "default_full_room_bonus_points")]
    pub full_room_bonus_points: i64,
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

fn default_dish_fresh_window_ms() -> f32 {
    12_000.0
}

fn default_dish_spoil_ms() -> f32 {
    32_000.0
}

fn default_fresh_bill_bonus_multiplier() -> f64 {
    1.25
}

fn default_trait_telegraph_ms() -> f32 {
    5_000.0
}

fn default_combo_milestone_interval() -> u32 {
    5
}

fn default_combo_milestone_cash() -> i64 {
    12
}

fn default_full_room_bonus_points() -> i64 {
    40
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
            visits_until_ready_by_tier: vec![2, 3, 4, 5],
            dish_fresh_window_ms: default_dish_fresh_window_ms(),
            dish_spoil_ms: default_dish_spoil_ms(),
            fresh_bill_bonus_multiplier: default_fresh_bill_bonus_multiplier(),
            trait_telegraph_ms: default_trait_telegraph_ms(),
            combo_milestone_interval: default_combo_milestone_interval(),
            combo_milestone_cash: default_combo_milestone_cash(),
            full_room_bonus_points: default_full_room_bonus_points(),
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
    pub tutorial_steps: Vec<TutorialStep>,
    pub specializations: Vec<SpecializationDef>,
    pub trait_behaviors: Vec<TraitBehavior>,
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
            tutorial_steps: parse_or_fallback(TUTORIAL_JSON, "[]"),
            specializations: parse_or_fallback(SPECIALIZATIONS_JSON, "[]"),
            trait_behaviors: parse_or_fallback(TRAIT_BEHAVIORS_JSON, "[]"),
            balance: parse_or_fallback(GAME_BALANCE_JSON, "{}"),
        }
    }

    pub fn customer_type_by_id(&self, id: &str) -> Option<&CustomerType> {
        self.customer_types.iter().find(|item| item.id == id)
    }

    pub fn dish_type_by_color(&self, color: &str) -> Option<&DishType> {
        self.dish_types.iter().find(|item| item.color == color)
    }

    pub fn specialization_by_id(&self, id: &str) -> Option<&SpecializationDef> {
        self.specializations.iter().find(|item| item.id == id)
    }

    pub fn trait_behavior(&self, trait_key: &str) -> Option<&TraitBehavior> {
        self.trait_behaviors
            .iter()
            .find(|item| item.trait_key == trait_key)
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
            tutorial_steps: serde_json::from_str(TUTORIAL_JSON).expect("tutorial.json must parse"),
            specializations: serde_json::from_str(SPECIALIZATIONS_JSON)
                .expect("specializations.json must parse"),
            trait_behaviors: serde_json::from_str(TRAIT_BEHAVIORS_JSON)
                .expect("trait_behaviors.json must parse"),
            balance: serde_json::from_str(GAME_BALANCE_JSON).expect("game_balance.json must parse"),
        }
    }

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
        assert!(
            !balance.visits_until_ready_by_tier.is_empty(),
            "tier readiness ladder drives early pacing"
        );
        for window in balance.visits_until_ready_by_tier.windows(2) {
            assert!(
                window[0] <= window[1],
                "readiness ladder must not get cheaper at higher tiers"
            );
        }
        assert!(balance
            .visits_until_ready_by_tier
            .iter()
            .all(|visits| *visits >= 1));
    }

    #[test]
    fn specializations_are_real_tradeoffs_on_known_effect_keys() {
        let data = parsed();
        assert!(data.specializations.len() >= 3, "need a real choice");
        assert_unique_ids(
            "specialization",
            &data
                .specializations
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
        );
        for spec in &data.specializations {
            assert!(
                spec.effects.len() >= 2,
                "{}: a specialization with one effect is a buff, not a trade-off",
                spec.id
            );
            for key in spec.effects.keys() {
                assert!(
                    KNOWN_EFFECTS.contains(&key.as_str()),
                    "{}: effect key {key} is never read by the engine",
                    spec.id
                );
            }
        }
    }

    #[test]
    fn trait_behaviors_cover_every_special_trait_flag() {
        // One entry per bool on CustomerSpecialTraits, keyed by field name.
        const TRAIT_FLAGS: [&str; 8] = [
            "low_appetite",
            "can_wander",
            "multiplies_on_process",
            "fast_spoilage",
            "can_steal_food",
            "can_eat_waste",
            "high_yield",
            "throws_food",
        ];
        let data = parsed();
        for flag in TRAIT_FLAGS {
            let behavior = data
                .trait_behavior(flag)
                .unwrap_or_else(|| panic!("missing trait behavior for {flag}"));
            assert!(
                !behavior.name.is_empty() && !behavior.hint.is_empty(),
                "{flag}"
            );
            if behavior.telegraphed {
                assert!(
                    !behavior.telegraph.is_empty(),
                    "{flag}: telegraphed traits need telegraph text"
                );
            }
        }
        assert_unique_ids(
            "trait behavior",
            &data
                .trait_behaviors
                .iter()
                .map(|item| item.trait_key.as_str())
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn freshness_and_streak_tuning_is_sane() {
        let data = parsed();
        let balance = &data.balance;
        assert!(balance.dish_fresh_window_ms > 0.0);
        assert!(
            balance.dish_spoil_ms > balance.dish_fresh_window_ms,
            "dishes must go stale before they spoil"
        );
        assert!(balance.fresh_bill_bonus_multiplier >= 1.0);
        assert!(balance.trait_telegraph_ms > 0.0);
        assert!(balance.combo_milestone_interval >= 2);
        assert!(balance.combo_milestone_cash > 0);
        assert!(balance.full_room_bonus_points > 0);
    }

    #[test]
    fn tutorial_steps_are_present_and_end_with_acknowledged() {
        let data = parsed();
        assert!(
            data.tutorial_steps.len() >= 5,
            "tutorial must cover the full cook->serve->fatten->process loop"
        );
        assert_unique_ids(
            "tutorial step",
            &data
                .tutorial_steps
                .iter()
                .map(|step| step.id.as_str())
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            data.tutorial_steps.last().map(|step| step.trigger),
            Some(TutorialTrigger::Acknowledged),
            "final step must wait for the player to acknowledge it"
        );
        for step in &data.tutorial_steps {
            assert!(
                !step.title.is_empty() && !step.body.is_empty(),
                "{}",
                step.id
            );
        }
    }
}
