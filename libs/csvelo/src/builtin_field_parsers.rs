use crate::ParseCsvField;

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

impl<'a> ParseCsvField<'a> for bool {
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
