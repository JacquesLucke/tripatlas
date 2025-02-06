pub use csvelo_derive::CSVParser;
use std::str::Utf8Error;

pub fn flatten_slices<T: Clone>(slices: &[&[T]]) -> Vec<T> {
    let total_len: usize = slices.iter().map(|s| s.len()).sum();
    let mut flattened = Vec::with_capacity(total_len);
    for v in slices {
        flattened.extend_from_slice(v);
    }
    flattened
}

#[derive(Default)]
pub struct CsvRecords<'b> {
    offsets: Vec<usize>,
    fields: Vec<&'b [u8]>,
}

pub struct CsvRecordsIter<'r, 'b> {
    records: &'r CsvRecords<'b>,
    i: usize,
}

pub struct CsvBufferSections<'a> {
    pub header: &'a [u8],
    pub data: &'a [u8],
}

pub struct CsvHeader<'a> {
    pub column_titles: Vec<&'a [u8]>,
}

pub trait ParseCsvField<'a>: Sized {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a;
}

impl<'a> ParseCsvField<'a> for &'a str {
    fn parse_csv_field<'b>(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        std::str::from_utf8(buffer).map_err(|_| ())
    }
}

impl<'a> ParseCsvField<'a> for i32 {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        std::str::from_utf8(buffer)
            .map_err(|_| ())
            .and_then(|s| s.parse().map_err(|_| ()))
    }
}

impl<'a> ParseCsvField<'a> for u32 {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        std::str::from_utf8(buffer)
            .map_err(|_| ())
            .and_then(|s| s.parse().map_err(|_| ()))
    }
}

impl<'a> ParseCsvField<'a> for String {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        std::str::from_utf8(buffer)
            .map_err(|_| ())
            .and_then(|s| s.parse().map_err(|_| ()))
    }
}

impl CsvHeader<'_> {
    pub fn get_column_index(&self, column_name: &str) -> Option<usize> {
        self.column_titles
            .iter()
            .position(|c| c == &column_name.as_bytes())
    }
}

pub fn parse_header(header: &[u8]) -> CsvHeader {
    let mut fields = vec![];
    parse_record_fields(header, 0, &mut fields);
    CsvHeader {
        column_titles: fields,
    }
}

pub fn parse_header_record_str(header: &[u8]) -> std::result::Result<Vec<&str>, Utf8Error> {
    let mut fields = vec![];
    parse_record_fields(header, 0, &mut fields);
    fields.iter().map(|f| std::str::from_utf8(f)).collect()
}

pub fn split_header_and_data(buffer: &[u8]) -> CsvBufferSections {
    let data_start_i = find_start_of_next_record(buffer, 0);
    CsvBufferSections {
        header: &buffer[..data_start_i],
        data: &buffer[data_start_i..],
    }
}

pub fn split_csv_buffer_into_record_aligned_chunks(
    buffer: &[u8],
    approximate_chunk_size: usize,
) -> Vec<&[u8]> {
    let mut chunks = vec![];
    let mut next_chunk_start = 0;
    while next_chunk_start < buffer.len() {
        let approximate_chunk_end = (next_chunk_start + approximate_chunk_size).min(buffer.len());
        let chunk_end = find_start_of_next_record(buffer, approximate_chunk_end);
        chunks.push(&buffer[next_chunk_start..chunk_end]);
        next_chunk_start = chunk_end;
    }
    chunks
}

impl<'r, 'b> Iterator for CsvRecordsIter<'r, 'b> {
    type Item = CsvRecord<'r, 'b>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i + 1 >= self.records.offsets.len() {
            return None;
        }
        let record = self.records.record(self.i);
        self.i += 1;
        Some(record)
    }
}

impl<'b> CsvRecords<'b> {
    pub fn from_buffer(buffer: &'b [u8]) -> Self {
        let mut offsets = vec![];
        let mut fields = vec![];

        offsets.push(0);
        let mut start = 0;
        while start < buffer.len() {
            start = parse_record_fields(buffer, start, &mut fields);
            offsets.push(fields.len());
        }
        CsvRecords { offsets, fields }
    }

    pub fn iter<'r>(&'r self) -> CsvRecordsIter<'r, 'b> {
        CsvRecordsIter {
            records: self,
            i: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.offsets.len() - 1
    }

    pub fn record<'r>(&'r self, i: usize) -> CsvRecord<'r, 'b> {
        let start = self.offsets[i];
        let end = self.offsets[i + 1];
        CsvRecord {
            fields: &self.fields[start..end],
        }
    }
}

pub struct CsvRecord<'r, 'b> {
    pub fields: &'r [&'b [u8]],
}

impl<'r, 'b> CsvRecord<'r, 'b> {
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn column(&self, column_i: usize) -> Option<&'b [u8]> {
        self.fields.get(column_i).copied()
    }
}

pub fn parse_column_value<'a, T>(
    records: &CsvRecords<'a>,
    column_i: usize,
    parse_field: impl Fn(&'a [u8]) -> std::result::Result<T, ()>,
) -> std::result::Result<Vec<T>, ()> {
    let mut data = vec![];
    data.reserve(records.len());
    for record in records.iter() {
        let column_buffer = record.column(column_i).unwrap_or(b"");
        data.push(parse_field(column_buffer)?);
    }
    Ok(data)
}

