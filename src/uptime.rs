pub fn duration(uptime: f64) -> (String, String, String) {
    let days = if uptime > 86400.0 {
        let days_pre = uptime / 60.0 / 60.0 / 24.0;
        days_pre.round().to_string() + "d"
    } else {
        "".to_string()
    };
    let hours = if uptime > 3600.0 {
        let hours_pre = (uptime / 60.0 / 60.0) % 24.0;
        hours_pre.round().to_string() + "h"
    } else {
        "".to_string()
    };
    let minutes = if uptime > 60.0 {
        let minutes_pre = (uptime / 60.0) % 60.0;
        minutes_pre.round().to_string() + "m"
    } else {
        "".to_string()
    };
    (days, hours, minutes)
}
