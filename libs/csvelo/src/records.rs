#[derive(Default)]
pub struct CsvRecords<'b> {
    offsets: Vec<usize>,
    fields: Vec<&'b [u8]>,
}

impl<'b> CsvRecords<'b> {
    pub fn from_buffer(buffer: &'b [u8]) -> Self {
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

pub struct CsvRecordsIter<'r, 'b> {
    records: &'r CsvRecords<'b>,
    i: usize,
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
