//! Module to parse time interval from the electricity maintenance schedule.
use chrono::NaiveTime;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{map, map_res};
use nom::error::Error;
use nom::sequence::separated_pair;
use nom::{Err, IResult};

#[derive(Eq, PartialEq, Debug)]
pub struct TimeInterval {
    from: NaiveTime,
    to: NaiveTime,
}

impl TimeInterval {
    #[allow(unused)]
    pub fn new(from: NaiveTime, to: NaiveTime) -> Self {
        Self { from, to }
    }

    #[allow(unused)]
    pub fn parse(input: &str) -> Result<Self, Err<Error<&str>>> {
        let (_, result) = parse_interval(input)?;
        Ok(result)
    }
}

impl From<(NaiveTime, NaiveTime)> for TimeInterval {
    fn from((from, to): (NaiveTime, NaiveTime)) -> Self {
        Self { from, to }
    }
}

fn digit_parse(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse::<u32>)(input)
}

fn parse_time(input: &str) -> IResult<&str, NaiveTime> {
    map_res(
        separated_pair(digit_parse, tag(":"), digit_parse),
        |(hh, mm)| NaiveTime::from_hms_opt(hh, mm, 0).ok_or("invalid native time"),
    )(input)
}

fn parse_interval(input: &str) -> IResult<&str, TimeInterval> {
    map(
        separated_pair(parse_time, tag("-"), parse_time),
        TimeInterval::from,
    )(input)
}

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_fail_to_parse_malformed_time(h in 25..99, m in 61..99) {
            let time_str = format!("{h:02}:{m:02}");
            prop_assert!(parse_time(&time_str).is_err());
        }

        #[test]
        fn test_parse_time(h in 0u32..24u32, m in 0u32..60u32) {
            let (_, time) = parse_time(&format!("{h:02}:{m:02}")).expect("parse time");
            prop_assert_eq!(time, NaiveTime::from_hms_opt(h, m, 00).expect("parse the time"));
        }

        #[test]
        fn test_parse_interval(h1 in 0u32..24u32, m1 in 0u32..60u32, h2 in 0u32..24u32, m2 in 0u32..60u32) {
            let time_string = format!("{h1:02}:{m1:02}-{h2:02}:{m2:02}");
            let time_range = TimeInterval::parse(&time_string).expect("parse time interval");
            prop_assert_eq!(
                time_range,
                TimeInterval::new(
                    NaiveTime::from_hms_opt(h1, m1, 0).expect("parse the time"),
                    NaiveTime::from_hms_opt(h2, m2, 0).expect("parse the time")
                )
            );
        }
    }
}
