use crate::ParseCsvField;

#[macro_export]
macro_rules! parse_primitive_type {
    ($ty:ty) => {
        impl<'buf> ParseCsvField<'buf> for $ty {
            fn parse_csv_field(buffer: &'buf [u8]) -> std::result::Result<Self, ()>
            where
                Self: 'buf,
            {
                std::str::from_utf8(buffer.trim_ascii())
                    .map_err(|_| ())
                    .and_then(|s| s.parse().map_err(|_| ()))
            }
        }
    };
}

// Intentionally not parsing bool here, because there are many ways to represent it.
parse_primitive_type!(f32);
parse_primitive_type!(f64);
parse_primitive_type!(i8);
parse_primitive_type!(i16);
parse_primitive_type!(i32);
parse_primitive_type!(u8);
parse_primitive_type!(u16);
parse_primitive_type!(u32);

impl<'buf> ParseCsvField<'buf> for &'buf str {
    fn parse_csv_field<'b>(buffer: &'buf [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'buf,
    {
        std::str::from_utf8(buffer).map_err(|_| ())
    }
}

impl<'buf> ParseCsvField<'buf> for &'buf [u8] {
    fn parse_csv_field(buffer: &'buf [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'buf,
    {
        Ok(buffer)
    }
}

impl<'buf> ParseCsvField<'buf> for String {
    fn parse_csv_field(buffer: &'buf [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'buf,
    {
        std::str::from_utf8(buffer)
            .map_err(|_| ())
            .and_then(|s| s.parse().map_err(|_| ()))
    }
}
