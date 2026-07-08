//! Seated-guest lifecycle: patience walkouts, satisfied departures (paying the
//! tab and banking a fattening visit), and satisfaction decay over time.

use crate::data::GameData;
use crate::engine::satisfied_tip;
use crate::state::{GameState, GuestState, ProgressionState, Timers};

pub(super) fn update_patience(
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

        for id in &removed_ids {
            let Some(customer) = game_state.customers.iter().find(|item| item.id == *id) else {
                continue;
            };
            let paid = customer.bill.max(0);
            let name = customer.display_name.clone();
            let served = customer.courses_served();
            let ordered = customer.order.len();
            progression.add_currency(paid);
            progression.record_customer_lost();
            if paid > 0 {
                game_state.add_message(format!(
                    "{name} left unhappy ({served}/{ordered} courses), only paid ${paid}."
                ));
            } else {
                game_state.add_message(format!("{name} left hungry and unhappy. A shame."));
            }
        }

        game_state
            .customers
            .retain(|customer| !removed_ids.contains(&customer.id));
        game_state.combo = 0;
    }
}

/// Guests who have eaten every course of their order finish up, settle their
/// tab (plus a tip), and leave — freeing the table and banking a satisfied
/// visit toward the day they are plump enough for the Last Meal Lounge.
pub(super) fn update_departures(
    dt_ms: f32,
    data: &GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) {
    let dwell = data.balance.content_dwell_ms.max(0.0);
    let mut departed = Vec::new();
    for customer in &mut game_state.customers {
        if !(customer.is_seated && customer.order_complete()) {
            customer.depart_timer_ms = 0.0;
            continue;
        }

        if customer.depart_timer_ms <= 0.0 {
            customer.depart_timer_ms = dwell.max(1.0);
        }
        customer.depart_timer_ms -= dt_ms;
        if customer.depart_timer_ms <= 0.0 {
            departed.push(customer.id);
        }
    }

    if departed.is_empty() {
        return;
    }

    let mut messages = Vec::new();
    for id in &departed {
        let Some(customer) = game_state.customers.iter().find(|item| item.id == *id) else {
            continue;
        };
        let bill = customer.bill.max(0);
        let tip = satisfied_tip(data, bill);
        let paid = bill.saturating_add(tip);
        progression.add_currency(paid);
        guest_state.record_guest_satisfied_visit(&customer.guest_id);
        let fed = customer.times_fed.saturating_add(1);
        let ready_hint = if fed >= data.balance.visits_until_ready {
            " Plump enough for the Lounge next visit."
        } else {
            ""
        };
        messages.push(format!(
            "{} left glowing and settled ${paid} (${bill} +${tip} tip).{ready_hint}",
            customer.display_name
        ));
    }

    game_state
        .customers
        .retain(|customer| !departed.contains(&customer.id));
    game_state.combo = game_state.combo.saturating_add(1);
    for message in messages {
        game_state.add_message(message);
    }
}

pub(super) fn update_satisfaction_decay(
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

    let decay_rate = crate::engine::satisfaction_decay_rate(data, progression);
    let decay_tick = decay_interval;
    while timers.decay_accum_ms >= decay_tick {
        timers.decay_accum_ms -= decay_tick;
        for customer in &mut game_state.customers {
            customer.satisfaction.decay_all(decay_rate);
            customer.refresh_totals();
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Course, Customer, Satisfaction};

    fn seated_customer(id: u32, bill: i64, order: Vec<Course>, times_fed: u32) -> Customer {
        Customer {
            id,
            guest_id: format!("guest-{id}"),
            display_name: "Test".to_string(),
            customer_type: "pig".to_string(),
            satisfaction: Satisfaction::default(),
            max_satisfaction: Satisfaction {
                blue: 40.0,
                green: 40.0,
                yellow: 40.0,
                red: 40.0,
            },
            deliciousness: 1.0,
            total_satisfaction: 0.0,
            overfed: false,
            table_index: 0,
            arrived_at_ms: 0.0,
            floor_x: 0.0,
            floor_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            is_seated: true,
            bill,
            depart_timer_ms: 0.0,
            order,
            times_fed,
        }
    }

    fn course(color: &str, served: bool) -> Course {
        Course {
            color: color.to_string(),
            label: "Main".to_string(),
            served,
        }
    }

    #[test]
    fn guest_pays_bill_and_tip_once_order_is_complete() {
        let data = GameData::load();
        let mut game_state = GameState::new(&data);
        let mut progression = ProgressionState::from_game_data(&data);
        // Empty guest list: the satisfied-visit bookkeeping is a no-op here, which
        // keeps the test off macroquad's time API (unavailable headless).
        let mut guest_state = GuestState::new();
        game_state.customers.push(seated_customer(
            1,
            20,
            vec![course("blue", true), course("red", true)],
            0,
        ));

        let dt = data.balance.content_dwell_ms + 100.0;
        update_departures(
            dt,
            &data,
            &mut game_state,
            &mut progression,
            &mut guest_state,
        );

        assert!(game_state.customers.is_empty(), "guest should have left");
        let tip = crate::engine::satisfied_tip(&data, 20);
        assert_eq!(progression.currency, 20 + tip);
    }

    #[test]
    fn guest_with_unserved_course_stays_seated() {
        let data = GameData::load();
        let mut game_state = GameState::new(&data);
        let mut progression = ProgressionState::from_game_data(&data);
        let mut guest_state = GuestState::new();
        game_state.customers.push(seated_customer(
            1,
            12,
            vec![course("blue", true), course("red", false)],
            0,
        ));

        let dt = data.balance.content_dwell_ms + 100.0;
        update_departures(
            dt,
            &data,
            &mut game_state,
            &mut progression,
            &mut guest_state,
        );

        assert_eq!(
            game_state.customers.len(),
            1,
            "order not finished, guest stays"
        );
        assert_eq!(progression.currency, 0);
    }

    #[test]
    fn readiness_is_reached_after_enough_fed_visits() {
        let data = GameData::load();
        let below = seated_customer(
            1,
            0,
            vec![course("blue", false)],
            data.balance.visits_until_ready - 1,
        );
        let ready = seated_customer(
            2,
            0,
            vec![course("blue", false)],
            data.balance.visits_until_ready,
        );
        assert!(!crate::engine::can_process_customer(&below, &data));
        assert!(crate::engine::can_process_customer(&ready, &data));
    }
}
