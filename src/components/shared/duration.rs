use chrono;

pub fn format_remaining(target: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = target.signed_duration_since(now);

    if duration.num_seconds() <= 0 {
        return "0m".to_owned();
    }

    let total_minutes = duration.num_minutes();
    let total_hours = duration.num_hours();
    let total_days = duration.num_days();

    if total_hours < 1 {
        format!("{total_minutes}m")
    } else if total_hours < 24 {
        let minutes = total_minutes % 60;
        format!("{total_hours}h {minutes}m")
    } else {
        let hours = total_hours % 24;
        format!("{total_days}d {hours}h")
    }
}
