use super::*;
use crate::translit::Translit as _;
use std::io::{BufRead, BufReader, Cursor};

#[test]
fn test_can_parse_complicated_address() {
    let res = address_number("36A/1").expect("parse the compilcated regular address");
    assert_eq!(res, ("", Number::from((36, Some("A/1")))));
}

#[test]
fn test_can_parse_a_range_of_addresses() {
    let res = address_number_range("123-321").expect("parse the range of addresses");
    assert_eq!(res, ("", Range::from((123, 321))))
}

#[test]
fn test_can_parse_one_of() {
    let res = broj("BB").expect("recognize BB");
    assert_eq!(res, ("", Building::Bb(None)));

    let res = broj("123A").expect("recognize an address number");
    assert_eq!(res, ("", Building::from(Number::from((123, Some("A"))))));

    let res = broj("123A-321B").expect("recognize an addresses range");
    assert_eq!(
        res,
        (
            "",
            Building::from(Range::from(((123, Some("A")), (321, Some("B")))))
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
                Building::Bb(None),
                Building::Number(Number::from(123)),
                Building::Range(Range::from((123, 321))),
            ]
        )
    )
}

#[test]
fn test_ignores_trailing_whitespaces() {
    let res = broj_list("   BB,BB   ").expect("rejects whitespaces before and after the sequence");
    assert_eq!(res, ("", vec![Building::Bb(None), Building::Bb(None),]))
}

#[test]
fn test_can_recognize_trailing_comma() {
    let res = broj_list("BB,").expect("parse the sequence of addresses followed by the trailing comma");
    assert_eq!(res, ("", vec![Building::Bb(None),]));
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
            Address::new(
                "AUTOPUT ZA NOVI SAD",
                vec![
                    Building::Bb(None),
                    Building::Number(Number::from(284)),
                    Building::Range(Range::from(((294, None), (296, Some("F"))))),
                ]
            )
        )
    );
}

#[test]
fn test_parse_multiple_addresses() {
    let (_, res) = address_row("autoput za novi sad: bb,284,294-296f,  batajnički drum: bb,261-265,269,303-303a,")
        .expect("parse the address row");

    assert_eq!(
        res,
        vec![
            Address::new(
                "autoput za novi sad",
                vec![
                    Building::Bb(None),
                    Building::from(Number::from(284)),
                    Building::from(Range::from(((294, None), (296, Some("f"))))),
                ]
            ),
            Address::new(
                "batajnički drum",
                vec![
                    Building::Bb(None),
                    Building::from(Range::from((261, 265))),
                    Building::from(Number::from(269)),
                    Building::from(Range::from(((303, None), (303, Some("a"))))),
                ]
            ),
        ]
    );
}

#[test]
fn test_settlement_parser() {
    let input = "naselje ripanj: put za marića kraj: 24,65-67, put za žuti potok: 14b,";
    let (_, res) = settlement(&input).expect("can parse settlement");
    assert_eq!(
        res,
        vec![
            Address::with_settlement(
                "ripanj",
                "put za marića kraj",
                vec![
                    Building::Number(Number::from(24)),
                    Building::Range(Range::from((Number::from(65), Number::from(67),))),
                ],
            ),
            Address::with_settlement(
                "ripanj",
                "put za žuti potok",
                vec![Building::Number(Number::from((14, Some("b"))))],
            ),
        ]
    );
}

#[test]
fn test_non_empty_bb() {
    static INPUT: &str = "drum: bb,bbimm stub-2,bbstub 10,bbstub-9,";
    let (_, res) = address_row(INPUT).expect("parse non empty BB");
    assert_eq!(
        res[0],
        Address::new(
            "drum",
            vec![
                Building::Bb(None),
                Building::Bb(Some("imm stub-2".to_string())),
                Building::Bb(Some("stub 10".to_string())),
                Building::Bb(Some("stub-9".to_string())),
            ]
        )
    )
}

#[test]
fn test_parse_timetable() {
    static DATA: &str = include_str!("timetable.txt");

    let reader = BufReader::new(Cursor::new(DATA));
    for line in reader.lines() {
        let trimmed_line = line.expect("read the line");
        if !trimmed_line.is_empty() {
            let input = trimmed_line.trim().translit();
            let (_, res) = address_row(&input).expect("parse real time-table");
            assert!(!res.is_empty())
        }
    }
}
