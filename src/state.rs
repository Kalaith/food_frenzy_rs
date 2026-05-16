use crate::data::{Achievement, CustomerSpecialTraits, GameData, Recipe, Upgrade};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const INFINITE_INGREDIENTS: i64 = -1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Satisfaction {
    pub blue: f32,
    pub green: f32,
    pub yellow: f32,
    pub red: f32,
}

impl Satisfaction {
    pub fn total(&self) -> f32 {
        self.blue + self.green + self.yellow + self.red
    }

    pub fn get(&self, color: &str) -> Option<f32> {
        match color {
            "blue" => Some(self.blue),
            "green" => Some(self.green),
            "yellow" => Some(self.yellow),
            "red" => Some(self.red),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, color: &str) -> Option<&mut f32> {
        match color {
            "blue" => Some(&mut self.blue),
            "green" => Some(&mut self.green),
            "yellow" => Some(&mut self.yellow),
            "red" => Some(&mut self.red),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookingStation {
    pub color: String,
    pub is_cooking: bool,
    pub remaining_ms: f32,
    pub dishes: Vec<String>,
}

impl CookingStation {
    pub fn new(color: String) -> Self {
        Self {
            color,
            is_cooking: false,
            remaining_ms: 0.0,
            dishes: Vec::new(),
        }
    }

    pub fn can_cook(&self, max_ready: usize) -> bool {
        !self.is_cooking && self.dishes.len() < max_ready
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerActor {
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    #[serde(default)]
    pub carried_station: Option<String>,
    #[serde(default)]
    pub task_label: String,
    #[serde(default)]
    pub clear_carry_on_arrival: bool,
    #[serde(default)]
    pub action_lock_ms: f32,
    #[serde(default)]
    pub lock_on_arrival_ms: f32,
}

impl Default for PlayerActor {
    fn default() -> Self {
        Self {
            x: -235.0,
            y: 104.0,
            target_x: -235.0,
            target_y: 104.0,
            carried_station: None,
            task_label: "Prep".to_string(),
            clear_carry_on_arrival: false,
            action_lock_ms: 0.0,
            lock_on_arrival_ms: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: u32,
    pub guest_id: String,
    pub display_name: String,
    pub customer_type: String,
    pub satisfaction: Satisfaction,
    pub max_satisfaction: Satisfaction,
    pub deliciousness: f32,
    pub total_satisfaction: f32,
    pub overfed: bool,
    pub table_index: usize,
    pub arrived_at_ms: f64,
    #[serde(default)]
    pub floor_x: f32,
    #[serde(default)]
    pub floor_y: f32,
    #[serde(default)]
    pub target_x: f32,
    #[serde(default)]
    pub target_y: f32,
    #[serde(default)]
    pub is_seated: bool,
}

impl Customer {
    pub fn traits<'a>(&self, data: &'a GameData) -> CustomerSpecialTraits {
        data.customer_type_by_id(&self.customer_type)
            .and_then(|customer_type| customer_type.special_traits.clone())
            .unwrap_or_default()
    }

    pub fn refresh_totals(&mut self) {
        self.total_satisfaction = self.satisfaction.total();
        self.overfed = self.total_satisfaction
            > self.max_satisfaction.blue
                + self.max_satisfaction.green
                + self.max_satisfaction.yellow
                + self.max_satisfaction.red;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestRecord {
    pub id: String,
    pub name: String,
    pub customer_type: String,
    pub visits: u32,
    pub feedings: u32,
    pub processed_count: u32,
    pub last_seen_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestState {
    pub guests: Vec<GuestRecord>,
}

impl GuestState {
    pub fn new() -> Self {
        Self { guests: Vec::new() }
    }

    fn random_id() -> String {
        format!("guest-{}", macroquad::rand::gen_range(0, 999_999))
    }

    pub fn create_guest(&mut self, name: &str, customer_type: &str) -> GuestRecord {
        let now = macroquad::time::get_time() as u64;
        let guest = GuestRecord {
            id: Self::random_id(),
            name: name.to_string(),
            customer_type: customer_type.to_string(),
            visits: 0,
            feedings: 0,
            processed_count: 0,
            last_seen_at: now,
        };

        self.guests.push(guest.clone());
        guest
    }

    pub fn get_returning_unlocked_guest(
        &self,
        unlocked_customer_types: &[String],
        excluded_ids: &[String],
    ) -> Option<GuestRecord> {
        let excluded: HashSet<_> = excluded_ids.iter().collect();
        let unlocked: HashSet<_> = unlocked_customer_types.iter().collect();
        let candidates: Vec<&GuestRecord> = self
            .guests
            .iter()
            .filter(|guest| {
                guest.feedings > 0
                    && !excluded.contains(&guest.id)
                    && unlocked.contains(&guest.customer_type)
            })
            .collect();

        crate::engine::random_index(candidates.len()).map(|index| candidates[index].clone())
    }

    pub fn record_guest_visit(&mut self, guest_id: &str) {
        if let Some(guest) = self.guests.iter_mut().find(|guest| guest.id == guest_id) {
            guest.visits = guest.visits.saturating_add(1);
            guest.last_seen_at = macroquad::time::get_time() as u64;
        }
    }

    pub fn record_guest_fed(&mut self, guest_id: &str) {
        if let Some(guest) = self.guests.iter_mut().find(|guest| guest.id == guest_id) {
            guest.feedings = guest.feedings.saturating_add(1);
            guest.last_seen_at = macroquad::time::get_time() as u64;
        }
    }

    pub fn record_guest_processed(&mut self, guest_id: &str) {
        if let Some(guest) = self.guests.iter_mut().find(|guest| guest.id == guest_id) {
            guest.processed_count = guest.processed_count.saturating_add(1);
            guest.last_seen_at = macroquad::time::get_time() as u64;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionState {
    pub currency: i64,
    pub upgrades: Vec<Upgrade>,
    pub recipes: Vec<Recipe>,
    pub achievements: Vec<Achievement>,
    pub prestige_level: u32,
    pub prestige_points: i32,
    pub total_score: i64,
    pub processed_customer_counts: HashMap<String, u32>,
    pub processed_customer_types: Vec<String>,
    pub feeding_capacity_bonus: i64,
    pub crafted_recipe_counts: HashMap<String, u32>,
    pub total_dishes_served: i64,
    pub preferred_dishes_served: i64,
    pub overfed_customer_count: i64,
    pub customers_lost: i64,
    #[serde(default)]
    pub unlocked_customer_types: Vec<String>,
}

impl ProgressionState {
    pub fn from_game_data(data: &crate::data::GameData) -> Self {
        let mut state = Self {
            currency: 0,
            upgrades: data.upgrades.clone(),
            recipes: data.recipes.clone(),
            achievements: data.achievements.clone(),
            prestige_level: 0,
            prestige_points: 0,
            total_score: 0,
            processed_customer_counts: HashMap::new(),
            processed_customer_types: Vec::new(),
            feeding_capacity_bonus: 0,
            crafted_recipe_counts: HashMap::new(),
            total_dishes_served: 0,
            preferred_dishes_served: 0,
            overfed_customer_count: 0,
            customers_lost: 0,
            unlocked_customer_types: starting_customer_type_ids(data),
        };

        state.ensure_customer_unlocks(data);
        state.set_upgrade_costs();
        state
    }

    pub fn ensure_customer_unlocks(&mut self, data: &crate::data::GameData) {
        for customer_type in &data.customer_types {
            if customer_type.initially_unlocked
                && !self
                    .unlocked_customer_types
                    .iter()
                    .any(|item| item == &customer_type.id)
            {
                self.unlocked_customer_types.push(customer_type.id.clone());
            }
        }

        if self.unlocked_customer_types.is_empty() {
            if let Some(customer_type) = data.customer_types.first() {
                self.unlocked_customer_types.push(customer_type.id.clone());
            }
        }
    }

    pub fn is_customer_unlocked(&self, customer_type_id: &str) -> bool {
        self.unlocked_customer_types
            .iter()
            .any(|item| item == customer_type_id)
    }

    pub fn unlocked_customer_count(&self) -> usize {
        self.unlocked_customer_types.len()
    }

    pub fn unlock_customer_type(&mut self, customer_type_id: &str) -> bool {
        if self.is_customer_unlocked(customer_type_id) {
            return false;
        }

        self.unlocked_customer_types
            .push(customer_type_id.to_string());
        true
    }

    pub fn add_currency(&mut self, amount: i64) {
        if amount > 0 {
            self.currency = self.currency.saturating_add(amount);
        }
    }

    pub fn spend_currency(&mut self, amount: i64) -> bool {
        if self.currency < amount {
            return false;
        }

        self.currency -= amount;
        true
    }

    pub fn get_effect(&self, key: &str, fallback: f64) -> f64 {
        let mut total = fallback;
        for upgrade in &self.upgrades {
            if let Some(value) = upgrade.effects.get(key) {
                let level = upgrade.level as f64;
                total += value * level;
            }
        }

        if (fallback - 1.0).abs() < f64::EPSILON {
            total.max(0.25)
        } else {
            total
        }
    }

    pub fn set_upgrade_costs(&mut self) {
        for upgrade in &mut self.upgrades {
            let level = upgrade.level as f64;
            upgrade.cost = (((upgrade.base_cost as f64) * upgrade.cost_growth.powf(level)).ceil()
                as i64)
                .max(1);
        }
    }

    pub fn buy_upgrade(&mut self, upgrade_id: &str) -> bool {
        let Some(index) = self.upgrades.iter().position(|item| item.id == upgrade_id) else {
            return false;
        };

        if self.upgrades[index].level >= self.upgrades[index].max_level {
            return false;
        }

        let cost = self.upgrades[index].cost;
        if !self.spend_currency(cost) {
            return false;
        }

        let upgrade = &mut self.upgrades[index];
        upgrade.level = upgrade.level.saturating_add(1);
        if upgrade.level >= upgrade.max_level {
            upgrade.purchased = true;
            upgrade.level = upgrade.max_level;
        }
        self.set_upgrade_costs();
        true
    }

    pub fn record_score(&mut self, amount: i64) {
        let scored = amount.max(0);
        self.total_score = self.total_score.saturating_add(scored);
        self.update_achievement("busy-night", self.total_score);
        self.update_achievement("restaurant-empire", self.total_score);
    }

    pub fn record_served_dish(&mut self, is_preferred: bool, is_overfed: bool) {
        self.total_dishes_served = self.total_dishes_served.saturating_add(1);
        if is_preferred {
            self.preferred_dishes_served = self.preferred_dishes_served.saturating_add(1);
        }
        if is_overfed {
            self.overfed_customer_count = self.overfed_customer_count.saturating_add(1);
        }

        self.update_achievement("steady-service", self.total_dishes_served);
        self.update_achievement("favorite-service", self.preferred_dishes_served);
        self.update_achievement("overfed-specialist", self.overfed_customer_count);
    }

    pub fn record_processed_customer(&mut self, customer_type: &str, chain_length: u32) {
        let count = self
            .processed_customer_counts
            .entry(customer_type.to_string())
            .or_insert(0);
        *count = count.saturating_add(1);

        if !self
            .processed_customer_types
            .iter()
            .any(|item| item == customer_type)
        {
            self.processed_customer_types
                .push(customer_type.to_string());
        }

        self.update_recipe_unlocks();
        self.update_achievement("first-customer", 1);
        self.update_achievement("combo-master", i64::from(chain_length));
        self.update_achievement(
            "broad-menu",
            i64::try_from(self.processed_customer_types.len()).unwrap_or(0),
        );
    }

    fn update_recipe_unlocks(&mut self) {
        let base_types = ["pig", "cow", "sheep", "rabbit", "cat"];
        for recipe in &mut self.recipes {
            match recipe.id.as_str() {
                "bacon-ramen" => {
                    if self
                        .processed_customer_counts
                        .get("pig")
                        .copied()
                        .unwrap_or(0)
                        >= 5
                    {
                        recipe.unlocked = true;
                    }
                }
                "golden-cutlets" => {
                    if self
                        .processed_customer_counts
                        .get("chicken")
                        .copied()
                        .unwrap_or(0)
                        >= 3
                    {
                        recipe.unlocked = true;
                    }
                }
                "tidal-platter" => {
                    if self
                        .processed_customer_counts
                        .get("fish")
                        .copied()
                        .unwrap_or(0)
                        >= 3
                    {
                        recipe.unlocked = true;
                    }
                }
                "street-skewers" => {
                    if self
                        .processed_customer_counts
                        .get("fox")
                        .copied()
                        .unwrap_or(0)
                        >= 3
                    {
                        recipe.unlocked = true;
                    }
                }
                "honey-roast-feast" => {
                    if self
                        .processed_customer_counts
                        .get("bear")
                        .copied()
                        .unwrap_or(0)
                        >= 2
                    {
                        recipe.unlocked = true;
                    }
                }
                "rainbow-stew" => {
                    if base_types.iter().all(|ctype| {
                        self.processed_customer_types
                            .iter()
                            .any(|seen| seen == ctype)
                    }) {
                        recipe.unlocked = true;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn record_crafted_recipe(&mut self, recipe_id: &str, capacity_bonus: i64) {
        self.feeding_capacity_bonus = (self.feeding_capacity_bonus + capacity_bonus.max(0))
            .min(self.achievements_maximum_capacity_bonus());

        let next = self
            .crafted_recipe_counts
            .entry(recipe_id.to_string())
            .or_insert(0);
        *next = next.saturating_add(1);

        let total_crafted = self
            .crafted_recipe_counts
            .values()
            .map(|count| i64::from(*count))
            .sum::<i64>();

        self.update_achievement("recipe-merchant", total_crafted);
        self.update_achievement("capacity-planner", self.feeding_capacity_bonus);
    }

    fn achievements_maximum_capacity_bonus(&self) -> i64 {
        80
    }

    pub fn record_customer_lost(&mut self) {
        self.customers_lost = self.customers_lost.saturating_add(1);
    }

    fn update_achievement(&mut self, id: &str, progress: i64) {
        if let Some(achievement) = self.achievements.iter_mut().find(|item| item.id == id) {
            if achievement.progress < progress {
                achievement.progress = progress.min(achievement.max_progress);
            }

            if achievement.progress >= achievement.max_progress {
                achievement.unlocked = true;
            }
        }
    }

    pub fn can_prestige(&self, requirement: i64) -> bool {
        self.total_score >= requirement
    }

    pub fn prestige_reward(&self) -> i64 {
        let score_reward = (self.total_score / 10_000).max(0);
        let achievement_reward = self
            .achievements
            .iter()
            .filter(|item| item.unlocked)
            .count();
        let capacity_reward = (self.feeding_capacity_bonus / 20).max(0);
        std::cmp::max(
            1,
            i64::try_from(score_reward + achievement_reward as i64 + capacity_reward).unwrap_or(1),
        )
    }

    pub fn prestige(&mut self, data: &crate::data::GameData) {
        let reward = self.prestige_reward();

        self.currency = self.currency.saturating_add(reward);
        self.prestige_level = self.prestige_level.saturating_add(1);
        self.prestige_points = self
            .prestige_points
            .saturating_add(i32::try_from(reward).unwrap_or(i32::MAX));

        self.total_score = 0;
        self.processed_customer_counts.clear();
        self.processed_customer_types.clear();
        self.feeding_capacity_bonus = 0;
        self.crafted_recipe_counts.clear();
        self.total_dishes_served = 0;
        self.preferred_dishes_served = 0;
        self.overfed_customer_count = 0;
        self.customers_lost = 0;
        self.unlocked_customer_types = starting_customer_type_ids(data);

        self.upgrades = data.upgrades.clone();
        self.recipes = data.recipes.clone();
        self.set_upgrade_costs();

        for achievement in &mut self.achievements {
            achievement.unlocked = false;
            achievement.progress = 0;
        }

        if let Some(item) = self
            .achievements
            .iter_mut()
            .find(|item| item.id == "fresh-start")
        {
            item.unlocked = true;
            item.progress = item.max_progress;
        }
    }
}

fn starting_customer_type_ids(data: &crate::data::GameData) -> Vec<String> {
    let mut ids: Vec<String> = data
        .customer_types
        .iter()
        .filter(|customer_type| customer_type.initially_unlocked)
        .map(|customer_type| customer_type.id.clone())
        .collect();

    if ids.is_empty() {
        if let Some(customer_type) = data.customer_types.first() {
            ids.push(customer_type.id.clone());
        }
    }

    ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub score: i64,
    pub combo: u32,
    pub chain: u32,
    pub customers: Vec<Customer>,
    pub ingredients: HashMap<String, i64>,
    pub cooking_stations: HashMap<String, CookingStation>,
    pub special_table_busy: bool,
    pub special_table_timer: f32,
    pub messages: Vec<String>,
    pub next_customer_id: u32,
    #[serde(default)]
    pub player: PlayerActor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timers {
    pub elapsed_ms: f64,
    pub next_spawn_ms: f64,
    pub spawn_step: usize,
    pub decay_accum_ms: f32,
    pub patience_accum_ms: f32,
    pub trait_accum_ms: f32,
    pub save_accum_ms: f32,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            elapsed_ms: 0.0,
            next_spawn_ms: 0.0,
            spawn_step: 0,
            decay_accum_ms: 0.0,
            patience_accum_ms: 0.0,
            trait_accum_ms: 0.0,
            save_accum_ms: 0.0,
        }
    }
}

impl GameState {
    pub fn new(data: &crate::data::GameData) -> Self {
        let mut stations = HashMap::new();
        for dish in &data.dish_types {
            stations.insert(dish.color.clone(), CookingStation::new(dish.color.clone()));
        }

        let mut ingredients = HashMap::new();
        ingredients.insert(
            data.balance.regular_ingredient_name.clone(),
            INFINITE_INGREDIENTS,
        );

        Self {
            score: 0,
            combo: 0,
            chain: 0,
            customers: Vec::new(),
            ingredients,
            cooking_stations: stations,
            special_table_busy: false,
            special_table_timer: 0.0,
            messages: vec!["Welcome to Feast Frenzy!".to_string()],
            next_customer_id: 1,
            player: PlayerActor::default(),
        }
    }

    pub fn next_customer_id(&mut self) -> u32 {
        let id = self.next_customer_id;
        self.next_customer_id = self.next_customer_id.saturating_add(1);
        id
    }

    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
        if self.messages.len() > 18 {
            self.messages.drain(0..(self.messages.len() - 18));
        }
    }

    pub fn station_mut(&mut self, color: &str) -> Option<&mut CookingStation> {
        self.cooking_stations.get_mut(color)
    }

    pub fn has_ing(&self, key: &str, amount: i64) -> bool {
        if key == "regular" {
            return true;
        }

        self.ingredients
            .get(key)
            .is_some_and(|value| *value >= amount)
    }

    pub fn remove_ingredients(&mut self, key: &str, amount: i64) -> bool {
        if !self.has_ing(key, amount) {
            return false;
        }

        let current = self.ingredients.entry(key.to_string()).or_insert(0);
        if *current != INFINITE_INGREDIENTS {
            *current = current.saturating_sub(amount);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;

    #[test]
    fn progression_starts_with_basic_clientele_only() {
        let data = GameData::load();
        let progression = ProgressionState::from_game_data(&data);

        assert!(progression.is_customer_unlocked("pig"));
        assert!(progression.is_customer_unlocked("sheep"));
        assert!(progression.is_customer_unlocked("rabbit"));
        assert!(!progression.is_customer_unlocked("bear"));
    }

    #[test]
    fn customer_type_unlocks_are_persistent_progression() {
        let data = GameData::load();
        let mut progression = ProgressionState::from_game_data(&data);

        assert!(progression.unlock_customer_type("cow"));
        assert!(progression.is_customer_unlocked("cow"));
        assert!(!progression.unlock_customer_type("cow"));
    }
}
