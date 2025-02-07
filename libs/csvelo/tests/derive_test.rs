use csvelo_derive::CSVParser;
use indoc::indoc;
use rayon::prelude::*;

#[test]
fn test_simple() {
    #[derive(CSVParser, Debug)]
    struct MyCsvData<'a> {
        a: Option<Vec<i32>>,
        b: Option<Vec<f32>>,
        c: Option<Vec<String>>,
        d: Option<Vec<&'a str>>,
        e: Option<Vec<i32>>,
    }

    let (data, records_num) = MyCsvData::from_csv_buffer(
        indoc! {r#"
            a,b,c,d,z
            1,2,3,4,5
            10,20,30,40,50
        "#}
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(records_num, 2);
    assert_eq!(data.a.unwrap(), vec![1, 10]);
    assert_eq!(data.b.unwrap(), vec![2.0, 20.0]);
    assert_eq!(data.c.unwrap(), vec!["3".to_string(), "30".to_string()]);
    assert_eq!(data.d.unwrap(), vec!["4", "40"]);
    assert!(data.e.is_none());
}
