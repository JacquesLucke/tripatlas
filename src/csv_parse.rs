use std::{collections::HashMap, ops::Range};

#[derive(Default)]
pub struct ParsedCsv {
    row_offsets: Vec<usize>,
    fields: Vec<Range<usize>>,
}

impl ParsedCsv {
    pub fn from_buffer(buffer: &[u8]) -> Self {
        let mut csv = ParsedCsv {
            ..Default::default()
        };
        csv.row_offsets.push(0);

        let mut start = 0;
        while start < buffer.len() {
            start = parse_row_fields(buffer, start, &mut csv.fields);
            csv.row_offsets.push(csv.fields.len());
        }

        csv
    }

    pub fn rows_len(&self) -> usize {
        self.row_offsets.len() - 1
    }

    pub fn row(&self, row: usize) -> ParsedCsvRow {
        let start = self.row_offsets[row];
        let end = self.row_offsets[row + 1];
        ParsedCsvRow {
            fields: &self.fields[start..end],
        }
    }

    pub fn headers<'a>(&self, buffer: &'a [u8]) -> Vec<&'a [u8]> {
        if self.rows_len() == 0 {
            return vec![];
        }
        self.row(0)
            .fields
            .iter()
            .map(|f| &buffer[f.clone()])
            .collect()
    }

    pub fn header_indices<'a>(&self, buffer: &'a [u8]) -> HashMap<&'a [u8], usize> {
        let mut map = HashMap::new();
        for (i, header) in self.headers(buffer).iter().enumerate() {
            map.insert(*header, i);
        }
        map
    }
}

pub struct ParsedCsvRow<'a> {
    fields: &'a [Range<usize>],
}

impl<'a> ParsedCsvRow<'a> {
    pub fn fields_len(&self) -> usize {
        self.fields.len()
    }

    pub fn field<'b>(&self, buffer: &'b [u8], field: usize) -> &'b [u8] {
        let range = &self.fields[field];
        &buffer[range.clone()]
    }
}

