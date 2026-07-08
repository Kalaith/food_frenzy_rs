//! Per-guest state readout: satisfaction, patience, and fattening progress
//! meters, the "plump and ready" callout, and a hover panel that explains
//! what the numbers mean. You can't play around state you can't see.

use super::common::{draw_bar, patience_color, patience_remaining_ratio, GOLD, LINE, MUTED, TEXT};
use crate::data::GameData;
use crate::engine::visits_until_ready_for;
use crate::state::{Customer, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const READY_GOLD: Color = Color::new(0.95, 0.78, 0.35, 1.0);
const PLUMP_PINK: Color = Color::new(0.92, 0.55, 0.62, 1.0);

pub(super) fn draw_guest_meters(
    pos: Vec2,
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
) {
    // Satisfaction: how well fed they are this sitting.
    draw_bar(
        pos.x - 52.0,
        pos.y - 72.0,
        104.0,
        6.0,
        customer.total_satisfaction,
        customer.max_satisfaction.total(),
        LIME,
    );

    if !customer.is_seated {
        return;
    }

    // Patience: how long before they storm out.
    let patience = patience_remaining_ratio(customer, data, progression, now_ms);
    draw_bar(
        pos.x - 52.0,
        pos.y + 8.0,
        104.0,
        5.0,
        patience,
        1.0,
        patience_color(patience),
    );

    draw_fattening_pips(pos, customer, data);
    draw_trait_alert(pos, customer, data);
}

/// Telegraphed trait warning: what they're about to do and how long the
/// player has to answer.
fn draw_trait_alert(pos: Vec2, customer: &Customer, data: &GameData) {
    let Some(alert) = &customer.trait_alert else {
        return;
    };
    let Some(behavior) = data.trait_behavior(&alert.trait_key) else {
        return;
    };
    let window = data.balance.trait_telegraph_ms.max(500.0);
    let remaining = (alert.remaining_ms / window).clamp(0.0, 1.0);
    let text = format!("! {}", behavior.telegraph);
    let dim = measure_ui_text(&text, None, 14, 1.0);
    let rect = Rect::new(
        pos.x - dim.width * 0.5 - 8.0,
        pos.y - 152.0,
        dim.width + 16.0,
        24.0,
    );
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.18, 0.05, 0.05, 0.92),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.5,
        Color::new(0.94, 0.42, 0.36, 1.0),
    );
    draw_ui_text(
        &text,
        rect.x + 8.0,
        rect.y + 17.0,
        14.0,
        Color::new(0.96, 0.72, 0.60, 1.0),
    );
    draw_bar(
        rect.x,
        rect.y + rect.h + 2.0,
        rect.w,
        3.0,
        remaining,
        1.0,
        Color::new(0.94, 0.42, 0.36, 1.0),
    );
}

/// One pip per satisfied visit toward Lounge readiness. The meter the whole
/// meta-loop hangs on, so it lives directly under every seated guest.
fn draw_fattening_pips(pos: Vec2, customer: &Customer, data: &GameData) {
    let needed = visits_until_ready_for(data, &customer.customer_type).max(1);
    let fed = customer.times_fed.min(needed);
    let ready = customer.times_fed >= needed;

    let spacing = 13.0;
    let total_w = spacing * (needed.saturating_sub(1)) as f32;
    let start_x = pos.x - total_w * 0.5;
    let y = pos.y + 21.0;
    for index in 0..needed {
        let filled = index < fed;
        let center = vec2(start_x + spacing * index as f32, y);
        if filled {
            draw_circle(center.x, center.y, 4.5, PLUMP_PINK);
        } else {
            draw_circle_lines(center.x, center.y, 4.5, 1.2, MUTED);
        }
    }

    if ready {
        let pulse = ((macroquad::time::get_time() * 4.0).sin() * 0.5 + 0.5) as f32;
        let label = "PLUMP & READY";
        let dim = measure_ui_text(label, None, 13, 1.0);
        let rect = Rect::new(
            pos.x - dim.width * 0.5 - 8.0,
            y + 8.0,
            dim.width + 16.0,
            20.0,
        );
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            Color::new(0.16, 0.06, 0.10, 0.90),
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.5,
            Color::new(
                READY_GOLD.r,
                READY_GOLD.g,
                READY_GOLD.b,
                0.55 + 0.45 * pulse,
            ),
        );
        draw_ui_text(label, rect.x + 8.0, rect.y + 15.0, 13.0, READY_GOLD);
    }
}

/// Detailed hover readout so every meter has words attached to it.
pub(super) fn draw_guest_hover_panel(
    pos: Vec2,
    hover_rect: Rect,
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
) {
    let mouse = vec2(mouse_position().0, mouse_position().1);
    if !hover_rect.contains(mouse) {
        return;
    }

    let customer_type = data.customer_type_by_id(&customer.customer_type);
    let type_line = customer_type
        .map(|item| format!("{} (Tier {})", item.name, item.profile_tier.max(1)))
        .unwrap_or_else(|| "Guest".to_string());
    let needed = visits_until_ready_for(data, &customer.customer_type);
    let fattening_line = if customer.times_fed >= needed {
        "Plump and ready for the Lounge".to_string()
    } else {
        format!(
            "Fattening: {}/{} visits ({} more to Lounge)",
            customer.times_fed,
            needed,
            needed - customer.times_fed
        )
    };
    let patience = patience_remaining_ratio(customer, data, progression, now_ms);
    let lines = [
        format!("{} - {}", customer.display_name, type_line),
        format!(
            "Satisfaction: {:.0}/{:.0}",
            customer.total_satisfaction,
            customer.max_satisfaction.total()
        ),
        format!("Patience: {:.0}%", patience * 100.0),
        fattening_line,
        format!("Tab so far: ${}", customer.bill.max(0)),
    ];

    let font = 14.0;
    let width = lines
        .iter()
        .map(|line| measure_ui_text(line, None, font as u16, 1.0).width)
        .fold(0.0_f32, f32::max)
        + 24.0;
    let height = lines.len() as f32 * 19.0 + 16.0;
    let panel = Rect::new(
        (pos.x + 64.0).min(screen_width() - width - 8.0),
        (pos.y - 120.0).max(8.0),
        width,
        height,
    );
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::new(0.02, 0.02, 0.025, 0.94),
    );
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.0, LINE);
    for (index, line) in lines.iter().enumerate() {
        let color = if index == 0 { GOLD } else { TEXT };
        draw_ui_text(
            line,
            panel.x + 12.0,
            panel.y + 22.0 + index as f32 * 19.0,
            font,
            color,
        );
    }
}
