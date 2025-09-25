use time::macros::format_description;
use time::Date;

pub fn human_date_fr(jj_mm_aaaa: &str) -> String {
    // expects validated JJ/MM/AAAA
    let parts: Vec<&str> = jj_mm_aaaa.split('/').collect();
    let d: i32 = parts[2].parse().unwrap_or(1970);
    let m: u8 = parts[1].parse().unwrap_or(1);
    let day: u8 = parts[0].parse().unwrap_or(1);
    let dt = Date::from_calendar_date(d, time::Month::try_from(m).unwrap(), day).unwrap();
    let fmt = format_description!("[day] [month repr:long] [year]");
    dt.format(&fmt).unwrap_or_else(|_| jj_mm_aaaa.to_string())
}
