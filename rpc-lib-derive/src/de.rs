use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, Ident};

pub fn expand_derive_de(input: DeriveInput) -> TokenStream {
    let struct_ident = input.ident;
    match input.data {
        Data::Struct(data_struct) => expand_struct(struct_ident, data_struct),
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}

pub fn expand_struct(struct_ident: Ident, data_struct: DataStruct) -> TokenStream {
    let fields_named = match data_struct.fields {
        Fields::Named(fields_named) => fields_named,
        Fields::Unnamed(_) | Fields::Unit => unreachable!(),
    };

    let deserializations = fields_named
        .named
        .iter()
        .map(|field| {
            let ident = &field.ident;
            quote! {
                #ident: XdrDeserialize::deserialize(&mut reader)?,
            }
        })
        .collect::<TokenStream>();

    quote! {
        impl XdrDeserialize for #struct_ident {
            fn deserialize(mut reader: impl ::std::io::Read) -> ::std::io::Result<Self> {
                Ok(Self {
                    #deserializations
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_xdr() {
        let input = parse_quote! {
            struct Foo {
                bar: u32,
                baz: u32,
            }
        };

        let output = quote! {
            impl XdrDeserialize for Foo {
                fn deserialize(mut reader: impl ::std::io::Read) -> ::std::io::Result<Self> {
                    Ok(Self {
                        bar: XdrDeserialize::deserialize(&mut reader)?,
                        baz: XdrDeserialize::deserialize(&mut reader)?,
                    })
                }
            }
        };

        assert_eq!(output.to_string(), expand_derive_de(input).to_string());
    }
}
