use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[inline]
pub fn remove_prefix(date_str: &str) -> &str {
    date_str.trim_start_matches("Reviewed ")
}

fn date_str_into_parts(date_str: &str) -> Option<(&str, &str, &str)> {
    let mut parts = date_str.split_whitespace();
    let month = parts.next()?;
    let day = parts.next()?;
    let year = parts.next()?;
    Some((month, day, year))
}

#[derive(Debug, Deserialize, Serialize)]
#[repr(u32)]
enum DirtyMonth {
    Jan = 1,
    Feb = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    Aug = 8,
    Sep = 9,
    Oct = 10,
    Nov = 11,
    Dec = 12
}

impl Into<u32> for DirtyMonth {
    fn into(self) -> u32 {
        self as u32
    }
}

// Appears in the messy CSV as 'Jan.'
impl std::str::FromStr for DirtyMonth {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Jan." => Ok(DirtyMonth::Jan),
            "Feb." => Ok(DirtyMonth::Feb),
            "March" => Ok(DirtyMonth::March),
            "April" => Ok(DirtyMonth::April),
            "May" => Ok(DirtyMonth::May),
            "June" => Ok(DirtyMonth::June),
            "July" => Ok(DirtyMonth::July),
            "Aug." => Ok(DirtyMonth::Aug),
            "Sep." => Ok(DirtyMonth::Sep),
            _ => Err("Invalid Month"),
        }
    }
}

pub fn convert_date(date_str: &str) -> Option<NaiveDate> {
    let date_str = remove_prefix(date_str);
    let (m,d,y) = date_str_into_parts(date_str)?;
    let month = m.parse::<DirtyMonth>().ok().unwrap().into();
    let day = d.trim_end_matches(",").parse::<u32>().ok()?;
    let year = y.parse::<i32>().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    const DATE_STR: &str = "Reviewed Aug 25, 2023";
    #[test]
    fn test_remove_prefix() {
        let date_str = remove_prefix(DATE_STR);
        assert_eq!(date_str, "Aug 25, 2023");
    }
    #[test]
    fn test_date_convert() {
        let converted_date = convert_date(DATE_STR).unwrap();
        assert_eq!(converted_date, NaiveDate::from_ymd_opt(2023, 8, 25).unwrap());
    }
}