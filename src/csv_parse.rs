use std::str::Utf8Error;

#[derive(Default)]
pub struct CsvRows<'b> {
    row_offsets: Vec<usize>,
    fields: Vec<&'b [u8]>,
}

pub struct CsvRowsIter<'r, 'b> {
    rows: &'r CsvRows<'b>,
    row_i: usize,
}

pub struct CsvBufferSections<'a> {
    pub header: &'a [u8],
    pub data: &'a [u8],
}

pub fn parse_header_row_str(header_row: &[u8]) -> std::result::Result<Vec<&str>, Utf8Error> {
    let mut fields = vec![];
    parse_row_fields(header_row, 0, &mut fields);
    fields.iter().map(|f| std::str::from_utf8(f)).collect()
}

pub fn split_header_and_data(buffer: &[u8]) -> CsvBufferSections {
    let header_end_i = buffer
        .iter()
        .position(|&b| b == b'\n')
        .unwrap_or(buffer.len());
    let header_buffer = &buffer[..header_end_i];
    let data_buffer = &buffer[header_end_i + 1..];
    CsvBufferSections {
        header: header_buffer,
        data: data_buffer,
    }
}

pub fn split_csv_buffer_into_line_aligned_chunks(
    buffer: &[u8],
    approximate_chunk_size: usize,
) -> Vec<&[u8]> {
    let mut chunks = vec![];
    let mut next_chunk_start = 0;
    while next_chunk_start < buffer.len() {
        let chunk_end = (next_chunk_start + approximate_chunk_size).min(buffer.len());
        if let Some(newline_offset) = buffer[chunk_end..].iter().position(|&c| c == b'\n') {
            let chunk_end = chunk_end + newline_offset + 1;
            chunks.push(&buffer[next_chunk_start..chunk_end]);
            next_chunk_start = chunk_end;
        } else {
            chunks.push(&buffer[next_chunk_start..]);
            break;
        }
    }

    chunks
}

impl<'r, 'b> Iterator for CsvRowsIter<'r, 'b> {
    type Item = CsvRow<'r, 'b>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row_i + 1 >= self.rows.row_offsets.len() {
            return None;
        }
        let row = self.rows.row(self.row_i);
        self.row_i += 1;
        Some(row)
    }
}

impl<'b> CsvRows<'b> {
    pub fn from_buffer(buffer: &'b [u8]) -> Self {
        let mut row_offsets = vec![];
        let mut fields = vec![];

        row_offsets.push(0);
        let mut start = 0;
        while start < buffer.len() {
            start = parse_row_fields(buffer, start, &mut fields);
            row_offsets.push(fields.len());
        }
        CsvRows {
            row_offsets,
            fields,
        }
    }

    pub fn iter<'r>(&'r self) -> CsvRowsIter<'r, 'b> {
        CsvRowsIter {
            rows: self,
            row_i: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.row_offsets.len() - 1
    }

    pub fn row<'r>(&'r self, row: usize) -> CsvRow<'r, 'b> {
        let start = self.row_offsets[row];
        let end = self.row_offsets[row + 1];
        CsvRow {
            fields: &self.fields[start..end],
        }
    }
}

pub struct CsvRow<'r, 'b> {
    pub fields: &'r [&'b [u8]],
}

impl<'r, 'b> CsvRow<'r, 'b> {
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn column(&self, column_i: usize) -> Option<&'b [u8]> {
        self.fields.get(column_i).copied()
    }
}

/// Adds all the fields of the current row and returns the first index after the row.
/// I.e. the index after the newline character or the end of the buffer.
/// The start index has to be the index of the first character in the row.
fn parse_row_fields<'a>(buffer: &'a [u8], start: usize, fields: &mut Vec<&'a [u8]>) -> usize {
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
        fields
            .iter()
            .map(|f| std::str::from_utf8(f).unwrap())
            .collect()
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
    fn test_parse_csv_buffer() {
        {
            let buffer = indoc! {"
                123,456,789
                1,2,3
            "}
            .as_bytes();
            let rows = CsvRows::from_buffer(buffer);
            assert_eq!(rows.len(), 2);
            assert_eq!(rows.row(0).len(), 3);
            assert_eq!(rows.row(0).column(0).unwrap(), b"123");
            assert_eq!(rows.row(0).column(1).unwrap(), b"456");
            assert_eq!(rows.row(0).column(2).unwrap(), b"789");
            assert_eq!(rows.row(1).len(), 3);
            assert_eq!(rows.row(1).column(0).unwrap(), b"1");
            assert_eq!(rows.row(1).column(1).unwrap(), b"2");
            assert_eq!(rows.row(1).column(2).unwrap(), b"3");
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
            let csv = CsvRows::from_buffer(buffer);
            assert_eq!(csv.len(), 6);
            assert_eq!(csv.row(0).len(), 6);
            assert_eq!(csv.row(0).column(0).unwrap(), b"stop_name");
            assert_eq!(csv.row(0).column(1).unwrap(), b"parent_station");
            assert_eq!(csv.row(0).column(2).unwrap(), b"stop_id");
            assert_eq!(csv.row(0).column(3).unwrap(), b"stop_lat");
            assert_eq!(csv.row(0).column(4).unwrap(), b"stop_lon");
            assert_eq!(csv.row(0).column(5).unwrap(), b"location_type");
            assert_eq!(csv.row(1).len(), 6);
            assert_eq!(
                csv.row(1).column(0).unwrap(),
                b"'s-Heerenberg Gouden Handen"
            );
            assert_eq!(csv.row(1).column(1).unwrap(), b"");
            assert_eq!(csv.row(1).column(2).unwrap(), b"237383");
            assert_eq!(csv.row(1).column(3).unwrap(), b"51.87225");
            assert_eq!(csv.row(1).column(4).unwrap(), b"6.2473383");
            assert_eq!(csv.row(1).column(5).unwrap(), b"1");
            assert_eq!(csv.row(2).len(), 6);
            assert_eq!(csv.row(2).column(0).unwrap(), b"AB-Leider, Hafen");
            assert_eq!(csv.row(2).column(1).unwrap(), b"49745");
            assert_eq!(csv.row(2).column(2).unwrap(), b"35003");
            assert_eq!(csv.row(2).column(3).unwrap(), b"49.9727");
            assert_eq!(csv.row(2).column(4).unwrap(), b"9.107453");
            assert_eq!(csv.row(2).column(5).unwrap(), b"");
            assert_eq!(csv.row(3).len(), 0);
            assert_eq!(csv.row(4).len(), 2);
            assert_eq!(csv.row(4).column(0).unwrap(), b"");
            assert_eq!(csv.row(4).column(1).unwrap(), b"");
            assert_eq!(csv.row(5).len(), 2);
            assert_eq!(csv.row(5).column(0).unwrap(), b"1");
            assert_eq!(csv.row(5).column(1).unwrap(), b"2");
        }
    }
}
