//! Dish freshness on the pass: fresh dishes pay a bill bonus, stale ones pay
//! normally, spoiled ones are thrown out. This is the kitchen's clock — it
//! turns fire-and-forget cooking into a timing decision.

use crate::data::GameBalance;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
    Spoiled,
}

pub fn classify_dish_age(age_ms: f32, balance: &GameBalance) -> Freshness {
    if age_ms >= balance.dish_spoil_ms.max(1.0) {
        Freshness::Spoiled
    } else if age_ms >= balance.dish_fresh_window_ms.max(1.0) {
        Freshness::Stale
    } else {
        Freshness::Fresh
    }
}

/// Bill multiplier for serving a dish in this state. Spoiled dishes never
/// reach a table (the pass discards them), so only Fresh pays extra.
pub fn freshness_bill_multiplier(freshness: Freshness, balance: &GameBalance) -> f64 {
    match freshness {
        Freshness::Fresh => balance.fresh_bill_bonus_multiplier.max(1.0),
        Freshness::Stale | Freshness::Spoiled => 1.0,
    }
}

/// Seconds until the dish stops being fresh (for the pass UI countdown).
pub fn seconds_until_stale(age_ms: f32, balance: &GameBalance) -> f32 {
    ((balance.dish_fresh_window_ms - age_ms) / 1000.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn balance() -> GameBalance {
        GameBalance::default()
    }

    #[test]
    fn dishes_progress_fresh_to_stale_to_spoiled() {
        let balance = balance();
        assert_eq!(classify_dish_age(0.0, &balance), Freshness::Fresh);
        assert_eq!(
            classify_dish_age(balance.dish_fresh_window_ms, &balance),
            Freshness::Stale
        );
        assert_eq!(
            classify_dish_age(balance.dish_spoil_ms, &balance),
            Freshness::Spoiled
        );
    }

    #[test]
    fn only_fresh_dishes_pay_the_bonus() {
        let balance = balance();
        assert!(freshness_bill_multiplier(Freshness::Fresh, &balance) > 1.0);
        assert_eq!(freshness_bill_multiplier(Freshness::Stale, &balance), 1.0);
    }
}
