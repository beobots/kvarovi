//! The module declares structures to hold the address information and
//! a set of functions to parse the raw data.
//!
//! Addressing system in Serbija, seems like, allows buildings without assigned
//! number (see https://www.politika.rs/sr/clanak/398656/Srbija-bez-b-b-adresa)
//! and they are called BB (Bez Broj or without number).
#![allow(unused)]

use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until1};
use nom::character::complete::{alpha0, digit1, multispace0};
use nom::combinator::{map, map_res, opt, recognize, value};
use nom::error::Error;
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, pair, separated_pair};
use nom::{Err, IResult};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct BrojNumber {
    value: usize,
    extension: Option<String>,
}

impl From<(usize, Option<&str>)> for BrojNumber {
    fn from((v, e): (usize, Option<&str>)) -> Self {
        Self {
            value: v,
            extension: e.map(|s| s.to_owned()),
        }
    }
}

impl From<usize> for BrojNumber {
    fn from(v: usize) -> Self {
        Self {
            value: v,
            extension: None,
        }
    }
}

impl Display for BrojNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.extension {
            Some(ext) => write!(f, "{}{}", self.value, ext),
            None => write!(f, "{}", self.value),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct BrojRange {
    from: BrojNumber,
    to: BrojNumber,
}

impl From<(usize, usize)> for BrojRange {
    fn from((from, to): (usize, usize)) -> Self {
        Self {
            from: BrojNumber::from(from),
            to: BrojNumber::from(to),
        }
    }
}

impl From<((usize, Option<&str>), (usize, Option<&str>))> for BrojRange {
    fn from((from, to): ((usize, Option<&str>), (usize, Option<&str>))) -> Self {
        Self {
            from: BrojNumber::from(from),
            to: BrojNumber::from(to),
        }
    }
}

impl From<(BrojNumber, BrojNumber)> for BrojRange {
    fn from((from, to): (BrojNumber, BrojNumber)) -> Self {
        Self { from, to }
    }
}

impl Display for BrojRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.from, self.to)
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Broj {
    Bez,
    Number(BrojNumber),
    Range(BrojRange),
}

impl From<BrojNumber> for Broj {
    fn from(v: BrojNumber) -> Self {
        Broj::Number(v)
    }
}

impl From<BrojRange> for Broj {
    fn from(v: BrojRange) -> Self {
        Broj::Range(v)
    }
}

impl Display for Broj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Broj::Bez => write!(f, "BB"),
            Broj::Number(v) => write!(f, "{}", v),
            Broj::Range(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct AddressRecord {
    pub street: String,
    numbers: Vec<Broj>,
}

impl AddressRecord {
    pub fn new(street: &str, numbers: Vec<Broj>) -> Self {
        Self {
            street: street.to_owned(),
            numbers,
        }
    }
}

impl From<(&str, Vec<Broj>)> for AddressRecord {
    fn from((street, numbers): (&str, Vec<Broj>)) -> Self {
        Self {
            street: street.to_owned(),
            numbers,
        }
    }
}

impl Clone for AddressRecord {
    fn clone(&self) -> Self {
        Self {
            street: self.street.clone(),
            numbers: self.numbers.clone(),
        }
    }
}

impl Display for AddressRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:", self.street)?;

        for number in &self.numbers {
            write!(f, " {}", number)?;
        }

        Ok(())
    }
}

/// Parser a regular address number with optional extension letter.
fn address_number(input: &str) -> IResult<&str, BrojNumber> {
    let digit_parser = map_res(digit1, |s: &str| s.parse::<usize>());
    let ext_parser = map(
        recognize(pair(alpha0, opt(pair(tag("/"), digit1)))),
        |x: &str| if !x.is_empty() { Some(x) } else { None },
    );
    map(pair(digit_parser, ext_parser), BrojNumber::from)(input)
}

/// Parse a range of addresses
fn address_number_range(input: &str) -> IResult<&str, BrojRange> {
    let parser = separated_pair(address_number, tag("-"), address_number);
    map(parser, BrojRange::from)(input)
}

/// Parses an address number, a range of addresses or a special BB case.
fn broj(input: &str) -> IResult<&str, Broj> {
    let bb_parser = value(Broj::Bez, tag_no_case("bb"));
    let number_parser = map(address_number, Broj::from);
    let range_parser = map(address_number_range, Broj::from);

    alt((bb_parser, range_parser, number_parser))(input)
}

