pub fn bytes_to_human_string(bytes: u64) -> String {
    let bytes_rounded = (bytes / 1000) * 1000;
    byte_unit::Byte::from_u64(bytes_rounded)
        .get_appropriate_unit(byte_unit::UnitType::Decimal)
        .to_string()
}
