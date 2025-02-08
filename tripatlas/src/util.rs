use byte_unit::Byte;

pub fn bytes_to_human_string(bytes: Byte) -> String {
    let bytes_rounded = (bytes.as_u64() / 1000) * 1000;
    byte_unit::Byte::from_u64(bytes_rounded)
        .get_appropriate_unit(byte_unit::UnitType::Decimal)
        .to_string()
}
