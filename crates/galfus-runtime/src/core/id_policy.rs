pub const CORE_RESERVED_LOGICAL_ID_COUNT: u32 = 0xffff;
pub const CORE_RESERVED_LOGICAL_ID_START: u32 = u32::MAX - CORE_RESERVED_LOGICAL_ID_COUNT + 1;
pub const CORE_RESERVED_LOGICAL_ID_START_U64: u64 =
    u64::MAX - CORE_RESERVED_LOGICAL_ID_COUNT as u64 + 1;

#[inline]
pub const fn is_core_reserved_logical_id(id: u32) -> bool {
    id >= CORE_RESERVED_LOGICAL_ID_START
}

#[inline]
pub fn validate_host_logical_id(id: u32, field_name: &str) -> Result<(), String> {
    if is_core_reserved_logical_id(id) {
        return Err(format!(
            "Logical id '{}' ({}) is reserved for core runtime. Host IDs must be lower than {}",
            field_name, id, CORE_RESERVED_LOGICAL_ID_START
        ));
    }
    Ok(())
}

#[inline]
pub fn validate_host_logical_id_u64(id: u64, field_name: &str) -> Result<(), String> {
    if id >= CORE_RESERVED_LOGICAL_ID_START_U64 {
        return Err(format!(
            "Logical id '{}' ({}) is reserved for core runtime. Host IDs must be lower than {}",
            field_name, id, CORE_RESERVED_LOGICAL_ID_START_U64
        ));
    }
    Ok(())
}
