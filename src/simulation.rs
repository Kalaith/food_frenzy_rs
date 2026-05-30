use crate::data::GameData;
use crate::engine::{
    chance, max_customer_count, max_satisfaction_for_customer, random_index,
    restaurant_entrance_position, restaurant_table_position, satisfaction_decay_rate,
    spawn_interval_ms, CAN_WANDER_CHANCE, CUSTOMER_WALK_SPEED, FOX_STEAL_CHANCE,
    MONKEY_THROW_CHANCE, RETURNING_GUEST_CHANCE,
};
use crate::gameplay::dish_display_name;
use crate::player::update_player_movement;
use crate::state::{Customer, GameState, GuestState, ProgressionState, Satisfaction, Timers};
use std::collections::HashSet;

pub fn update_game_world(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
) {
    timers.elapsed_ms += f64::from(dt_ms);
    let now_ms = timers.elapsed_ms;
    timers.patience_accum_ms += dt_ms;
    timers.decay_accum_ms += dt_ms;
    timers.trait_accum_ms += dt_ms;
    update_cooking(dt_ms, data, game_state);
    update_spawn(
        data,
        game_state,
        progression_state,
        guest_state,
        timers,
        now_ms,
    );
    update_customer_movement(dt_ms, data, progression_state, game_state);
    update_player_movement(dt_ms, game_state);
    update_patience(data, game_state, progression_state, timers, now_ms);
    update_satisfaction_decay(data, game_state, progression_state, timers);
    update_traits(data, game_state, timers, progression_state);
    update_lounge(dt_ms, game_state);
}

fn update_customer_movement(
    dt_ms: f32,
    data: &GameData,
    progression: &ProgressionState,
    game_state: &mut GameState,
) {
    let max_tables = max_customer_count(data, progression);
    let travel = CUSTOMER_WALK_SPEED * (dt_ms / 1000.0);
    for customer in &mut game_state.customers {
        let (target_x, target_y) = restaurant_table_position(customer.table_index, max_tables);
        customer.target_x = target_x;
        customer.target_y = target_y;

        let dx = target_x - customer.floor_x;
        let dy = target_y - customer.floor_y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= travel || distance <= 1.0 {
            customer.floor_x = target_x;
            customer.floor_y = target_y;
            customer.is_seated = true;
        } else {
            let step = travel / distance;
            customer.floor_x += dx * step;
            customer.floor_y += dy * step;
            customer.is_seated = false;
        }
    }
}

fn update_cooking(dt_ms: f32, data: &GameData, game_state: &mut GameState) {
    let mut messages = Vec::new();
    for station in game_state.cooking_stations.values_mut() {
        if !station.is_cooking {
            continue;
        }

        if station.remaining_ms <= dt_ms {
            station.is_cooking = false;
            station.remaining_ms = 0.0;
            if let Some(dish) = data.dish_type_by_color(&station.color) {
                let cooked_name = crate::engine::random_dish_name(dish);
                station.dishes.push(cooked_name.clone());
                messages.push(format!(
                    "{} ready: {cooked_name}",
                    dish_display_name(data, &station.color)
                ));
            }
        } else {
            station.remaining_ms -= dt_ms;
        }
    }

    for message in messages {
        game_state.add_message(message);
    }
}

fn update_spawn(
    data: &GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    now_ms: f64,
) {
    while timers.spawn_step < data.balance.initial_spawn_delays.len() {
        let initial_delay = data.balance.initial_spawn_delays[timers.spawn_step];
        if now_ms >= f64::from(initial_delay) {
            let _ = try_spawn_customer(data, game_state, progression, guest_state, now_ms);
            timers.spawn_step = timers.spawn_step.saturating_add(1);
        } else {
            break;
        }
    }

    let interval = f64::from(spawn_interval_ms(data, progression));
    if now_ms >= timers.next_spawn_ms {
        let _ = try_spawn_customer(data, game_state, progression, guest_state, now_ms);
        timers.next_spawn_ms = now_ms + interval;
    }
}

fn update_patience(
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    timers: &mut Timers,
    now_ms: f64,
) {
    if game_state.customers.is_empty() {
        timers.patience_accum_ms = 0.0;
        return;
    }

    while timers.patience_accum_ms >= 1_000.0_f32 {
        timers.patience_accum_ms -= 1_000.0;
        let removed_ids = impatient_customer_ids(data, game_state, progression, now_ms);
        if removed_ids.is_empty() {
            continue;
        }

        let before = game_state.customers.len();
        game_state
            .customers
            .retain(|customer| !removed_ids.contains(&customer.id));
        let lost = before.saturating_sub(game_state.customers.len());
        if lost > 0 {
            game_state.combo = 0;
            for _ in 0..lost {
                progression.record_customer_lost();
            }
            game_state.add_message(format!("{lost} customers left from impatience."));
        }
    }
}

fn update_satisfaction_decay(
    data: &GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    timers: &mut Timers,
) {
    let decay_interval = data.balance.satisfaction_decay_interval;
    if decay_interval <= 0.0 {
        return;
    }
    if game_state.customers.is_empty() {
        timers.decay_accum_ms = 0.0;
        return;
    }

    let decay_rate = satisfaction_decay_rate(data, progression);
    let decay_tick = decay_interval as f32;
    while timers.decay_accum_ms >= decay_tick {
        timers.decay_accum_ms -= decay_tick;
        for customer in &mut game_state.customers {
            customer.satisfaction.decay_all(decay_rate);
            customer.refresh_totals();
        }
    }
}