/// Adds all the fields of the current row and returns the first index after the row.
/// I.e. the index after the newline character or the end of the buffer.
/// The start index has to be the index of the first character in the row.
fn parse_row_fields(buffer: &[u8], start: usize, fields: &mut Vec<Range<usize>>) -> usize {
    let mut i = start;
    while i < buffer.len() {
        match buffer[i] {
            b'\n' => {
                return i + 1;
            }
            b'\r' => {
                i += 1;
            }
            b',' => {
                fields.push(i..i);
                i += 1;
                handle_potentially_trailing_comma(buffer, i, fields);
            }
            b'"' => {
                i += 1;
                let end_of_field = find_end_of_quoted_field(buffer, i);
                fields.push(i..end_of_field);
                i = end_of_field;
                while i < buffer.len() {
                    match buffer[i] {
                        b'"' => {
                            i += 1;
                        }
                        b',' => {
                            i += 1;
                            handle_potentially_trailing_comma(buffer, i, fields);
                            break;
                        }
                        b'\r' | b'\n' => {
                            break;
                        }
                        _ => {
                            i += 1;
                        }
                    }
                }
            }
            _ => {
                let end_of_field = find_end_of_simple_field(buffer, i);
                fields.push(i..end_of_field);
                i = end_of_field;
                while i < buffer.len() {
                    match buffer[i] {
                        b',' => {
                            i += 1;
                            handle_potentially_trailing_comma(buffer, i, fields);
                            break;
                        }
                        b'\r' | b'\n' => {
                            break;
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }
            }
        }
    }

    buffer.len()
}

fn handle_potentially_trailing_comma(buffer: &[u8], i: usize, fields: &mut Vec<Range<usize>>) {
    if i <= buffer.len() {
        if i < buffer.len() {
            if buffer[i] == b'\n' || buffer[i] == b'\r' {
                fields.push(i..i);
            }
        } else {
            fields.push(i..i);
        }
    }
}

/// Find the index that ends the current field (e.g. the index of the next comma or newline).
/// The start index has to be the index of the first character in the field.
/// It may also be the end of the field already if the field is empty.
fn find_end_of_simple_field(buffer: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < buffer.len() {
        match buffer[i] {
            b',' | b'\n' | b'\r' => {
                return i;
            }
            _ => {
                i += 1;
            }
        }
    }
    buffer.len()
}

/// Find the index of the quote that ends the current field.
/// The start index has to be the index after the opening quote.
fn find_end_of_quoted_field(buffer: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < buffer.len() {
        match buffer[i] {
            b'"' => {
                if i + 1 < buffer.len() && buffer[i + 1] == b'"' {
                    // Two consecutive quotes with in a quoted field are the escape sequence for a single quote.
                    i += 2;
                    continue;
                }
                return i;
            }
            b'\n' | b'\r' => {
                // Assume there is an end-quote missing.
                return i;
            }
            _ => {
                i += 1;
            }
        }
    }
    buffer.len()
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_find_end_of_simple_field() {
        assert_eq!(find_end_of_simple_field(b"123", 0), 3);
        assert_eq!(find_end_of_simple_field(b"123", 1), 3);
        assert_eq!(find_end_of_simple_field(b"123", 2), 3);
        assert_eq!(find_end_of_simple_field(b"123", 3), 3);
        assert_eq!(find_end_of_simple_field(b"1'3", 3), 3);
        assert_eq!(find_end_of_simple_field(b"123,", 0), 3);
        assert_eq!(find_end_of_simple_field(b"123,456", 0), 3);
        assert_eq!(find_end_of_simple_field(b"123,456,789", 0), 3);
        assert_eq!(find_end_of_simple_field(b" 23", 0), 3);
        assert_eq!(find_end_of_simple_field(b"", 0), 0);
        assert_eq!(find_end_of_simple_field(b"\n", 0), 0);
        assert_eq!(find_end_of_simple_field(b"12\n", 0), 2);
        assert_eq!(find_end_of_simple_field(b"0,12\n", 0), 1);
        assert_eq!(find_end_of_simple_field(b"0,12\n", 2), 4);
        assert_eq!(find_end_of_simple_field(b"\r\n", 0), 0);
        assert_eq!(find_end_of_simple_field(b"12\r\n", 0), 2);
        assert_eq!(find_end_of_simple_field(b"0,12\r\n", 0), 1);
        assert_eq!(find_end_of_simple_field(b"0,12\r\n", 2), 4);
    }

    #[test]
    fn test_find_end_of_quoted_field() {
        assert_eq!(find_end_of_quoted_field(b"", 0), 0);
        assert_eq!(find_end_of_quoted_field(b"123", 0), 3);
        assert_eq!(find_end_of_quoted_field(b"123\n", 0), 3);
        assert_eq!(find_end_of_quoted_field(b"123\r\n", 0), 3);
        assert_eq!(find_end_of_quoted_field(b"123\"", 0), 3);
        assert_eq!(find_end_of_quoted_field(b"\"", 0), 0);
        assert_eq!(find_end_of_quoted_field(b"\"\"", 0), 2);
        assert_eq!(find_end_of_quoted_field(b"123\"\"", 0), 5);
        assert_eq!(find_end_of_quoted_field(b"123\"\"\"", 0), 5);
        assert_eq!(find_end_of_quoted_field(b"123\"\"\"\"", 0), 7);
        assert_eq!(find_end_of_quoted_field(b"123\"\"\"\"\"", 0), 7);
        assert_eq!(find_end_of_quoted_field(b"123\"\"0\"\"\"", 0), 8);
        assert_eq!(find_end_of_quoted_field(b",", 0), 1);
        assert_eq!(find_end_of_quoted_field(b",\"", 0), 1);
        assert_eq!(find_end_of_quoted_field(b"0,1\"", 0), 3);
        assert_eq!(find_end_of_quoted_field(b"0,1\n", 0), 3);
        assert_eq!(find_end_of_quoted_field(b"0,1\"\"", 0), 5);
        assert_eq!(find_end_of_quoted_field(b"0,1\"\"\"", 0), 5);
    }

    fn get_parsed_row(buffer: &str) -> Vec<&str> {
        let mut fields = vec![];
        parse_row_fields(buffer.as_bytes(), 0, &mut fields);
        fields.iter().map(|f| &buffer[f.clone()]).collect()
    }

    #[test]
    fn test_parse_row() {
        assert!(get_parsed_row("").is_empty());
        assert_eq!(get_parsed_row("123,456,789"), &["123", "456", "789"]);
        assert_eq!(get_parsed_row("1,2,3\r\n123"), &["1", "2", "3"]);
        assert_eq!(
            get_parsed_row("1,\"this,is a \"\" test\",3\r\n123"),
            &["1", "this,is a \"\" test", "3"],
        );
        assert_eq!(get_parsed_row(","), &["", ""]);
        assert_eq!(get_parsed_row(",\n"), &["", ""]);
        assert_eq!(get_parsed_row(",\r\n"), &["", ""]);
        assert_eq!(get_parsed_row(",,,"), &["", "", "", ""]);
        assert_eq!(get_parsed_row(",,,3"), &["", "", "", "3"]);
        assert_eq!(get_parsed_row(" , , "), &[" ", " ", " "]);
        assert_eq!(get_parsed_row("\"\" "), &[""]);
        assert_eq!(get_parsed_row("0,"), &["0", ""]);
        assert_eq!(get_parsed_row("\" \","), &[" ", ""]);
    }

    #[test]
    fn test_parse_csv() {
        {
            let buffer = indoc! {"
                123,456,789
                1,2,3
            "}
            .as_bytes();
            let csv = ParsedCsv::from_buffer(buffer);
            assert_eq!(csv.rows_len(), 2);
            assert_eq!(csv.row(0).fields_len(), 3);
            assert_eq!(csv.row(0).field(buffer, 0), b"123");
            assert_eq!(csv.row(0).field(buffer, 1), b"456");
            assert_eq!(csv.row(0).field(buffer, 2), b"789");
            assert_eq!(csv.row(1).fields_len(), 3);
            assert_eq!(csv.row(1).field(buffer, 0), b"1");
            assert_eq!(csv.row(1).field(buffer, 1), b"2");
            assert_eq!(csv.row(1).field(buffer, 2), b"3");
        }
        {
            let buffer = indoc! {r#"
                stop_name,parent_station,stop_id,stop_lat,stop_lon,location_type
                's-Heerenberg Gouden Handen,,237383,51.87225,6.2473383,1
                "AB-Leider, Hafen",49745,35003,49.9727,9.107453,

                ,
                1,2
            "#}
            .as_bytes();
            let csv = ParsedCsv::from_buffer(buffer);
            assert_eq!(csv.rows_len(), 6);
            assert_eq!(csv.row(0).fields_len(), 6);
            assert_eq!(csv.row(0).field(buffer, 0), b"stop_name");
            assert_eq!(csv.row(0).field(buffer, 1), b"parent_station");
            assert_eq!(csv.row(0).field(buffer, 2), b"stop_id");
            assert_eq!(csv.row(0).field(buffer, 3), b"stop_lat");
            assert_eq!(csv.row(0).field(buffer, 4), b"stop_lon");
            assert_eq!(csv.row(0).field(buffer, 5), b"location_type");
            assert_eq!(csv.row(1).fields_len(), 6);
            assert_eq!(csv.row(1).field(buffer, 0), b"'s-Heerenberg Gouden Handen");
            assert_eq!(csv.row(1).field(buffer, 1), b"");
            assert_eq!(csv.row(1).field(buffer, 2), b"237383");
            assert_eq!(csv.row(1).field(buffer, 3), b"51.87225");
            assert_eq!(csv.row(1).field(buffer, 4), b"6.2473383");
            assert_eq!(csv.row(1).field(buffer, 5), b"1");
            assert_eq!(csv.row(2).fields_len(), 6);
            assert_eq!(csv.row(2).field(buffer, 0), b"AB-Leider, Hafen");
            assert_eq!(csv.row(2).field(buffer, 1), b"49745");
            assert_eq!(csv.row(2).field(buffer, 2), b"35003");
            assert_eq!(csv.row(2).field(buffer, 3), b"49.9727");
            assert_eq!(csv.row(2).field(buffer, 4), b"9.107453");
            assert_eq!(csv.row(2).field(buffer, 5), b"");
            assert_eq!(csv.row(3).fields_len(), 0);
            assert_eq!(csv.row(4).fields_len(), 2);
            assert_eq!(csv.row(4).field(buffer, 0), b"");
            assert_eq!(csv.row(4).field(buffer, 1), b"");
            assert_eq!(csv.row(5).fields_len(), 2);
            assert_eq!(csv.row(5).field(buffer, 0), b"1");
            assert_eq!(csv.row(5).field(buffer, 1), b"2");
        }
    }
}
