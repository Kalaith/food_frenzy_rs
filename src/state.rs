use crate::data::{Achievement, CustomerSpecialTraits, GameData, Recipe, Upgrade};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod progression;

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

    pub fn decay_all(&mut self, amount: f32) {
        for value in [
            &mut self.blue,
            &mut self.green,
            &mut self.yellow,
            &mut self.red,
        ] {
            *value = (*value - amount).max(0.0);
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
    pub fn traits(&self, data: &GameData) -> CustomerSpecialTraits {
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
        format!("guest-{}", macroquad_toolkit::rng::gen_range(0, 999_999))
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

        macroquad_toolkit::rng::choose(&candidates).map(|guest| (**guest).clone())
    }

    pub fn record_guest_visit(&mut self, guest_id: &str) {
        self.record_guest_touch(guest_id, |guest| {
            guest.visits = guest.visits.saturating_add(1);
        });
    }

    pub fn record_guest_fed(&mut self, guest_id: &str) {
        self.record_guest_touch(guest_id, |guest| {
            guest.feedings = guest.feedings.saturating_add(1);
        });
    }

    pub fn record_guest_processed(&mut self, guest_id: &str) {
        self.record_guest_touch(guest_id, |guest| {
            guest.processed_count = guest.processed_count.saturating_add(1);
        });
    }

    fn record_guest_touch(&mut self, guest_id: &str, update: impl FnOnce(&mut GuestRecord)) {
        if let Some(guest) = self.guests.iter_mut().find(|guest| guest.id == guest_id) {
            update(guest);
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
            messages: vec!["Service started.".to_string()],
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
