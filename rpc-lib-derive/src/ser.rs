use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, Ident};

pub fn expand_derive_ser(input: DeriveInput) -> TokenStream {
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

    let serializations = fields_named
        .named
        .iter()
        .map(|field| {
            let field_ident = &field.ident;
            quote! {
                self.#field_ident.serialize(&mut writer)?;
            }
        })
        .collect::<TokenStream>();

    quote! {
        impl XdrSerialize for #struct_ident {
            fn serialize(&self, mut writer: impl ::std::io::Write) -> ::std::io::Result<()> {
                #serializations
                Ok(())
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
            impl XdrSerialize for Foo {
                fn serialize(&self, mut writer: impl ::std::io::Write) -> ::std::io::Result<()> {
                    self.bar.serialize(&mut writer)?;
                    self.baz.serialize(&mut writer)?;
                    Ok(())
                }
            }
        };

        assert_eq!(output.to_string(), expand_derive_ser(input).to_string());
    }
}
