use chrono::{DateTime, Local};

//Mon, 06 Mar 2023 10:21:03 -0800
pub fn date_group(sent_at: &str) -> String {
    let sent: Option<DateTime<Local>> = DateTime::parse_from_rfc2822(sent_at).ok().map(Into::into);
    let now = Local::now();
    sent.map(|date| {
        if date.date_naive() == now.date_naive() {
            "Today".to_string()
        } else if date.format("%W-%Y").to_string() == now.format("%W-%Y").to_string() {
            "This week".to_string()
        } else if date.format("%m-%Y").to_string() == now.format("%m-%Y").to_string() {
            "This month".to_string()
        } else {
            date.format("%m-%Y").to_string()
        }
    })
    .unwrap_or_else(|| {
        dbg!(sent_at);
        "Date Error".to_string()
    })
}
// make local dates!
pub fn date_format(sent_at: &str) -> String {
    let sent: Option<DateTime<Local>> = DateTime::parse_from_rfc2822(sent_at).ok().map(Into::into);
    sent.map(|date| date.format("%a, %b %d, %Y, %I:%M %p").to_string())
        .unwrap_or_else(|| "".to_string())
}
pub fn relative_date_format(sent_at: &str) -> String {
    let sent: Option<DateTime<Local>> = DateTime::parse_from_rfc2822(sent_at).ok().map(Into::into);
    let now = Local::now();
    sent.map(|date| {
        if date.date_naive() == now.date_naive() {
            date.format("%I:%M %p").to_string()
        } else if date.format("%Y").to_string() == now.format("%Y").to_string() {
            date.format("%b %d").to_string()
        } else {
            date.format("%m/%d/%Y").to_string()
        }
    })
    .unwrap_or_else(|| "".to_string())
}
