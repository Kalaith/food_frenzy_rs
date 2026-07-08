//! Core gameplay helpers used by the Macroquad simulation loop.

use crate::data::{CustomerSpecialTraits, DishType, GameBalance, GameData};
use crate::state::{Course, Customer, ProgressionState, Satisfaction};

pub const RETURNING_GUEST_CHANCE: f64 = 0.65;
pub const FOX_STEAL_CHANCE: f64 = 0.35;
pub const CAN_WANDER_CHANCE: f64 = 0.25;
pub const MONKEY_THROW_CHANCE: f64 = 0.30;
pub const VIP_ACCEPT_CHANCE: f64 = 0.85;
pub const RESTAURANT_FLOOR_WIDTH: f32 = 1_000.0;
pub const RESTAURANT_FLOOR_HEIGHT: f32 = 620.0;
pub const CUSTOMER_WALK_SPEED: f32 = 190.0;
pub const PLAYER_WALK_SPEED: f32 = 340.0;
pub const KITCHEN_SERVICE_LEFT: f32 = -260.0;

pub fn max_customer_count(data: &GameData, progression: &ProgressionState) -> usize {
    let bonus = progression.get_effect("max_customers_bonus", 0.0);
    let total = (f64::from(data.balance.max_customers) + bonus).max(1.0);
    total as usize
}

pub fn spawn_interval_ms(data: &GameData, progression: &ProgressionState) -> f32 {
    (data.balance.customer_spawn_interval
        * progression.get_effect("spawn_interval_multiplier", 1.0) as f32)
        .max(5_000.0)
}

pub fn cooking_time_ms(data: &GameData, progression: &ProgressionState, color: &str) -> f32 {
    let dish = data.dish_type_by_color(color);
    if dish.is_none() {
        return 1_000.0;
    }

    let base = dish.unwrap().cook_time_ms;
    let multiplier = progression
        .get_effect("cook_time_multiplier", 1.0)
        .max(0.25);
    (base * multiplier as f32).max(250.0)
}

pub fn max_satisfaction_for_customer(
    data: &GameBalance,
    traits: &CustomerSpecialTraits,
    capacity_bonus: i64,
) -> Satisfaction {
    let mut base = data.max_satisfaction_per_type + (capacity_bonus as f32);
    if traits.low_appetite {
        base *= 0.7;
    }
    if traits.high_yield {
        base *= 1.5;
    }
    let per_slot = base.max(1.0).floor();

    Satisfaction {
        blue: per_slot,
        green: per_slot,
        yellow: per_slot,
        red: per_slot,
    }
}

pub fn satisfaction_decay_rate(data: &GameData, progression: &ProgressionState) -> f32 {
    (data.balance.satisfaction_decay_rate
        * progression.get_effect("satisfaction_decay_multiplier", 1.0) as f32)
        .max(0.05)
}

pub fn patience_multiplier(progression: &ProgressionState) -> f32 {
    progression.get_effect("patience_multiplier", 1.0) as f32
}

pub fn dish_is_preferred(data: &GameData, customer_type: &str, color: &str) -> bool {
    let Some(customer_type) = data.customer_type_by_id(customer_type) else {
        return false;
    };

    customer_type
        .preferred_dishes
        .iter()
        .any(|value| value == color)
}

pub fn serving_gain(
    data: &GameData,
    customer_type: &str,
    traits: &CustomerSpecialTraits,
    dish_color: &str,
) -> (f32, f32, bool) {
    let preferred = dish_is_preferred(data, customer_type, dish_color);
    if preferred {
        (data.balance.preferred_satisfaction_gain, 1.0, true)
    } else if traits.can_eat_waste {
        (data.balance.preferred_satisfaction_gain - 2.0, 1.0, false)
    } else {
        (data.balance.base_satisfaction_gain, 0.0, false)
    }
}

pub fn overfeed_multiplier(data: &GameData, traits: &CustomerSpecialTraits) -> f32 {
    if traits.low_appetite {
        data.balance.overfeed_multiplier_for_low_appetite
    } else {
        data.balance.overfeed_multiplier
    }
}

/// A guest is ready for the Last Meal Lounge once they have been fully served
/// on enough prior visits (fattened up over time), not from a single sitting.
pub fn can_process_customer(customer: &Customer, data: &GameData) -> bool {
    customer.times_fed >= data.balance.visits_until_ready
}

/// Build a guest's order for this visit: 1-3 distinct courses, favouring the
/// dishes the customer type prefers, then filling from the rest of the menu.
pub fn roll_order(data: &GameData, customer_type_id: &str) -> Vec<Course> {
    let mut colors: Vec<String> = Vec::new();
    if let Some(customer_type) = data.customer_type_by_id(customer_type_id) {
        let mut preferred = customer_type.preferred_dishes.clone();
        shuffle(&mut preferred);
        colors.extend(preferred);
    }
    let mut rest: Vec<String> = data
        .dish_types
        .iter()
        .map(|dish| dish.color.clone())
        .filter(|color| !colors.contains(color))
        .collect();
    shuffle(&mut rest);
    colors.extend(rest);

    let min = data.balance.min_courses.max(1) as i32;
    let max = (data.balance.max_courses as i32).max(min);
    let available = (colors.len() as i32).max(1);
    let upper = max.min(available);
    let count = if upper <= min {
        upper
    } else {
        macroquad_toolkit::rng::gen_range(min, upper + 1)
    };

    let chosen: Vec<String> = colors.into_iter().take(count.max(1) as usize).collect();
    let total = chosen.len();
    chosen
        .into_iter()
        .enumerate()
        .map(|(index, color)| Course {
            color,
            label: course_label(index, total).to_string(),
            served: false,
        })
        .collect()
}

