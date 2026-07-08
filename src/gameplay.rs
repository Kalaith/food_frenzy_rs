use crate::data::{GameData, TutorialTrigger};
use crate::engine::{
    can_process_customer, cooking_time_ms, overfeed_multiplier, recipe_capacity_gain,
    recipe_value_multiplier, serving_bill, serving_gain, serving_points, vip_meat_gain, vip_points,
    visits_until_ready_for,
};
use crate::state::{
    FloaterAnchor, FloaterKind, GameState, GuestState, ProcessingCinematic, ProgressionState,
    INFINITE_INGREDIENTS,
};

pub fn dish_display_name(data: &GameData, color: &str) -> String {
    data.dish_type_by_color(color)
        .map(|dish| dish.name.clone())
        .unwrap_or_else(|| format!("{color} dish"))
}

pub fn try_prestige(
    data: &GameData,
    progression: &mut ProgressionState,
    game_state: &mut GameState,
) {
    if progression.can_prestige(data.balance.prestige_score_requirement) {
        progression.prestige(data);
        game_state.add_message(format!(
            "Prestige complete! +{} currency gained.",
            progression.prestige_level
        ));
        game_state.floaters.spawn(
            format!("Prestige {}!", progression.prestige_level),
            FloaterKind::Renown,
            FloaterAnchor::Header,
        );
    } else {
        game_state.add_message("Prestige unavailable yet.".to_string());
    }
}

pub fn start_cooking(
    station_color: &str,
    data: &GameData,
    progression: &ProgressionState,
    game_state: &mut GameState,
) -> bool {
    let cooking_slot_limit = data.balance.cooking_slots_limit;
    let Some(station) = game_state.station_mut(station_color) else {
        return false;
    };

    if station.can_cook(cooking_slot_limit) {
        station.is_cooking = true;
        station.remaining_ms = cooking_time_ms(data, progression, station_color);
        game_state.tutorial_observe(TutorialTrigger::CookingStarted, data);
        true
    } else {
        false
    }
}

pub fn serve_customer(
    station_color: &str,
    customer_id: u32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) -> bool {
    let Some(pos) = game_state
        .customers
        .iter()
        .position(|customer| customer.id == customer_id)
    else {
        return false;
    };
    if !game_state.customers[pos].is_seated {
        game_state.add_message(format!(
            "{} is still walking to their table.",
            game_state.customers[pos].display_name
        ));
        return false;
    }

    let Some(course_idx) = game_state.customers[pos].next_course_for(station_color) else {
        game_state.add_message(format!(
            "{} didn't order {}.",
            game_state.customers[pos].display_name,
            dish_display_name(data, station_color)
        ));
        return false;
    };

    let dish_name = {
        let Some(station) = game_state.station_mut(station_color) else {
            return false;
        };
        if station.dishes.is_empty() {
            return false;
        }
        station.dishes.remove(0)
    };

    let (satisfaction_gain, delicious_gain, preferred) = {
        let customer = &game_state.customers[pos];
        let traits = customer.traits(data);
        if data.customer_type_by_id(&customer.customer_type).is_some() {
            serving_gain(data, &customer.customer_type, &traits, station_color)
        } else {
            (data.balance.base_satisfaction_gain, 0.0, false)
        }
    };

    {
        let customer = &mut game_state.customers[pos];
        let traits = customer.traits(data);
        if let Some(existing) = customer.satisfaction.get_mut(station_color) {
            let max_total = customer.max_satisfaction.get(station_color).unwrap_or(40.0);
            let limit = max_total * overfeed_multiplier(data, &traits);
            *existing = (*existing + satisfaction_gain).min(limit);
        }
        customer.deliciousness = (customer.deliciousness + delicious_gain).min(5.0);
        customer.refresh_totals();
    }

    let is_overfed = game_state.customers[pos].overfed;
    let score_gain = serving_points(data, satisfaction_gain, preferred) as f64;
    let total_gain = add_score(game_state, progression, score_gain, true);
    let bill_gain = serving_bill(data, preferred);
    let (course_label, courses_done, courses_total, running_tab) = {
        let customer = &mut game_state.customers[pos];
        customer.order[course_idx].served = true;
        customer.bill = customer.bill.saturating_add(bill_gain);
        (
            customer.order[course_idx].label.clone(),
            customer.courses_served(),
            customer.order.len(),
            customer.bill,
        )
    };
    progression.record_served_dish(preferred, is_overfed);
    guest_state.record_guest_fed(&game_state.customers[pos].guest_id);
    game_state.combo = game_state.combo.saturating_add(1);
    game_state.chain = game_state.chain.saturating_add(1);
    game_state.add_message(format!(
        "{} served {course_label} ({dish_name}) +{total_gain} pts, tab ${running_tab} [{courses_done}/{courses_total}]",
        game_state.customers[pos].display_name
    ));
    let (floor_x, floor_y) = {
        let customer = &game_state.customers[pos];
        (customer.floor_x, customer.floor_y)
    };
    let gain_text = if preferred {
        format!("+{total_gain} renown  (loved it!)")
    } else {
        format!("+{total_gain} renown")
    };
    game_state
        .floaters
        .spawn_at(gain_text, FloaterKind::Renown, floor_x, floor_y);
    game_state.tutorial_observe(TutorialTrigger::CourseServed, data);

    true
}

