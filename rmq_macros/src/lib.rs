use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(RMQDeserializer)]
pub fn rmq_deserializer_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let generated = quote! {
        impl RMQDeserializer for #name {}
    };

    generated.into()
}
