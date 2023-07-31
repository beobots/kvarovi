//! The module declares structures to hold the address information and
//! a set of functions to parse the raw data.
//!
//! Addressing system in Serbija, seems like, allows buildings without assigned
//! number (see https://www.politika.rs/sr/clanak/398656/Srbija-bez-b-b-adresa)
//! and they are called BB (Bez Broj or without number).
#![allow(unused)]
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until1};
use nom::character::complete::{alpha0, alpha1, digit0, digit1, multispace0, space0, space1};
use nom::combinator::{all_consuming, map, map_res, not, opt, peek, recognize, value};
use nom::error::Error;
use nom::error::VerboseError;
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::{Err, IResult};
use serde::Serialize;
use std::fmt::{write, Display};

type AddrError<'a> = VerboseError<&'a str>;

#[derive(Eq, PartialEq, Clone, Debug, Serialize)]
pub(crate) struct Number {
    value: usize,
    extension: Option<String>,
}

impl From<(usize, Option<&str>)> for Number {
    fn from((v, e): (usize, Option<&str>)) -> Self {
        Self {
            value: v,
            extension: e.map(|s| s.to_owned()),
        }
    }
}

impl From<usize> for Number {
    fn from(v: usize) -> Self {
        Self {
            value: v,
            extension: None,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.extension {
            Some(ext) => write!(f, "{}{}", self.value, ext),
            None => write!(f, "{}", self.value),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize)]
pub(crate) struct Range {
    from: Number,
    to: Number,
}

impl From<(usize, usize)> for Range {
    fn from((from, to): (usize, usize)) -> Self {
        Self {
            from: Number::from(from),
            to: Number::from(to),
        }
    }
}

impl From<((usize, Option<&str>), (usize, Option<&str>))> for Range {
    fn from((from, to): ((usize, Option<&str>), (usize, Option<&str>))) -> Self {
        Self {
            from: Number::from(from),
            to: Number::from(to),
        }
    }
}

impl From<(Number, Number)> for Range {
    fn from((from, to): (Number, Number)) -> Self {
        Self { from, to }
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.from, self.to)
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize)]
pub(crate) enum Building {
    /// A building without number (Bez Broj).
    Bb(Option<String>),
    Number(Number),
    Range(Range),
}

impl From<Number> for Building {
    fn from(v: Number) -> Self {
        Building::Number(v)
    }
}

impl From<Range> for Building {
    fn from(v: Range) -> Self {
        Building::Range(v)
    }
}

impl Display for Building {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Building::Bb(None) => write!(f, "BB"),
            Building::Bb(Some(v)) => write!(f, "BB {v}"),
            Building::Number(v) => write!(f, "{v}"),
            Building::Range(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub(crate) struct Address {
    pub settlement: Option<String>,
    pub street: String,
    pub buildings: Vec<Building>,
}

impl Address {
    fn new(street: &str, numbers: Vec<Building>) -> Self {
        Self {
            settlement: None,
            street: street.to_owned(),
            buildings: numbers,
        }
    }

    fn with_settlement(settlement: &str, street: &str, numbers: Vec<Building>) -> Self {
        Self {
            settlement: Some(settlement.to_owned()),
            street: street.to_owned(),
            buildings: numbers,
        }
    }

    fn add_settlement((name, its): (&str, Vec<Self>)) -> Vec<Self> {
        its.into_iter()
            .map(|it| Self {
                settlement: Some(name.to_string()),
                ..it
            })
            .collect()
    }
}

impl From<(&str, Vec<Building>)> for Address {
    fn from((street, numbers): (&str, Vec<Building>)) -> Self {
        Self {
            settlement: None,
            street: street.to_owned(),
            buildings: numbers,
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.street)?;
        for number in &self.buildings {
            write!(f, "{},", number)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AddressRow {
    items: Vec<Address>,
}

impl AddressRow {
    pub fn parse(input: &str) -> anyhow::Result<Self> {
        let r = address_row(input)
            .map(|(_, items)| Self { items })
            .map_err(|e| match e {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    anyhow::Error::msg(nom::error::convert_error(input, err))
                }
                nom::Err::Incomplete(_) => unreachable!("incomplete address string"),
            })?;

        Ok(r)
    }
}

impl IntoIterator for AddressRow {
    type Item = <Vec<Address> as IntoIterator>::Item;
    type IntoIter = <Vec<Address> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl Display for AddressRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.items {
            writeln!(f, "{}", item)?;
        }

        Ok(())
    }
}

/// Parser a regular address number with optional extension letter.
fn address_number(input: &str) -> IResult<&str, Number, AddrError> {
    let digit_parser = map_res(digit1, |s: &str| s.parse::<usize>());
    let ext_parser = map(
        recognize(pair(alpha0, opt(pair(tag("/"), digit1)))),
        |x: &str| if !x.is_empty() { Some(x) } else { None },
    );
    map(pair(digit_parser, ext_parser), Number::from)(input)
}

/// Parse a range of addresses
fn address_number_range(input: &str) -> IResult<&str, Range, AddrError> {
    let parser = separated_pair(address_number, tag("-"), address_number);
    map(parser, Range::from)(input)
}

fn bez_broj(input: &str) -> IResult<&str, Building, AddrError> {
    let alpha_digit = alt((alpha1, digit1));

    let bb_ext = opt(recognize(pair(
        many1(alpha_digit),
        many0(alt((tag("-"), space1, alpha1, digit1))),
    )));

    map(
        preceded(tag_no_case("bb"), bb_ext),
        |extension: Option<&str>| {
            let maybe_ext = extension.map(|it| it.to_owned());
            Building::Bb(maybe_ext)
        },
    )(input)
}

/// Parses an address number, a range of addresses or a special BB case.
fn broj(input: &str) -> IResult<&str, Building, AddrError> {
    // let bb_parser = value(Building::Bb(None), tag_no_case("bb"));
    let number_parser = map(address_number, Building::from);
    let range_parser = map(address_number_range, Building::from);

    alt((bez_broj, range_parser, number_parser))(input)
}

/// Recognizes a list of addresses, ranges of addresses or special BB cases.
fn broj_list(input: &str) -> IResult<&str, Vec<Building>, AddrError> {
    let parser = separated_list1(tag(","), broj);
    delimited(
        multispace0,
        // potentially we can simply skip the second element of the pair (the trailing comma)
        map(pair(parser, opt(tag(","))), |(x, _)| x),
        multispace0,
    )(input)
}

/// Recognizes a pair of an address and the list of addresses' numbers.
fn address_number_pair(input: &str) -> IResult<&str, Address, AddrError> {
    let take_pp = take_until1(":");
    map(separated_pair(take_pp, tag(":"), broj_list), |(a, b)| {
        Address::new(a.trim(), b)
    })(input)
}

fn settlement(input: &str) -> IResult<&str, Vec<Address>, AddrError> {
    static SETTLEMENT: &str = "naselje";
    let address = preceded(peek(not(tag_no_case(SETTLEMENT))), address_number_pair);
    let addresses = many1(address);
    let settlement_name = map(
        separated_pair(tag_no_case(SETTLEMENT), space1, take_until1(":")),
        |(_, it)| it,
    );
    map(
        separated_pair(settlement_name, tag(":"), addresses),
        Address::add_settlement,
    )(input)
}

fn address_kind(input: &str) -> IResult<&str, Vec<Address>, AddrError> {
    let addr_parser = map(address_number_pair, |it| vec![it]);
    alt((settlement, addr_parser))(input)
}

fn address_row(input: &str) -> IResult<&str, Vec<Address>, AddrError> {
    all_consuming(map(many1(address_kind), |it| {
        it.into_iter().flatten().collect()
    }))(input)
}

#[cfg(test)]
mod tests;