pub fn invite_customer_to_vip(
    customer_id: u32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) -> bool {
    if game_state.special_table_busy {
        game_state.add_message("Last Meal Lounge is occupied right now.".to_string());
        return false;
    }

    let Some(index) = game_state
        .customers
        .iter()
        .position(|customer| customer.id == customer_id)
    else {
        return false;
    };
    if !game_state.customers[index].is_seated {
        game_state.add_message(format!(
            "{} needs to sit before lounge service.",
            game_state.customers[index].display_name
        ));
        return false;
    }
    if !can_process_customer(&game_state.customers[index], data) {
        let customer = &game_state.customers[index];
        let needed = visits_until_ready_for(data, &customer.customer_type);
        game_state.add_message(format!(
            "{} isn't plump enough yet — {} more visits ({}/{}).",
            customer.display_name,
            needed.saturating_sub(customer.times_fed),
            customer.times_fed,
            needed
        ));
        return false;
    }

    if !crate::engine::chance(crate::engine::VIP_ACCEPT_CHANCE) {
        game_state.add_message(format!(
            "{} declined the invitation.",
            game_state.customers[index].display_name
        ));
        return false;
    }

    let customer = game_state.customers.remove(index);
    let meat_gain = vip_meat_gain(&customer, data, progression);
    let meat_type = format!("{}-meat", customer.customer_type);
    let entry = game_state.ingredients.entry(meat_type.clone()).or_insert(0);
    if *entry != INFINITE_INGREDIENTS {
        *entry = entry.saturating_add(meat_gain);
    }

    let chain_value = game_state.chain.saturating_add(1);
    game_state.chain = chain_value;
    game_state.combo = game_state.combo.saturating_add(1);
    guest_state.record_guest_processed(&customer.guest_id);

    let meal_points = ((vip_points(&customer, data) as f64) + (meat_gain as f64 * 10.0))
        * data.balance.base_score_multiplier;
    let awarded = add_score(game_state, progression, meal_points, true);
    let cash_gain = (awarded / 5).max(0);
    progression.add_currency(cash_gain);
    progression.record_processed_customer(&customer.customer_type, chain_value);
    game_state.special_table_busy = true;
    game_state.special_table_timer = data.balance.special_table_process_time;
    game_state.add_message(format!(
        "{} steps into the Last Meal Lounge. Larder +{} {} (+{} renown).",
        customer.display_name, meat_gain, meat_type, awarded
    ));
    // Rewards are already banked above; the cinematic only stages the moment.
    game_state.processing_cinematic = Some(ProcessingCinematic::new(
        customer.display_name.clone(),
        customer.customer_type.clone(),
        meat_gain,
        meat_type,
        awarded,
        cash_gain,
        (customer.floor_x, customer.floor_y),
    ));
    game_state.tutorial_observe(TutorialTrigger::GuestProcessed, data);

    true
}