/// Recognizes a list of addresses, ranges of addresses or special BB cases.
fn broj_list(input: &str) -> IResult<&str, Vec<Broj>> {
    let parser = separated_list1(tag(","), broj);
    delimited(
        multispace0,
        // potentially we can simply skip the second element of the pair (the trailing comma)
        map(pair(parser, opt(tag(","))), |(x, _)| x),
        multispace0,
    )(input)
}

/// Recognizes a pair of an address and the list of addresses' numbers.
fn address_number_pair(input: &str) -> IResult<&str, AddressRecord> {
    let take_pp = take_until1(":");
    map(separated_pair(take_pp, tag(":"), broj_list), |(a, b)| {
        AddressRecord::new(a.trim(), b)
    })(input)
}

/// Parse addresses info (row).
fn addresses(input: &str) -> IResult<&str, Vec<AddressRecord>> {
    many1(address_number_pair)(input)
}

#[derive(Eq, PartialEq, Debug)]
pub struct Addresses {
    items: Vec<AddressRecord>,
}

impl Addresses {
    #[inline(always)]
    pub fn parse(input: &str) -> Result<Addresses, Err<Error<&str>>> {
        match addresses(input) {
            Ok((_, items)) => Ok(Self { items }),
            Err(err) => Err(err),
        }
    }
}

impl IntoIterator for Addresses {
    type Item = <Vec<AddressRecord> as IntoIterator>::Item;
    type IntoIter = <Vec<AddressRecord> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl Clone for Addresses {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}

impl Display for Addresses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.items {
            writeln!(f, "{}", item)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_can_parse_complicated_address() {
        let res = address_number("36A/1").expect("parse the compilcated regular address");
        assert_eq!(res, ("", BrojNumber::from((36, Some("A/1")))));
    }

    #[test]
    fn test_can_parse_a_range_of_addresses() {
        let res = address_number_range("123-321").expect("parse the range of addresses");
        assert_eq!(res, ("", BrojRange::from((123, 321))))
    }

    #[test]
    fn test_can_parse_one_of() {
        let res = broj("BB").expect("recognize BB");
        assert_eq!(res, ("", Broj::Bez));

        let res = broj("123A").expect("recognize an address number");
        assert_eq!(res, ("", Broj::from(BrojNumber::from((123, Some("A"))))));

        let res = broj("123A-321B").expect("recognize an addresses range");
        assert_eq!(
            res,
            (
                "",
                Broj::from(BrojRange::from(((123, Some("A")), (321, Some("B")))))
            )
        );
    }

    #[test]
    fn test_can_parse_numbers_sequences() {
        let res = broj_list("BB,123,123-321").expect("parse the sequence of numbers");
        assert_eq!(
            res,
            (
                "",
                vec![
                    Broj::Bez,
                    Broj::Number(BrojNumber::from(123)),
                    Broj::Range(BrojRange::from((123, 321))),
                ]
            )
        )
    }

    #[test]
    fn test_ignores_trailing_whitespaces() {
        let res = broj_list("   BB,BB   ").expect("rejects whitespaces before and after the sequence");
        assert_eq!(res, ("", vec![Broj::Bez, Broj::Bez,]))
    }

    #[test]
    fn test_can_recognize_trailing_comma() {
        let res = broj_list("BB,").expect("parse the sequence of addresses followed by the trailing comma");
        assert_eq!(res, ("", vec![Broj::Bez,]));
    }

