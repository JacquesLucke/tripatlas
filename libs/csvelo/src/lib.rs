pub use csvelo_derive::CSVParser;
pub use flatten::flatten_slices;
pub use records::CsvRecords;

mod builtin_field_parsers;
mod flatten;
mod parse_record;
mod records;

use parse_record::*;

pub struct CsvBufferSections<'buf> {
    pub header: &'buf [u8],
    pub data: &'buf [u8],
}

pub struct CsvHeader<'buf> {
    pub column_titles: Vec<&'buf [u8]>,
}

pub trait ParseCsvField<'buf>: Sized {
    fn parse_csv_field(buffer: &'buf [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'buf;
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

pub fn parse_header_record_str(
    header: &[u8],
) -> std::result::Result<Vec<&str>, std::str::Utf8Error> {
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

pub fn parse_column_value<'buf, T>(
    records: &CsvRecords<'buf>,
    column_i: usize,
    parse_field: impl Fn(&'buf [u8]) -> std::result::Result<T, ()>,
) -> std::result::Result<Vec<T>, ()> {
    let mut data = vec![];
    data.reserve(records.len());
    for record in records.iter() {
        let column_buffer = record.column(column_i).unwrap_or(b"");
        data.push(parse_field(column_buffer)?);
    }
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

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
