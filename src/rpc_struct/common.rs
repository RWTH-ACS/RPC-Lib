use quote::__private::TokenStream as QuoteTokenStream;
use quote::{format_ident, quote};

pub fn generate_ctor_dtor(
    struct_name: &String,
    program_number: i32,
    version_number: i32,
) -> QuoteTokenStream {
    let name = format_ident!("{}", struct_name);

    // Required use-Statements & C-Functions
    let c_bindings = quote! {
        use std::os::raw::c_char;
        use std::ffi::c_void;
        use std::ffi::CString;

        extern "C" {
            fn clnt_create(host: *const c_char, program: i32, version: i32, protocol: *const c_char) -> *mut c_void;
            fn rpc_deinitialize(client: *mut c_void);
        }
    };

    // Struct Definition with CLIENT*
    let struct_def = quote! {
        struct #name {
            client: *mut c_void
        }
    };

    // Constructor
    let ctor = quote! {
        impl #name {
            pub fn new(host: &str) -> #name {
                let host_c_str = CString::new(host).unwrap();
                let host_char_ptr: *const c_char = host_c_str.as_ptr() as *const c_char;
                let protocol_c_str = CString::new("tcp").unwrap();
                let protocol_c_ptr: *const c_char = protocol_c_str.as_ptr() as *const c_char;
                unsafe {
                    let x: *mut c_void = clnt_create(host_char_ptr, #program_number, #version_number, protocol_c_ptr);
                    match x.is_null() {
                        true => panic!("Error Initializing RPC"),
                        false => println!("Initialized Connection")
                    };
                    #name {client: x}
                }
            }
        }
    };

    // Destructor (Implementation for Drop-Trait)
    let dtor = quote! {
        impl Drop for #name {
            fn drop(&mut self) {
                //Call to C
                unsafe {
                    rpc_deinitialize(self.client);
                }
                println!("Closed Connection");
            }
        }
    };

    // Pasting everything together
    let code = quote! {
        #c_bindings
        #struct_def
        #ctor
        #dtor
    };
    code
}
