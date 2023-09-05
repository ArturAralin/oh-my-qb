use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields};

#[proc_macro_derive(Row)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            let builder_fileds = fields.named.iter().map(|field| {
                let name = &field.ident;
                quote!(builder.append_binding(self.#name.value()))
            });

            let names = fields.named.iter().map(|field| {
                let name = &field.ident;
                quote!(stringify!(#name))
            });

            let name = input.ident;

            return TokenStream::from(quote!(
            impl<'a> crate::query_builder::Row<'a> for #name {
                fn columns() -> &'static [&'static str] {
                    &[
                        #(#names),*
                    ]
                }

                fn into_row(self, builder: &mut crate::query_builder::RowBuilder<'a>) {
                    #(#builder_fileds);*
                }
            }));
        }
    }

    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields can derive `Row`",
        )
        .to_compile_error(),
    )
}
