/// Adds all the fields of the current record and returns the first index after the record.
/// I.e. the index after the newline character or the end of the buffer.
/// The start index has to be the index of the first character in the record.
pub fn parse_record_fields<'a>(
    buffer: &'a [u8],
    start: usize,
    fields: &mut Vec<&'a [u8]>,
) -> usize {
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
                fields.push(b"");
                i += 1;
                handle_potentially_trailing_comma(buffer, i, fields);
            }
            b'"' => {
                i += 1;
                let end_of_field = find_end_of_quoted_field(buffer, i);
                fields.push(&buffer[i..end_of_field]);
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
                fields.push(&buffer[i..end_of_field]);
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

fn handle_potentially_trailing_comma<'a>(buffer: &'a [u8], i: usize, fields: &mut Vec<&'a [u8]>) {
    if i <= buffer.len() {
        if i < buffer.len() {
            if buffer[i] == b'\n' || buffer[i] == b'\r' {
                fields.push(b"");
            }
        } else {
            fields.push(b"");
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

/// Finds the index of the first character in the next record, or the end of the buffer
/// if there is no next record.
pub fn find_start_of_next_record(buffer: &[u8], start: usize) -> usize {
    if let Some(newline_offset) = buffer[start..].iter().position(|&c| c == b'\n') {
        start + newline_offset + 1
    } else {
        buffer.len()
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_parse_record() {
        assert!(get_parsed_record("").is_empty());
        assert_eq!(get_parsed_record("123,456,789"), &["123", "456", "789"]);
        assert_eq!(get_parsed_record("1,2,3\r\n123"), &["1", "2", "3"]);
        assert_eq!(
            get_parsed_record("1,\"this,is a \"\" test\",3\r\n123"),
            &["1", "this,is a \"\" test", "3"],
        );
        assert_eq!(get_parsed_record(","), &["", ""]);
        assert_eq!(get_parsed_record(",\n"), &["", ""]);
        assert_eq!(get_parsed_record(",\r\n"), &["", ""]);
        assert_eq!(get_parsed_record(",,,"), &["", "", "", ""]);
        assert_eq!(get_parsed_record(",,,3"), &["", "", "", "3"]);
        assert_eq!(get_parsed_record(" , , "), &[" ", " ", " "]);
        assert_eq!(get_parsed_record("\"\" "), &[""]);
        assert_eq!(get_parsed_record("0,"), &["0", ""]);
        assert_eq!(get_parsed_record("\" \","), &[" ", ""]);
    }

    fn get_parsed_record(buffer: &str) -> Vec<&str> {
        let mut fields = vec![];
        parse_record_fields(buffer.as_bytes(), 0, &mut fields);
        fields
            .iter()
            .map(|f| std::str::from_utf8(f).unwrap())
            .collect()
    }
}
