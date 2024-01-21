use chrono::{DateTime, Datelike, Utc};

pub fn timestamp_to_week_id(timestamp: i64) -> f64 {
    let dt = DateTime::<Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(timestamp, 0), Utc);

    let year = dt.year();
    let week_number = dt.iso_week().week();

    let result = f64::from(year) + f64::from(week_number) / 100.0;

    return (result * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_format_timestamp_to_float() {
        let expected = 2023.47;
        let payload_in_sec = 1_700_683_357;
        let formatted = timestamp_to_week_id(payload_in_sec);
        assert_eq!(&expected, &formatted);
    }
}
