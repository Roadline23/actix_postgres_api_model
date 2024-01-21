use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use tracing::error;

pub fn format_custom_dberr(errors: String) -> String {
    let err = errors.split("Object").nth(1).unwrap().to_string();
    return err.replace("String(\"", "\"").replace("\")", "\"");
}

pub fn format_validation_error(error: &Value) -> String {
    let validation_error = error.get(0).unwrap().to_owned();
    return validation_error["code"].to_string().replace("\"", "");
}

pub fn format_timestamp_into_string_date(timestamp: i64) -> String {
    let dt = Utc.timestamp_opt(timestamp + 3_600, 0).unwrap();

    let dt_string = dt.format("%A %e %B %Y").to_string();
    let (wd_str, other_part) = dt_string.split_once(" ").unwrap();
    let (d_str, last_part) = other_part.split_once(" ").unwrap();
    let (m_str, y_str) = last_part.split_once(" ").unwrap();

    let weekday_str = match wd_str {
        "Monday" => "lundi",
        "Tuesday" => "mardi",
        "Wednesday" => "mercredi",
        "Thursday" => "jeudi",
        "Friday" => "vendredi",
        "Saturday" => "samedi",
        "Sunday" => "dimanche",
        _ => "",
    };

    let day_str = match d_str {
        "1" => "1er",
        _ => d_str,
    };

    let month_str = match m_str {
        "January" => "janvier",
        "February" => "février",
        "March" => "mars",
        "April" => "avril",
        "May" => "mai",
        "June" => "juin",
        "July" => "juillet",
        "August" => "août",
        "September" => "septembre",
        "October" => "octobre",
        "November" => "novembre",
        "December" => "décembre",
        _ => "",
    };

    return format!("{} {} {} {}", weekday_str, day_str, month_str, y_str);
}

pub fn format_timestamp_into_string_times(timestamp: i32) -> String {
    let dt = DateTime::<Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp_opt(timestamp as i64, 0)
            .unwrap_or_else(|| panic!("Failed to convert timestamp to NaiveDateTime")),
        Utc,
    );

    let dt_string = dt.format("%H:%M").to_string();
    let (h_str, m_str) = dt_string.split_once(":").unwrap();

    let hours_str = match h_str {
        "00" => "0",
        "01" => "1",
        "02" => "2",
        "03" => "3",
        "04" => "4",
        "05" => "5",
        "06" => "6",
        "07" => "7",
        "08" => "8",
        "09" => "9",
        _ => h_str,
    };

    return format!("{}h{}", hours_str, m_str);
}

pub fn format_timestamp_into_slash_date(timestamp: i64) -> String {
    let dt = DateTime::<Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp_opt(timestamp + 3_600, 0).unwrap(),
        Utc,
    );

    let dt_string = dt.format("%A %d/%m").to_string();
    let (wd_str, other_part) = dt_string.split_once(" ").unwrap();

    let weekday_str = match wd_str {
        "Monday" => "lun",
        "Tuesday" => "mar",
        "Wednesday" => "mer",
        "Thursday" => "jeu",
        "Friday" => "ven",
        "Saturday" => "sam",
        "Sunday" => "dim",
        _ => "",
    };

    return format!("{} {}", weekday_str, other_part);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_format_timestamp_into_string_date() {
        let expected: String = String::from("jeudi 21 décembre 2023");
        let payload_in_sec = 1_703_113_200;
        let formatted = format_timestamp_into_string_date(payload_in_sec);
        assert_eq!(expected, formatted);
    }

    #[test]
    fn it_format_timestamp_into_slash_date() {
        let expected: String = String::from("jeu 21/12");
        let payload_in_sec = 1_703_113_200;
        let formatted = format_timestamp_into_slash_date(payload_in_sec);
        assert_eq!(&expected, &formatted);
    }

    #[test]
    fn it_format_timestamp_into_string_times() {
        let expected: String = String::from("9h00");
        let payload_in_sec = 32_400;
        let formatted = format_timestamp_into_string_times(payload_in_sec);
        assert_eq!(&expected, &formatted);
    }
}
