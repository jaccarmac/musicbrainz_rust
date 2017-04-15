use std::str::FromStr;
use std::num::ParseIntError;

/// The `Date` type used by the `musicbrainz` crate.
/// It allows the representation of partial dates.
// TODO: Write conversions to and from `chrono` date types for interoperability.
// TODO: Consider checking the field values for validity (i.e. month and day within appropriate
// ranges). To make sure only valid instances are created we might actually need to do something
// like it is described here: http://stackoverflow.com/a/28090996 because in general Rust enum
// constructors cannot be made private.
// (And for the users of the `Date` type it actually shouldn't even matter that much if they can
// pattern match on it or not, it's just more about properly representing the data returned from
// the MusicBrainz API.)
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Date {
    /// Date with resolution up to a year.
    Year { year: u16 },
    /// Date with resolution up to a month.
    Month { year: u16, month: u8 },
    /// Date with resolution up to a day.
    /// year=0 <=> year 0 in
    Day { year: u16, month: u8, day: u8 },
}

impl Date {
    /// Return the year from the date.
    pub fn year(&self) -> u16 {
        match *self {
            Date::Year { year } => year,
            Date::Month { year, .. } => year,
            Date::Day { year, .. } => year,
        }
    }

    /// Return the month from the date.
    /// If it is not present, 0 will be returned.
    pub fn month(&self) -> u8 {
        match *self {
            Date::Year { .. } => 0,
            Date::Month { month, .. } => month,
            Date::Day { month, .. } => month,
        }
    }

    /// Return the day from the date.
    /// If it is not present, 0 will be returned.
    pub fn day(&self) -> u8 {
        match *self {
            Date::Year { .. } => 0,
            Date::Month { .. } => 0,
            Date::Day { day, .. } => day,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ParseDateError {
    /// A wrong number of `-` separated components was found in the string.
    WrongNumberOfComponents(usize),

    /// Failed parsing a component into the appropriate number type.
    ComponentInvalid(ParseIntError),
}

impl From<ParseIntError> for ParseDateError {
    fn from(e: ParseIntError) -> Self {
        ParseDateError::ComponentInvalid(e)
    }
}

impl FromStr for Date {
    type Err = ParseDateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Get the pieces of the date.
        let ps: Vec<&str> = s.split("-").collect();

        // Create result.
        if ps.len() == 1 {
            Ok(Date::Year { year: ps[0].parse()? })
        } else if ps.len() == 2 {
            Ok(Date::Month {
                   year: ps[0].parse()?,
                   month: ps[1].parse()?,
               })
        } else if ps.len() == 3 {
            Ok(Date::Day {
                   year: ps[0].parse()?,
                   month: ps[1].parse()?,
                   day: ps[2].parse()?,
               })
        } else {
            Err(ParseDateError::WrongNumberOfComponents(ps.len()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        let date1 = Date::from_str("2017").unwrap();
        let date2 = Date::from_str("2017-4").unwrap();
        let date3 = Date::from_str("2017-04-15").unwrap();

        assert_eq!(date1.year(), 2017);
        assert_eq!(date1.month(), 0);
        assert_eq!(date1.day(), 0);
        assert_eq!(date2.year(), 2017);
        assert_eq!(date2.month(), 4);
        assert_eq!(date2.day(), 0);
        assert_eq!(date3.year(), 2017);
        assert_eq!(date3.month(), 4);
        assert_eq!(date3.day(), 15);
    }

    #[test]
    fn wrong_number_comps() {
        let fail = Date::from_str("1-1-1-1");
        assert_eq!(fail.err().unwrap(),
                   ParseDateError::WrongNumberOfComponents(4));
    }

    #[test]
    fn invalid_components() {
        let fail1 = Date::from_str("abc");
        let fail2 = Date::from_str("2017-abc");
        let fail3 = Date::from_str("2017-04-abc");

        let err = ParseDateError::from("abc".parse::<u16>().err().unwrap());

        assert_eq!(fail1.err().unwrap(), err);
        assert_eq!(fail2.err().unwrap(), err);
        assert_eq!(fail3.err().unwrap(), err);
    }
}
