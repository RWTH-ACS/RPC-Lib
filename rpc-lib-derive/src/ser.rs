use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, Generics, Ident};

pub fn expand_derive_ser(input: DeriveInput) -> TokenStream {
    let struct_ident = input.ident;
    match input.data {
        Data::Struct(data_struct) => expand_struct(struct_ident, input.generics, data_struct),
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}

pub fn expand_struct(
    struct_ident: Ident,
    generics: Generics,
    data_struct: DataStruct,
) -> TokenStream {
    let fields_named = match data_struct.fields {
        Fields::Named(fields_named) => fields_named,
        Fields::Unnamed(_) | Fields::Unit => unreachable!(),
    };

    let lengths = fields_named
        .named
        .iter()
        .map(|field| {
            let field_ident = &field.ident;
            quote! {
                XdrSerialize::len(&self.#field_ident) +
            }
        })
        .collect::<TokenStream>();

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
        impl #generics XdrSerialize for #struct_ident #generics {
            fn len(&self) -> usize {
                #lengths 0
            }

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
                fn len(&self) -> usize {
                    XdrSerialize::len(&self.bar) + XdrSerialize::len(&self.baz) + 0
                }

                fn serialize(&self, mut writer: impl ::std::io::Write) -> ::std::io::Result<()> {
                    self.bar.serialize(&mut writer)?;
                    self.baz.serialize(&mut writer)?;
                    Ok(())
                }
            }
        };

        assert_eq!(output.to_string(), expand_derive_ser(input).to_string());
    }

    #[test]
    fn test_generics() {
        let input = parse_quote! {
            struct Foo<'a> {
                bar: &'a u32,
                baz: &'a u32,
            }
        };

        let output = quote! {
            impl<'a> XdrSerialize for Foo<'a> {
                fn len(&self) -> usize {
                    XdrSerialize::len(&self.bar) + XdrSerialize::len(&self.baz) + 0
                }

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
