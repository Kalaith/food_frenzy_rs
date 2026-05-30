use super::common::{
    can_afford_cost, draw_button, draw_card, draw_panel, draw_section_title, ellipsize,
    format_unlock_cost, sorted_ingredient_lines, MUTED, TEXT,
};
use super::types::UiActions;
use crate::data::GameData;
use crate::state::{GameState, ProgressionState};
use macroquad::prelude::*;

pub(super) fn draw_growth_panel(
    panel: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_panel(panel);
    draw_section_title("Cafe plan", panel.x + 18.0, panel.y + 30.0);

    let card_x = panel.x + 14.0;
    let card_w = panel.w - 28.0;
    let guest_card = Rect::new(card_x, panel.y + 48.0, card_w, 286.0);
    let upgrade_card = Rect::new(card_x, guest_card.y + guest_card.h + 12.0, card_w, 178.0);
    let recipe_card = Rect::new(
        card_x,
        upgrade_card.y + upgrade_card.h + 12.0,
        card_w,
        panel.y + panel.h - (upgrade_card.y + upgrade_card.h + 26.0),
    );

    draw_guest_card(guest_card, game, progression, data, ui);
    draw_upgrade_card(upgrade_card, progression, ui);
    draw_recipe_card(recipe_card, progression, data, ui);
}

pub(super) fn draw_event_feed(rect: Rect, game: &GameState) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.07, 0.07, 0.085, 0.95),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(0.18, 0.18, 0.20, 1.0),
    );
    draw_text("Ticker", rect.x + 14.0, rect.y + 24.0, 18.0, TEXT);

    let mut x = rect.x + 92.0;
    for message in game.messages.iter().rev().take(5) {
        let text = ellipsize(message, 38);
        let text_dim = measure_text(&text, None, 15, 1.0);
        let pill_w = (text_dim.width + 28.0).min(330.0);
        if x + pill_w > rect.x + rect.w - 14.0 {
            break;
        }
        draw_rectangle(
            x,
            rect.y + 10.0,
            pill_w,
            30.0,
            Color::new(0.11, 0.11, 0.13, 1.0),
        );
        draw_text(&text, x + 12.0, rect.y + 30.0, 15.0, LIGHTGRAY);
        x += pill_w + 12.0;
    }
}

fn draw_guest_card(
    card: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_card(card, "Guests");
    draw_pantry(card, game);

    draw_text("Next", card.x + 12.0, card.y + 88.0, 15.0, TEXT);
    let mut locked_customer_types: Vec<_> = data
        .customer_types
        .iter()
        .filter(|customer_type| !progression.is_customer_unlocked(&customer_type.id))
        .collect();
    locked_customer_types.sort_by(|left, right| {
        left.profile_tier
            .cmp(&right.profile_tier)
            .then_with(|| left.name.cmp(&right.name))
    });
    if locked_customer_types.is_empty() {
        draw_text(
            "All known guests unlocked",
            card.x + 12.0,
            card.y + 116.0,
            15.0,
            MUTED,
        );
        return;
    }

    let mut y = card.y + 116.0;
    for customer_type in locked_customer_types.iter().take(3) {
        let can_attract = can_afford_cost(game, &customer_type.unlock_cost);
        draw_text(
            &format!(
                "T{} {}",
                customer_type.profile_tier.max(1),
                ellipsize(&customer_type.name, 18)
            ),
            card.x + 12.0,
            y,
            16.0,
            if can_attract { TEXT } else { MUTED },
        );
        draw_text(
            &ellipsize(&format_unlock_cost(&customer_type.unlock_cost), 26),
            card.x + 12.0,
            y + 19.0,
            12.0,
            if can_attract { LIGHTGRAY } else { MUTED },
        );
        let button_rect = Rect::new(card.x + card.w - 94.0, y - 18.0, 78.0, 28.0);
        draw_button(button_rect, "Attract", can_attract, !can_attract);
        ui.attract_buttons
            .insert(customer_type.id.clone(), button_rect);
        y += 58.0;
    }
}

fn draw_pantry(card: Rect, game: &GameState) {
    let ingredients = sorted_ingredient_lines(game);
    draw_text("Pantry", card.x + 12.0, card.y + 54.0, 14.0, MUTED);
    if ingredients.is_empty() {
        draw_text("Empty", card.x + 68.0, card.y + 54.0, 14.0, MUTED);
    } else {
        let pantry = ingredients
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("  ");
        draw_text(
            &ellipsize(&pantry, 34),
            card.x + 68.0,
            card.y + 54.0,
            14.0,
            LIGHTGRAY,
        );
    }
}

fn draw_upgrade_card(card: Rect, progression: &ProgressionState, ui: &mut UiActions) {
    draw_card(card, "Upgrades");
    let mut y = card.y + 58.0;
    for upgrade in progression.upgrades.iter().take(2) {
        let can_buy = progression.currency >= upgrade.cost && upgrade.level < upgrade.max_level;
        draw_text(
            &format!(
                "{}  L{}/{}",
                ellipsize(&upgrade.name, 20),
                upgrade.level,
                upgrade.max_level
            ),
            card.x + 12.0,
            y,
            15.0,
            if can_buy { TEXT } else { LIGHTGRAY },
        );
        draw_text(
            &format!("${}", upgrade.cost),
            card.x + 12.0,
            y + 18.0,
            12.0,
            MUTED,
        );
        let button_rect = Rect::new(card.x + card.w - 94.0, y - 18.0, 78.0, 28.0);
        draw_button(button_rect, "Buy", can_buy, !can_buy);
        ui.upgrade_buttons.insert(upgrade.id.clone(), button_rect);
        y += 50.0;
    }
}

fn draw_recipe_card(
    card: Rect,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_card(card, "Recipes");
    let mut recipe_y = card.y + 58.0;
    for recipe in progression.recipes.iter().take(2) {
        let button_rect = Rect::new(card.x + card.w - 94.0, recipe_y - 18.0, 78.0, 28.0);
        draw_text(
            &ellipsize(&recipe.name, 22),
            card.x + 12.0,
            recipe_y,
            15.0,
            if recipe.unlocked { TEXT } else { MUTED },
        );
        draw_button(
            button_rect,
            if recipe.unlocked { "Craft" } else { "Locked" },
            recipe.unlocked,
            !recipe.unlocked,
        );
        ui.recipe_buttons.insert(recipe.id.clone(), button_rect);
        recipe_y += 46.0;
    }

    let prestige_rect = Rect::new(card.x + 12.0, card.y + card.h - 42.0, card.w - 24.0, 32.0);
    let can_prestige = progression.total_score >= data.balance.prestige_score_requirement;
    draw_button(
        prestige_rect,
        &format!("Prestige +{}", progression.prestige_reward()),
        can_prestige,
        !can_prestige,
    );
    ui.prestige_button = Some(prestige_rect);
}
