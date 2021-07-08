use quote::__private::TokenStream as QuoteTokenStream;
use quote::{format_ident, quote};

pub enum TypeKind {
    SimpleIdent,
    FixedArray,
    VarlenArray,
}

// Example:
// int<> ->
// struct i32_var_arr {
//    len: u32,
//    data: *mut i32
// }
pub fn generate_varlen_struct_for_type(type_name: &str) -> QuoteTokenStream {
    let data_type = convert_primitve_type(type_name).unwrap_or_else(|| type_name);
    let ident = format_ident!("{}_var_arr", data_type);
    let data_type_formatted = format_ident!("{}", data_type);
    quote! {
        #[repr(C)]
        struct #ident {
            len: u32,
            data: *mut #data_type_formatted,
        }
    }
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
        "string" | "string<>" => Some("*mut c_char"),

        // Opaque
        "opaque" => Some("c_void"),

        // Invalid primitive Type
        _ => None,
    }
}

// Code for converting &str (Rust) to char* (C)
pub fn prepare_char_ptr(identifier: &str) -> QuoteTokenStream {
    let param_fmt = format_ident!("{}", identifier);
    let temp_ident1 = format_ident!("{}_cstr", identifier);
    let temp_ident2 = format_ident!("{}_cptr", identifier);
    quote! {
        let #temp_ident1 = CString::new(#param_fmt).unwrap();
        let #temp_ident2 = #temp_ident1.as_ptr() as *const c_char;
    }
}