    #[test]
    fn test_reject_simple_comma() {
        let res = broj("");
        assert!(res.is_err());

        let res = broj_list("   ,   ");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse() {
        let res = address_number_pair("  AUTOPUT ZA NOVI SAD  : BB,284,294-296F,").unwrap();
        assert_eq!(
            res,
            (
                "",
                AddressRecord::new(
                    "AUTOPUT ZA NOVI SAD",
                    vec![
                        Broj::Bez,
                        Broj::Number(BrojNumber::from(284)),
                        Broj::Range(BrojRange::from(((294, None), (296, Some("F"))))),
                    ]
                )
            )
        );
    }

    #[test]
    fn test_parse_all_addresses() {
        let res =
            addresses("AUTOPUT ZA NOVI SAD: BB,284,294-296F,  BATAJNIČKI DRUM: BB,261-265,269,283-293,299,303-303A,")
                .expect("parse the address row");

        assert_eq!(
            res,
            (
                "",
                vec![
                    AddressRecord::new(
                        "AUTOPUT ZA NOVI SAD",
                        vec![
                            Broj::Bez,
                            Broj::from(BrojNumber::from(284)),
                            Broj::from(BrojRange::from(((294, None), (296, Some("F"))))),
                        ]
                    ),
                    AddressRecord::new(
                        "BATAJNIČKI DRUM",
                        vec![
                            Broj::Bez,
                            Broj::from(BrojRange::from((261, 265))),
                            Broj::from(BrojNumber::from(269)),
                            Broj::from(BrojRange::from((283, 293))),
                            Broj::from(BrojNumber::from(299)),
                            Broj::from(BrojRange::from(((303, None), (303, Some("A"))))),
                        ]
                    ),
                ]
            )
        );
    }

    #[test]
    fn test_full_row_test() {
        static TEST_INPUT: &str = "AUTOPUT ZA NOVI SAD: BB,284,294-296F,  BATAJNIČKI DRUM: BB,261-265,269,283-293,299,303-303A,  BATAJNIČKI DRUM 14 DEO: 14,  NIKOLE SUKNJAREVIĆA PRIKE: 2-18,1-17, NASELJE BATAJNICA:   1 SREMSKOG ODREDA: 2-90,1-89,  AERODROMSKA: 68A-80,84-88I,98,1-1A,5-13,23A,  BANOVAČKA: 4A-12,20,24,28,34,1-1A,  BATAJNIČKIH ŽRTAVA: 2-16,1-13,  BATINSKE BITKE: 2-60,1-59,  BEČEJSKA: 22-26,30-32,42-44,  BIHAĆKA: 2-28,1-7,  BOJČINSKA: 1-15,  BOSANSKE KRAJINE: 2-86,1-73,  BRAĆE BARIŠIĆA: 2-18,1-3,7-19,  BRAĆE MIHAJLOVIĆ-TRIPIĆ: 6-106,43-45,49-51,  BRAĆE NEŠTINAC: 2-12,1-11,  BRAĆE RADIŠIĆ: 4A-4V,1-41,  BRAĆE SAVIĆA: 2-54,1-73,  BRAĆE SMILJANIĆA: 4-6,14-64,68-72,3-11,15-61A,65A-71,75B-83F/3,  BRAĆE VOJINOVIĆA: 1-1A,5-7A,11-11A,15-15A,19-19A,  BRANISLAVA BARIŠIĆA: 2-92,1-43,47,53-57,  BRILETOVA: 2-6,1-5,  BRODSKA: 2-18,1-19,  CARICE JELENE : 2-26,1-27,  DALMATINSKE ZAGORE : 8-16,20-160,1-145B,  DALMATINSKIH BRIGADA: 4A-6A,10-12,20,24,36A/1-56,60,19-21A,25-33,39-43,47,51,57-57,61-85,89-91,97-103,  DESPOTA IVANIŠA : 10-12,18-24,  DIMITRIJA LAZAROVA RAŠE: 2-28,1-33,37-41,  DISKONT PKB NOVA 21: 2-14,17,  ĐORĐA BOŠKOVIĆA - BATE: 6,12-16B,26-36A,42-54,3-19B,23-39,43-63B,  DRAGE MIHAJLOVIĆA: 2-58,1-47,51-53,  ĐURĐA BALŠIĆA : 2-6,10-20,3,9,  ISLAMA GRČKOG: 4,8,12-18,17,21-31,  IVANA DELNEGRA-ENGLEZA : 2-42,1-17,  IVANA SENKOVIĆA: 2-78,1-73,  JOVANA BRANKOVIĆA : 2-118,122-152,156-166C,170,174-176D,180-182,1-137,141-155,161-171,  KARLOVČIĆKA: 2-6,1-5,  KATICE OPAČIĆ: 2-18,22-40,44-46,50,72,76,94-94D,98-104D,1,5-11,17-17,25A-69B,  KESARA HRELJE : 2,12-24,28,34-40,  KESARA NOVAKA : 2-14A,  KESARA PRELJUBA : 4-8,12,20,24-26,30-36,3-9,13-25,  KESARA VOJIHNE : 4-6,3-23,27-33,  KLISINA NOVA  8: 2,3,7-17,  KLISINA NOVA  9: 2A,6,10,14,18-20,3-5,9-9A,13-17,  KNEZA PASKAČA : 2-14,18,1-5,  KRALJA MIHAILA ZETSKOG : 2-4,8-24,30-32,48-52,1-11,45-47,51-67O,73-83,87,  KRALJA RADOSLAVA : 38-120,126-148,152-178,53-81,85-85,99-99N,105-181,  KRALJA STEFANA TOMAŠA : 40-42,48-58,64-66,67-89,  KRALJA UROŠA PRVOG : 2-16G,1,9A,  KRALJA VLADISLAVA : 22-42,46-50B,54-102,106,110-116,120-150,13-29,33-35,39-43,47-61,65-73,77-117,121-129,133-139,  KULSKA: 23-29E,  MAJKE JUGOVIĆA: 16-16A,30-36,11-11E,99N,  MAJORA ZORANA RADOSAVLJEVIĆA : 2-50,116-226,236-258B,262-290,372-374,382,1-49,117-143,149-277,281,  MAKSIMA BRANKOVIĆA : 2-26,30,38-56,1-3,7-47,  MALA: 2-10,1,  MARKA PERIČINA-KAMENJARA : 2-8A,16,24-26,32,42-70,1,25,39-43,  MATROZOVA: BB,  MIHALJEVAČKA: 2-20,1-19,  MILICE RAKIĆ : 2-96,3-21,39-79,83-117,  MITROVAČKA: 2-26,1-27,  MRCINIŠTE NOVA 28: 2-10,14-16,24-36,3-27,  NATALIJE DUBAJIĆ: 2-6A,1-11,  NIKICE POPOVIĆA: 2-18,1-13,  NOVAKA ATANACKOVIĆA: 2-6,1-3,  NOVOSADSKA : 10-98,1-41,45-47,51-61D,65-75Ž,81D-81E,97G-99J,103A-109V,  OFICIRSKA KOLONIJA : 4-10,14-16,1-9,13-17,  PALIĆKA: 2-52,1-83,  PEĆINAČKA: 2-76,1-39,  PILOTSKA: 2-20,1-19,  PRIMOŠTENSKA: 3,11,19-21,  PUKOVNIKA MILENKA PAVLOVIĆA : 2-142,160-162,180,1-9A,13-127,143-159A,175,  RATARSKA: 2-42,1-39,  ROMSKA: 2,14-16,23,  SAVE GRKINIĆA: 2-30,1-33,43,  SAVE RADOVANOVIĆA: 2-2A,6-8A,12-12A,16,20-20A,1-5,15-17,  SEVASTOKRATORA BRANKA : 2-90,1-89,  SEVASTOKRATORA DEJANA : 2-36,40,1,9-43,47-49,  SEVASTOKRATORA VLATKA : 2-68,1-79,  ŠIMANOVAČKA: 2-80,1-55,  ŠIROKI PUT: 2-16A,36,1-19,31E-31K,  ŠKOLSKA: 2-6,1-5,  SLOBODANA MACURE : 2-4,8-12,1-15,33-37,41-69,  SREMSKOG FRONTA: 2-20,1-9,13-25,  STANKA TIŠME: 2-84G,31A-47,71B-85V,  STEVANA DUBAJIĆA : 2A-42,46-48,52-68,74-82,1-17,21-29,33-73,79-81,85-91,  STEVE STANKOVIĆA: 2-18,1-11,  STOJANA BOŠKOVIĆA : 1,5-17,  SUNCOKRETA: 2-6,10-14,20,24,28-30,  SVETISLAVA VULOVIĆA : 4,10,14-18,27-33,37,  SVETOG RAFAILA ŠIŠATOVAČKOG : 2-12,1-15,  SVETOG SERAFIMA SAROVSKOG : 2-12,1-15,  SVILAJSKA: 2-12,1-9,  TITELSKA: 10-12,18-20,  VASILIJA RANKOVIĆA-BAĆE: 2-12,1-5,9-19,  VERE MIŠČEVIĆ: 2-30,5-15,19-33,  VOJVOĐANSKIH BRIGADA: 2-34,44-134,1-37,41-87,91-139A,143-145Z,  VOJVODE JAKŠE : 2-10,  VOJVODE NOVAKA : 2-6,10,44-46E,1-29F,33-39V,55M-55N,61B,  VOJVODE VOJISLAVA VOJINOVIĆA : 2-16,1-9,  VOJVODE VRATKA : BB,4-28,1A-37,  ŽARKA BOKUNA: 2-104,108-110,1-11,49,61-103,121-129,  ŽARKA OBREŠKOG: 2-14,18-20,30,34-34,38-40,44,1A-29E,33-41B,  ŽIKE MARKOVIĆA: 2-10,1-13,  ŽUPANA PRIBILA : 2-36,1-31, NASELJE ZEMUN:   BATAJNIČKI DRUM 13 DEO: 301,  KLISINA NOVA 10: 8-10,  TEMERINSKA 1 DEO: 1,";
        let res = addresses(TEST_INPUT);
        assert!(res.is_ok())
    }
}
