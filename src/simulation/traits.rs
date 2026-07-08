//! Special-trait ticks: wandering guests changing tables, foxes stealing from
//! the pass, monkeys throwing food when unhappy, and fast-spoilage decay.

use crate::data::GameData;
use crate::engine::{
    chance, max_customer_count, CAN_WANDER_CHANCE, FOX_STEAL_CHANCE, MONKEY_THROW_CHANCE,
};
use crate::gameplay::dish_display_name;
use crate::state::{GameState, ProgressionState, Timers};
use std::collections::HashSet;

pub(super) fn update_traits(
    data: &GameData,
    game_state: &mut GameState,
    timers: &mut Timers,
    progression: &ProgressionState,
) {
    let tick = data.balance.trait_tick_interval;
    if tick <= 0.0 || game_state.customers.is_empty() {
        timers.trait_accum_ms = 0.0;
        return;
    }

    let max_tables = max_customer_count(data, progression);
    let mut occupied = occupied_tables(game_state, max_tables);
    let trait_tick = tick;
    while timers.trait_accum_ms >= trait_tick {
        timers.trait_accum_ms -= trait_tick;
        for index in 0..game_state.customers.len() {
            apply_customer_traits(index, data, game_state, max_tables, &mut occupied);
        }
    }
}

fn apply_customer_traits(
    index: usize,
    data: &GameData,
    game_state: &mut GameState,
    max_tables: usize,
    occupied: &mut HashSet<usize>,
) {
    let traits = game_state.customers[index].traits(data);
    if traits.can_wander && chance(CAN_WANDER_CHANCE) && max_tables > 0 {
        move_customer_to_empty_table(index, game_state, max_tables, occupied);
    }

    if traits.can_steal_food && chance(FOX_STEAL_CHANCE) {
        if let Some((station_color, dish_name)) = steal_or_throw_dish(game_state) {
            let station_name = dish_display_name(data, &station_color);
            game_state.add_message(format!(
                "{} stole {dish_name} from {station_name}",
                game_state.customers[index].display_name
            ));
        }
    }

    if traits.throws_food
        && chance(MONKEY_THROW_CHANCE)
        && game_state.customers[index].total_satisfaction < 60.0
        && game_state.customers[index].total_satisfaction > 0.0
    {
        if let Some((station_color, dish_name)) = steal_or_throw_dish(game_state) {
            let display_name = &game_state.customers[index].display_name;
            let station_name = dish_display_name(data, &station_color);
            game_state.add_message(format!(
                "{display_name} threw away {dish_name} from {station_name}"
            ));
            game_state.combo = 0;
        }
    }

    if traits.fast_spoilage {
        game_state.customers[index].satisfaction.decay_all(2.0);
        game_state.customers[index].refresh_totals();
    }
}

fn move_customer_to_empty_table(
    index: usize,
    game_state: &mut GameState,
    max_tables: usize,
    occupied: &mut HashSet<usize>,
) {
    let empty_tables: Vec<usize> = (0..max_tables)
        .filter(|idx| !occupied.contains(idx))
        .collect();
    let Some(next) = macroquad_toolkit::rng::choose(&empty_tables).copied() else {
        return;
    };
    let Some(previous) = game_state
        .customers
        .get(index)
        .map(|customer| customer.table_index)
    else {
        return;
    };
    if previous >= max_tables {
        return;
    }

    occupied.remove(&previous);
    occupied.insert(next);
    let display_name = {
        let customer = &mut game_state.customers[index];
        customer.table_index = next;
        customer.is_seated = false;
        customer.display_name.clone()
    };
    game_state.add_message(format!("{display_name} moved to table {}", next + 1));
}

fn occupied_tables(game_state: &GameState, max_tables: usize) -> HashSet<usize> {
    game_state
        .customers
        .iter()
        .filter_map(|customer| (customer.table_index < max_tables).then_some(customer.table_index))
        .collect()
}

fn steal_or_throw_dish(game_state: &mut GameState) -> Option<(String, String)> {
    let candidates: Vec<String> = game_state
        .cooking_stations
        .iter()
        .filter_map(|(color, station)| {
            if station.dishes.is_empty() {
                None
            } else {
                Some(color.clone())
            }
        })
        .collect();

    let station_color = macroquad_toolkit::rng::choose(&candidates).cloned()?;

    let station = game_state.cooking_stations.get_mut(&station_color)?;
    if station.dishes.is_empty() {
        return None;
    }
    let dish = station.dishes.remove(0);
    Some((station_color, dish))
}