fn course_label(index: usize, total: usize) -> &'static str {
    match (index, total) {
        (_, 1) => "Main",
        (0, 2) => "Entrée",
        (_, 2) => "Dessert",
        (0, _) => "Entrée",
        (1, _) => "Main",
        _ => "Dessert",
    }
}

fn shuffle<T>(items: &mut [T]) {
    let len = items.len();
    if len < 2 {
        return;
    }
    for i in (1..len).rev() {
        let j = macroquad_toolkit::rng::gen_range(0i32, (i + 1) as i32) as usize;
        items.swap(i, j);
    }
}

pub fn serving_points(data: &GameData, satisfaction_gain: f32, preferred: bool) -> i64 {
    let mut total = satisfaction_gain as f64;
    if preferred {
        total *= data.balance.preferred_dish_score_multiplier;
    } else {
        total *= data.balance.base_score_multiplier;
    }

    total.max(0.0).floor() as i64
}

/// Cash a single served dish adds to the guest's tab. Preferred dishes are worth
/// more, so serving what a guest actually craves pays off.
pub fn serving_bill(data: &GameData, preferred: bool) -> i64 {
    let base = data.balance.dish_bill_value.max(0) as f64;
    let value = if preferred {
        base * data.balance.preferred_bill_multiplier.max(0.0)
    } else {
        base
    };
    value.floor().max(0.0) as i64
}

/// Bonus tip paid on top of the tab when a guest leaves fully satisfied.
pub fn satisfied_tip(data: &GameData, bill: i64) -> i64 {
    let tip = (bill as f64) * data.balance.satisfied_tip_rate.max(0.0);
    tip.floor().max(0.0) as i64
}

pub fn vip_meat_gain(customer: &Customer, data: &GameData, progression: &ProgressionState) -> i64 {
    let traits = customer.traits(data);
    // Yield scales with how plump the guest got over their visits, plus a little
    // for the flavour built up from preferred dishes this sitting.
    let base = customer.times_fed as f32 + customer.deliciousness.floor();
    let bonus = if traits.multiplies_on_process {
        2.0
    } else {
        0.0
    };
    let trait_multiplier = if traits.high_yield { 1.35 } else { 1.0 };
    let yield_multiplier =
        progression.get_effect("meat_yield_multiplier", 1.0) as f32 * trait_multiplier;
    let produced = (base * yield_multiplier).floor() + bonus;

    if produced <= 0.0 {
        1
    } else {
        produced as i64
    }
}

pub fn vip_points(customer: &Customer, data: &GameData) -> i64 {
    (data.balance.vip_points_per_deliciousness * f64::from(customer.deliciousness)).floor() as i64
}

pub fn recipe_value_multiplier(progression: &ProgressionState) -> f64 {
    progression.get_effect("recipe_value_multiplier", 1.0)
}

pub fn recipe_capacity_gain(progression: &ProgressionState, base_bonus: i64) -> i64 {
    let capacity = progression.get_effect("capacity_gain_multiplier", 1.0);
    let adjusted = ((base_bonus as f64) * capacity).floor();
    adjusted.max(0.0) as i64
}

pub fn restaurant_entrance_position() -> (f32, f32) {
    (70.0, RESTAURANT_FLOOR_HEIGHT - 78.0)
}

pub fn kitchen_station_position(color: &str) -> (f32, f32) {
    let x = match color {
        "blue" => -218.0,
        "green" => -158.0,
        "yellow" => -98.0,
        "red" => -38.0,
        _ => -218.0,
    };
    (x, 58.0)
}

pub fn kitchen_pass_position() -> (f32, f32) {
    (-235.0, 104.0)
}

pub fn restaurant_table_position(table_index: usize, max_tables: usize) -> (f32, f32) {
    let max_tables = max_tables.max(1);
    let columns = if max_tables <= 4 { 2 } else { 3 };
    let rows = max_tables.div_ceil(columns).max(1);
    let column = table_index % columns;
    let row = table_index / columns;
    let usable_w = RESTAURANT_FLOOR_WIDTH - 300.0;
    let usable_h = RESTAURANT_FLOOR_HEIGHT - 260.0;
    let spacing_x = if columns <= 1 {
        0.0
    } else {
        usable_w / ((columns - 1) as f32)
    };
    let spacing_y = if rows <= 1 {
        0.0
    } else {
        usable_h / ((rows - 1) as f32)
    };

    (
        170.0 + spacing_x * column as f32,
        145.0 + spacing_y * row as f32,
    )
}

pub fn chance(probability: f64) -> bool {
    macroquad_toolkit::rng::chance(probability as f32)
}

pub fn random_dish_name(dish: &DishType) -> String {
    macroquad_toolkit::rng::choose(&dish.examples)
        .cloned()
        .unwrap_or_else(|| dish.name.clone())
}
