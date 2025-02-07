/// A (part) of a CSV file parsed into records and their fields.
/// One can iterate over the individual records and access the fields as `&[u8]` slices.
///
/// 'a is the lifetime of the parsed buffer. The fields still reference the original buffer
/// to avoid unnecessary copies.
#[derive(Default)]
pub struct CsvRecords<'a> {
    /// An offsets array where consecutive values specify a range of fields in the `fields` array.
    /// The first value is 0 and the last value is the length of the `fields` array.
    /// This is one longer than the number of records.
    offsets: Vec<usize>,
    /// The fields of each record in a flat vector. This is cheaper than having a vector of vectors.
    fields: Vec<&'a [u8]>,
}

impl<'a> CsvRecords<'a> {
    /// Splits the given buffer into records and their fields.
    pub fn from_buffer(buffer: &'a [u8]) -> Self {
        let mut offsets = vec![];
        let mut fields = vec![];

        offsets.push(0);
        let mut start = 0;
        while start < buffer.len() {
            start = crate::parse_record::parse_record_fields(buffer, start, &mut fields);
            offsets.push(fields.len());
        }
        CsvRecords { offsets, fields }
    }

    /// Get an iterator over all the records.
    pub fn iter<'r>(&'r self) -> CsvRecordsIter<'r, 'a> {
        CsvRecordsIter {
            records: self,
            i: 0,
        }
    }

    /// Get the number of records.
    pub fn len(&self) -> usize {
        self.offsets.len() - 1
    }

    /// Get a specific record by its index. This function panics if the index is out of bounds.
    pub fn record<'r>(&'r self, i: usize) -> CsvRecord<'r, 'a> {
        let start = self.offsets[i];
        let end = self.offsets[i + 1];
        CsvRecord {
            fields: &self.fields[start..end],
        }
    }
}

/// Contains the fields of an individual record (i.e. a line/row).
pub struct CsvRecord<'r, 'a> {
    pub fields: &'r [&'a [u8]],
}

impl<'r, 'a> CsvRecord<'r, 'a> {
    /// Get the number of fields in this record.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Get the value for a specific column or None if the index is out of bounds.
    pub fn column(&self, column_i: usize) -> Option<&'a [u8]> {
        self.fields.get(column_i).copied()
    }
}

/// An iterator over the records of a CSV file buffer.
pub struct CsvRecordsIter<'r, 'a> {
    records: &'r CsvRecords<'a>,
    i: usize,
}

impl<'r, 'a> Iterator for CsvRecordsIter<'r, 'a> {
    type Item = CsvRecord<'r, 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i + 1 >= self.records.offsets.len() {
            return None;
        }
        let record = self.records.record(self.i);
        self.i += 1;
        Some(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

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
}
