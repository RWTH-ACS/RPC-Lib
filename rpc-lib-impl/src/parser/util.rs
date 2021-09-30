pub enum TypeKind {
    SimpleIdent,
    FixedArray,
    VarlenArray,
}

pub fn convert_primitve_type(s: &str) -> Option<&str> {
    match s {
        // Integer
        "int8_t" | "char" | "signed char" => Some("i8"),
        "uint8_t" | "unsigned char" => Some("u8"),
        "int16_t" | "short" | "signed short" => Some("i16"),
        "uint16_t" | "unsigned short" => Some("u16"),
        "int32_t" | "int" | "signed int" => Some("i32"),
        "uint32_t" | "unsigned int" => Some("u32"),
        "int64_t" | "long" | "signed long" => Some("i64"),
        "uint64_t" | "unsigned long" => Some("u64"),

        // Floating Point
        "float" => Some("f32"),
        "double" => Some("f64"),

        // Rpcl-strings
        "string" | "string<>" => Some("String"),

        // Opaque
        "opaque" => Some("u8"),

        // Invalid primitive Type
        _ => None,
    }
}
