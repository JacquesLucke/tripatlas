use csvelo_derive::CSVParser;
use indoc::indoc;
use rayon::prelude::*;

#[test]
fn test_simple() {
    #[derive(CSVParser, Debug)]
    struct MyCsvData {
        a: Vec<i32>,
        b: Vec<i32>,
        c: Option<Vec<i32>>,
    }

    let data = MyCsvData::from_csv_buffer(
        indoc! {r#"
            a,b,c
            1,2,3
            4,5,6
        "#}
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(data.a.len(), 2);
    assert_eq!(data.b.len(), 2);
    assert_eq!(data.c.unwrap().len(), 2);
}

#[test]
fn test_missing_optional_column() {
    #[derive(CSVParser, Debug)]
    struct MyCsvData {
        a: Vec<i32>,
        b: Option<Vec<i32>>,
    }

    let data = MyCsvData::from_csv_buffer(
        indoc! {r#"
            a
            1
            4
        "#}
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(data.a.len(), 2);
    assert!(data.b.is_none());
}

#[test]
fn test_required_optional_column() {
    #[derive(CSVParser, Debug)]
    struct MyCsvData {
        a: Vec<i32>,
        c: Option<Vec<i32>>,
    }

    let data = MyCsvData::from_csv_buffer(
        indoc! {r#"
            c
            2
            5
        "#}
        .as_bytes(),
    );
    assert!(data.is_err());
}

#[test]
fn test_extra_column() {
    #[derive(CSVParser, Debug)]
    struct MyCsvData {
        a: Vec<i32>,
    }

    let data = MyCsvData::from_csv_buffer(
        indoc! {r#"
            q,a,b
            0,1,2
            0,4,5
        "#}
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(data.a.len(), 2);
}

#[test]
fn test_with_ref() {
    #[derive(CSVParser)]
    struct MyCsvData<'a> {
        data: Vec<&'a str>,
    }

    let bytes = indoc! {r#"
        data
        why
        where
    "#}
    .as_bytes();

    let data = MyCsvData::from_csv_buffer(bytes).unwrap();
    assert_eq!(data.data.len(), 2);
}
