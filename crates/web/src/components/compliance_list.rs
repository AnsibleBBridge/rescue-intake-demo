//! Records & reminders list: one row per applicable item with its due state (48h deadlines
//! for dogs/cats, plain reminders for everything else).
use chrono::NaiveDateTime;
use rescue_core::compliance::{ComplianceItem, ItemState};
use leptos::prelude::*;

/// Human wording for an item's state; pure so it can be unit-tested on the host.
pub fn format_due(key: &str, state: &ItemState, now: NaiveDateTime) -> String {
    match state {
        ItemState::Done => "Done".into(),
        ItemState::NotApplicable => String::new(),
        // The rabies reminder is only for dogs/cats; every other reminder has no clock.
        ItemState::Reminder if key == "rabies" => "Add when vaccinated".into(),
        ItemState::Reminder => "Reminder \u{2014} no deadline".into(),
        ItemState::Open { due } => {
            let mins = (*due - now).num_minutes();
            if mins < 60 { "due in <1h".into() } else { format!("due in {}h", (mins + 59) / 60) }
        }
        ItemState::Overdue { due } => {
            // Round up and never say "0h": anything past due is at least an hour late.
            let hours = (((now - *due).num_minutes() + 59) / 60).max(1);
            match (hours / 24, hours % 24) {
                (0, h) => format!("overdue by {h}h"),
                (d, 0) => format!("overdue by {d}d"),
                (d, h) => format!("overdue by {d}d {h}h"),
            }
        }
    }
}

/// Applicable items only (N/A rows are hidden); each shows icon, label, due text and its cite.
#[component]
pub fn ComplianceList(items: Vec<ComplianceItem>, now: NaiveDateTime) -> impl IntoView {
    let rows = items
        .into_iter()
        .filter(|i| i.state != ItemState::NotApplicable)
        .map(|i| {
            let (icon, class) = match i.state {
                ItemState::Done => ("\u{2705}", "ci done"),
                ItemState::Overdue { .. } => ("\u{1F534}", "ci overdue"),
                ItemState::Reminder => ("\u{2139}\u{FE0F}", "ci reminder"),
                _ => ("\u{23F3}", "ci open"),
            };
            view! {
                <li class=class data-key=i.key>
                    <span class="ci-icon" aria-hidden="true">{icon}</span>
                    <span class="ci-body">
                        <span class="ci-label">{i.label}</span>
                        <span class="ci-due">{format_due(i.key, &i.state, now)}</span>
                        <span class="ci-cite">{i.section}</span>
                    </span>
                </li>
            }
        })
        .collect_view();
    view! { <ul class="ci-list">{rows}</ul> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate};

    fn now() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 25).unwrap().and_hms_opt(12, 0, 0).unwrap()
    }
    fn open(mins: i64) -> String { format_due("k", &ItemState::Open { due: now() + Duration::minutes(mins) }, now()) }
    fn over(mins: i64) -> String { format_due("k", &ItemState::Overdue { due: now() - Duration::minutes(mins) }, now()) }

    #[test]
    fn open_rounds_hours_up() {
        assert_eq!(open(120), "due in 2h");
        assert_eq!(open(121), "due in 3h");
        assert_eq!(open(60), "due in 1h");
    }
    #[test]
    fn open_under_an_hour() {
        assert_eq!(open(59), "due in <1h");
        assert_eq!(open(1), "due in <1h");
    }
    #[test]
    fn overdue_hours_then_days() {
        assert_eq!(over(60), "overdue by 1h");
        assert_eq!(over(5 * 60), "overdue by 5h");
        assert_eq!(over(23 * 60), "overdue by 23h");
        assert_eq!(over(24 * 60), "overdue by 1d");
        assert_eq!(over(30 * 60), "overdue by 1d 6h");
    }
    #[test]
    fn overdue_under_an_hour_is_at_least_1h() {
        assert_eq!(over(10), "overdue by 1h");
    }
    #[test]
    fn done_and_na() {
        assert_eq!(format_due("k", &ItemState::Done, now()), "Done");
        assert_eq!(format_due("k", &ItemState::NotApplicable, now()), "");
        assert!(format_due("rabies", &ItemState::Reminder, now()).contains("Add when vaccinated"));
        assert!(format_due("coggins", &ItemState::Reminder, now()).contains("no deadline"));
    }
}
