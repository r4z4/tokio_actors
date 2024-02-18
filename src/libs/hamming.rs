use struct_iterable::Iterable;

pub fn hamming_distance(x: impl Iterable, y: impl Iterable) -> Option<usize> {
    let x_ct = x.iter().count();
    let y_ct = y.iter().count();

    if x_ct != y_ct {
        None
    } else {
        Some(2)
    }
}

#[derive(Debug, Clone, Iterable)]
pub struct Record {
    pub name: String,
    pub count: i32,
    pub quote: String,
}

#[derive(Debug, Clone, Iterable)]
pub struct LongerRecord {
    pub name: String,
    pub count: i32,
    pub quote: String,
    pub extra_quote: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unequal_lengths_return_none() {
        let x_eg: Record = Record {
            name: "Name".to_owned(),
            count: 44,
            quote: "What".to_owned(),
        };
        let y_eg: Record = Record {
            name: "Jim".to_owned(),
            count: 3,
            quote: "Huh".to_owned(),
        };
        let y_long_eg: LongerRecord = LongerRecord {
            name: "Steve".to_owned(),
            count: 2,
            quote: "Whater".to_owned(),
            extra_quote: "Whaterer".to_owned(),
        };
        let res = hamming_distance(x_eg, y_long_eg);
        assert_eq!(res, None);
    }
    // #[test]
    // fn test_date_convert() {
    //     let converted_date = convert_date(DATE_STR).unwrap();
    //     assert_eq!(converted_date, NaiveDate::from_ymd_opt(2023, 8, 25).unwrap());
    // }
}