fn update_traits(
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
    let trait_tick = tick as f32;
    while timers.trait_accum_ms >= trait_tick {
        timers.trait_accum_ms -= trait_tick;
        for index in 0..game_state.customers.len() {
            apply_customer_traits(index, data, game_state, max_tables, &mut occupied);
        }
    }
}

fn try_spawn_customer(
    data: &GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    guest_state: &mut GuestState,
    now_ms: f64,
) -> bool {
    let max_customers = max_customer_count(data, progression);
    if game_state.customers.len() >= max_customers {
        return false;
    }

    let active_guest_ids: Vec<String> = game_state
        .customers
        .iter()
        .map(|customer| customer.guest_id.clone())
        .collect();
    let returning = if chance(RETURNING_GUEST_CHANCE) {
        guest_state
            .get_returning_unlocked_guest(&progression.unlocked_customer_types, &active_guest_ids)
    } else {
        None
    };

    let selected_type_id = returning
        .as_ref()
        .and_then(|guest| data.customer_type_by_id(&guest.customer_type))
        .cloned()
        .or_else(|| {
            let unlocked_customer_types: Vec<_> = data
                .customer_types
                .iter()
                .filter(|customer_type| progression.is_customer_unlocked(&customer_type.id))
                .collect();
            random_index(unlocked_customer_types.len())
                .map(|index| unlocked_customer_types[index].clone())
        });
    let Some(customer_type) = selected_type_id else {
        return false;
    };

    let guest_record = if let Some(guest) = returning {
        guest
    } else {
        let guest_name = random_guest_name();
        guest_state.create_guest(&guest_name, &customer_type.id)
    };

    let Some(table_index) = first_empty_table(game_state, max_customers) else {
        return false;
    };

    let traits = customer_type.special_traits.clone().unwrap_or_default();
    let max_satisfaction =
        max_satisfaction_for_customer(&data.balance, &traits, progression.feeding_capacity_bonus);
    let customer_id = game_state.next_customer_id();
    let (floor_x, floor_y) = restaurant_entrance_position();
    let (target_x, target_y) = restaurant_table_position(table_index, max_customers);
    game_state.customers.push(Customer {
        id: customer_id,
        guest_id: guest_record.id.clone(),
        display_name: guest_record.name.clone(),
        customer_type: customer_type.id.clone(),
        satisfaction: Satisfaction::default(),
        max_satisfaction,
        deliciousness: customer_type.base_deliciousness,
        total_satisfaction: 0.0,
        overfed: false,
        table_index,
        arrived_at_ms: now_ms,
        floor_x,
        floor_y,
        target_x,
        target_y,
        is_seated: false,
    });
    guest_state.record_guest_visit(&guest_record.id);
    game_state.add_message(format!(
        "{} arrived at table {}.",
        guest_record.name,
        table_index + 1
    ));

    true
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
    let Some(next) = random_index(empty_tables.len()).map(|idx| empty_tables[idx]) else {
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

fn impatient_customer_ids(
    data: &GameData,
    game_state: &GameState,
    progression: &ProgressionState,
    now_ms: f64,
) -> Vec<u32> {
    let patience_multiplier = crate::engine::patience_multiplier(progression);
    game_state
        .customers
        .iter()
        .filter_map(|customer| {
            let traits = customer.traits(data);
            let mut patience = data.balance.customer_patience_time * patience_multiplier;
            if traits.fast_spoilage {
                patience *= 0.55;
            }
            (now_ms - customer.arrived_at_ms > f64::from(patience)).then_some(customer.id)
        })
        .collect()
}

fn occupied_tables(game_state: &GameState, max_tables: usize) -> HashSet<usize> {
    game_state
        .customers
        .iter()
        .filter_map(|customer| (customer.table_index < max_tables).then_some(customer.table_index))
        .collect()
}

fn first_empty_table(game_state: &GameState, max_customers: usize) -> Option<usize> {
    (0..max_customers).find(|index| {
        game_state
            .customers
            .iter()
            .all(|customer| customer.table_index != *index)
    })
}

fn update_lounge(dt_ms: f32, game_state: &mut GameState) {
    if !game_state.special_table_busy {
        return;
    }
    if game_state.special_table_timer > 0.0 {
        game_state.special_table_timer -= dt_ms;
    }
    if game_state.special_table_timer <= 0.0 {
        game_state.special_table_busy = false;
        game_state.special_table_timer = 0.0;
    }
}

fn random_guest_name() -> String {
    const NAMES: &[&str] = &[
        "Marnie", "Lark", "Penny", "Bram", "Sora", "Rin", "Nova", "Haze", "Felix", "Tara", "Lune",
        "Milo", "Ari", "Violet",
    ];
    random_index(NAMES.len())
        .map(|index| NAMES[index])
        .unwrap_or("Guest")
        .to_string()
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

    let Some(station_color) = random_index(candidates.len()).map(|index| candidates[index].clone())
    else {
        return None;
    };

    let station = game_state.cooking_stations.get_mut(&station_color)?;
    if station.dishes.is_empty() {
        return None;
    }
    let dish = station.dishes.remove(0);
    Some((station_color, dish))
}