/// Adds all the fields of the current record and returns the first index after the record.
/// I.e. the index after the newline character or the end of the buffer.
/// The start index has to be the index of the first character in the record.
fn parse_record_fields<'a>(buffer: &'a [u8], start: usize, fields: &mut Vec<&'a [u8]>) -> usize {
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
fn find_start_of_next_record(buffer: &[u8], start: usize) -> usize {
    if let Some(newline_offset) = buffer[start..].iter().position(|&c| c == b'\n') {
        start + newline_offset + 1
    } else {
        buffer.len()
    }
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

    fn get_parsed_record(buffer: &str) -> Vec<&str> {
        let mut fields = vec![];
        parse_record_fields(buffer.as_bytes(), 0, &mut fields);
        fields
            .iter()
            .map(|f| std::str::from_utf8(f).unwrap())
            .collect()
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

    #[test]
    fn test_parse_csv_buffer() {
        {
            let buffer = indoc! {"
                123,456,789
                1,2,3
            "}
            .as_bytes();
            let records = CsvRecords::from_buffer(buffer);
            assert_eq!(records.len(), 2);
            assert_eq!(records.record(0).len(), 3);
            assert_eq!(records.record(0).column(0).unwrap(), b"123");
            assert_eq!(records.record(0).column(1).unwrap(), b"456");
            assert_eq!(records.record(0).column(2).unwrap(), b"789");
            assert_eq!(records.record(1).len(), 3);
            assert_eq!(records.record(1).column(0).unwrap(), b"1");
            assert_eq!(records.record(1).column(1).unwrap(), b"2");
            assert_eq!(records.record(1).column(2).unwrap(), b"3");
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
            let csv = CsvRecords::from_buffer(buffer);
            assert_eq!(csv.len(), 6);
            assert_eq!(csv.record(0).len(), 6);
            assert_eq!(csv.record(0).column(0).unwrap(), b"stop_name");
            assert_eq!(csv.record(0).column(1).unwrap(), b"parent_station");
            assert_eq!(csv.record(0).column(2).unwrap(), b"stop_id");
            assert_eq!(csv.record(0).column(3).unwrap(), b"stop_lat");
            assert_eq!(csv.record(0).column(4).unwrap(), b"stop_lon");
            assert_eq!(csv.record(0).column(5).unwrap(), b"location_type");
            assert_eq!(csv.record(1).len(), 6);
            assert_eq!(
                csv.record(1).column(0).unwrap(),
                b"'s-Heerenberg Gouden Handen"
            );
            assert_eq!(csv.record(1).column(1).unwrap(), b"");
            assert_eq!(csv.record(1).column(2).unwrap(), b"237383");
            assert_eq!(csv.record(1).column(3).unwrap(), b"51.87225");
            assert_eq!(csv.record(1).column(4).unwrap(), b"6.2473383");
            assert_eq!(csv.record(1).column(5).unwrap(), b"1");
            assert_eq!(csv.record(2).len(), 6);
            assert_eq!(csv.record(2).column(0).unwrap(), b"AB-Leider, Hafen");
            assert_eq!(csv.record(2).column(1).unwrap(), b"49745");
            assert_eq!(csv.record(2).column(2).unwrap(), b"35003");
            assert_eq!(csv.record(2).column(3).unwrap(), b"49.9727");
            assert_eq!(csv.record(2).column(4).unwrap(), b"9.107453");
            assert_eq!(csv.record(2).column(5).unwrap(), b"");
            assert_eq!(csv.record(3).len(), 0);
            assert_eq!(csv.record(4).len(), 2);
            assert_eq!(csv.record(4).column(0).unwrap(), b"");
            assert_eq!(csv.record(4).column(1).unwrap(), b"");
            assert_eq!(csv.record(5).len(), 2);
            assert_eq!(csv.record(5).column(0).unwrap(), b"1");
            assert_eq!(csv.record(5).column(1).unwrap(), b"2");
        }
    }

    #[test]
    fn test_records_iterator() {
        let buffer = indoc! {"
                1
                3
                5
            "}
        .as_bytes();
        let records = CsvRecords::from_buffer(buffer);
        let mut iter = records.iter();
        assert_eq!(iter.next().unwrap().column(0).unwrap(), b"1");
        assert_eq!(iter.next().unwrap().column(0).unwrap(), b"3");
        assert_eq!(iter.next().unwrap().column(0).unwrap(), b"5");
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_split_csv_buffer_into_line_aligned_chunks() {
        let buffer = indoc! {r#"
            0,1,2,3
            ,,,
            4,5,6,7
        "#};
        {
            let chunks = split_csv_buffer_into_record_aligned_chunks(buffer.as_bytes(), 0);
            assert_eq!(chunks.len(), 3);
            assert_eq!(chunks[0], b"0,1,2,3\n");
            assert_eq!(chunks[1], b",,,\n");
            assert_eq!(chunks[2], b"4,5,6,7\n");
        }
        {
            let chunks = split_csv_buffer_into_record_aligned_chunks(buffer.as_bytes(), 11);
            assert_eq!(chunks.len(), 2);
            assert_eq!(chunks[0], b"0,1,2,3\n,,,\n");
            assert_eq!(chunks[1], b"4,5,6,7\n");
        }
        {
            let chunks = split_csv_buffer_into_record_aligned_chunks(buffer.as_bytes(), 1000);
            assert_eq!(chunks.len(), 1);
            assert_eq!(chunks[0], b"0,1,2,3\n,,,\n4,5,6,7\n");
        }
    }

    #[test]
    fn test_split_header_and_data() {
        let buffer = indoc! {r#"
            Title,Author,Year
            1,2,3
            4,5,6
        "#};
        let sections = split_header_and_data(buffer.as_bytes());
        assert_eq!(sections.header, b"Title,Author,Year\n");
        assert_eq!(sections.data, b"1,2,3\n4,5,6\n");

        let headers = parse_header_record_str(sections.header).unwrap();
        assert_eq!(headers, &["Title", "Author", "Year"]);
    }
}
