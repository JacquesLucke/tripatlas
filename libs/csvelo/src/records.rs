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
