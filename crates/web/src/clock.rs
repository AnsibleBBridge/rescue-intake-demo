//! Browser clock bridge: core never reads time, so the web layer supplies local "now".
use chrono::{NaiveDate, NaiveDateTime};

pub fn now() -> NaiveDateTime {
    let d = js_sys::Date::new_0();
    // JS getters are local-time, matching what the user sees on their clock.
    NaiveDate::from_ymd_opt(d.get_full_year() as i32, d.get_month() + 1, d.get_date())
        .and_then(|date| date.and_hms_opt(d.get_hours(), d.get_minutes(), d.get_seconds()))
        .unwrap_or_default()
}
