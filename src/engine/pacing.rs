//! Course pacing: after a course lands, the guest spends a while eating, then
//! wants the next course within a grace window. Serving into the eating
//! window is rushed (penalized renown); hitting the window between is
//! well-paced (bonus); past the grace the guest is kept waiting (penalized,
//! and their satisfaction bleeds until the course arrives).

use crate::data::GameBalance;
use crate::state::Customer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoursePacing {
    /// The guest's first course this visit: no rhythm to judge yet.
    FirstCourse,
    /// Served while they were still eating the previous course.
    Rushed,
    /// Served after eating, within the grace window.
    WellPaced,
    /// Served after the grace window had already run out.
    KeptWaiting,
}

/// Judge the pacing of serving this guest their *next* course right now.
pub fn classify_course_pacing(customer: &Customer, balance: &GameBalance) -> CoursePacing {
    if customer.courses_served() == 0 {
        CoursePacing::FirstCourse
    } else if customer.eating_ms > 0.0 {
        CoursePacing::Rushed
    } else if customer.waiting_ms <= balance.course_wait_grace_ms.max(0.0) {
        CoursePacing::WellPaced
    } else {
        CoursePacing::KeptWaiting
    }
}

pub fn pacing_score_multiplier(pacing: CoursePacing, balance: &GameBalance) -> f64 {
    match pacing {
        CoursePacing::FirstCourse => 1.0,
        CoursePacing::Rushed => balance.rushed_course_score_multiplier.clamp(0.0, 1.0),
        CoursePacing::WellPaced => balance.paced_course_score_multiplier.max(1.0),
        CoursePacing::KeptWaiting => balance.late_course_score_multiplier.clamp(0.0, 1.0),
    }
}

/// True while the guest has finished eating, still has courses coming, and
/// the grace window has expired — the state that bleeds satisfaction.
pub fn is_kept_waiting(customer: &Customer, balance: &GameBalance) -> bool {
    customer.is_seated
        && !customer.order_complete()
        && customer.courses_served() > 0
        && customer.eating_ms <= 0.0
        && customer.waiting_ms > balance.course_wait_grace_ms.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Course, Satisfaction};

    fn customer(served: usize, total: usize, eating_ms: f32, waiting_ms: f32) -> Customer {
        let order = (0..total)
            .map(|index| Course {
                color: "blue".to_string(),
                label: "Main".to_string(),
                served: index < served,
            })
            .collect();
        Customer {
            id: 1,
            guest_id: "guest-1".to_string(),
            display_name: "Test".to_string(),
            customer_type: "pig".to_string(),
            satisfaction: Satisfaction::default(),
            max_satisfaction: Satisfaction::default(),
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
            bill: 0,
            depart_timer_ms: 0.0,
            order,
            times_fed: 0,
            trait_alert: None,
            personality: None,
            eating_ms,
            waiting_ms,
        }
    }

    #[test]
    fn pacing_covers_the_full_meal_rhythm() {
        let balance = GameBalance::default();
        assert_eq!(
            classify_course_pacing(&customer(0, 3, 0.0, 0.0), &balance),
            CoursePacing::FirstCourse
        );
        assert_eq!(
            classify_course_pacing(&customer(1, 3, 4_000.0, 0.0), &balance),
            CoursePacing::Rushed
        );
        assert_eq!(
            classify_course_pacing(
                &customer(1, 3, 0.0, balance.course_wait_grace_ms - 1.0),
                &balance
            ),
            CoursePacing::WellPaced
        );
        assert_eq!(
            classify_course_pacing(
                &customer(1, 3, 0.0, balance.course_wait_grace_ms + 1.0),
                &balance
            ),
            CoursePacing::KeptWaiting
        );
    }

    #[test]
    fn multipliers_penalize_rush_and_lateness_but_reward_rhythm() {
        let balance = GameBalance::default();
        assert!(pacing_score_multiplier(CoursePacing::Rushed, &balance) < 1.0);
        assert!(pacing_score_multiplier(CoursePacing::KeptWaiting, &balance) < 1.0);
        assert!(pacing_score_multiplier(CoursePacing::WellPaced, &balance) > 1.0);
        assert_eq!(
            pacing_score_multiplier(CoursePacing::FirstCourse, &balance),
            1.0
        );
    }

    #[test]
    fn hangry_state_requires_expired_grace_and_pending_courses() {
        let balance = GameBalance::default();
        let over_grace = balance.course_wait_grace_ms + 1.0;
        assert!(is_kept_waiting(&customer(1, 3, 0.0, over_grace), &balance));
        // Still eating: not hangry.
        assert!(!is_kept_waiting(
            &customer(1, 3, 500.0, over_grace),
            &balance
        ));
        // Order complete: nothing left to wait for.
        assert!(!is_kept_waiting(&customer(3, 3, 0.0, over_grace), &balance));
        // No course served yet: ordinary patience covers the first wait.
        assert!(!is_kept_waiting(&customer(0, 3, 0.0, over_grace), &balance));
    }
}