pub fn attract_customer_type(
    customer_type_id: &str,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    let Some(customer_type) = data.customer_type_by_id(customer_type_id) else {
        game_state.add_message("Unknown clientele.".to_string());
        return;
    };
    if progression.is_customer_unlocked(customer_type_id) {
        game_state.add_message(format!("{} already visits the cafe.", customer_type.name));
        return;
    }

    let missing = missing_ingredients(game_state, &customer_type.unlock_cost);
    if !missing.is_empty() {
        game_state.add_message(format!(
            "Need more ingredients to attract {}: {}.",
            customer_type.name,
            missing.join(", ")
        ));
        return;
    }

    if !spend_ingredients(
        game_state,
        &customer_type.unlock_cost,
        "attracting clientele",
    ) {
        return;
    }

    if progression.unlock_customer_type(customer_type_id) {
        game_state.add_message(format!(
            "{} clientele unlocked. They can now walk in.",
            customer_type.name
        ));
    }
}

pub fn craft_recipe(
    recipe_id: &str,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    let Some(index) = progression
        .recipes
        .iter()
        .position(|recipe| recipe.id == recipe_id)
    else {
        game_state.add_message("Unknown recipe.".to_string());
        return;
    };

    let recipe = progression.recipes[index].clone();
    if !recipe.unlocked {
        game_state.add_message(format!("{} is locked.", recipe.name));
        return;
    }

    let required: Vec<_> = recipe
        .ingredients
        .iter()
        .filter(|(ingredient, _)| ingredient != &&data.balance.regular_ingredient_name)
        .map(|(ingredient, amount)| (ingredient.clone(), *amount))
        .collect();
    if !required
        .iter()
        .all(|(ingredient, amount)| game_state.has_ing(ingredient, *amount))
    {
        game_state.add_message(format!("Not enough ingredients for {}.", recipe.name));
        return;
    }

    if !spend_ingredient_pairs(game_state, &required, "crafting") {
        return;
    }

    let points = recipe.base_value as f64
        * recipe.profit_multiplier
        * recipe_value_multiplier(progression)
        * data.balance.base_score_multiplier;
    let awarded = add_score(game_state, progression, points, false);
    let cash_gain = awarded / 4;
    progression.add_currency(cash_gain);
    let bonus_capacity = recipe_capacity_gain(progression, recipe.capacity_bonus);
    progression.record_crafted_recipe(&recipe.id, bonus_capacity);
    game_state.add_message(format!("Crafted {} for {} points.", recipe.name, awarded));
    game_state.floaters.spawn(
        format!("{}: +{} renown, +${}", recipe.name, awarded, cash_gain),
        FloaterKind::Renown,
        FloaterAnchor::Header,
    );
}

fn missing_ingredients(
    game_state: &GameState,
    cost: &std::collections::HashMap<String, i64>,
) -> Vec<String> {
    cost.iter()
        .filter_map(|(ingredient, amount)| {
            if game_state.has_ing(ingredient, *amount) {
                None
            } else {
                let current = game_state.ingredients.get(ingredient).copied().unwrap_or(0);
                Some(format!("{ingredient} {current}/{amount}"))
            }
        })
        .collect()
}

fn spend_ingredients(
    game_state: &mut GameState,
    cost: &std::collections::HashMap<String, i64>,
    context: &str,
) -> bool {
    let pairs: Vec<_> = cost
        .iter()
        .map(|(ingredient, amount)| (ingredient.clone(), *amount))
        .collect();
    spend_ingredient_pairs(game_state, &pairs, context)
}

fn spend_ingredient_pairs(
    game_state: &mut GameState,
    pairs: &[(String, i64)],
    context: &str,
) -> bool {
    for (ingredient, amount) in pairs {
        if !game_state.remove_ingredients(ingredient, *amount) {
            game_state.add_message(format!("Missing {ingredient} while {context}."));
            return false;
        }
    }
    true
}

fn add_score(
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    points: f64,
    apply_combo: bool,
) -> i64 {
    let combo_multiplier = if apply_combo {
        let combo_boost = progression.get_effect("combo_multiplier", 1.0);
        1.0 + (f64::from(game_state.combo) * 0.1 * combo_boost)
    } else {
        1.0
    };
    let prestige_multiplier = 1.0 + (f64::from(progression.prestige_points) * 0.03);
    let awarded = (points * combo_multiplier * prestige_multiplier).max(0.0);

    let next_score = awarded.floor() as i64;
    if next_score > 0 {
        progression.record_score(next_score);
        game_state.score = game_state.score.saturating_add(next_score);
    }

    next_score
}
