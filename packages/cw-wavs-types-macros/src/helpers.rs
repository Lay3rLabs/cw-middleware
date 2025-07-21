use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput};

pub fn merge_variants(metadata: TokenStream, left: TokenStream, right: TokenStream) -> TokenStream {
    use syn::Data::Enum;

    // Ensure no arguments are provided to the macro that calls this function
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "macro takes no arguments")
            .to_compile_error()
            .into();
    }

    // Parse both enums

    let mut left: DeriveInput = parse_macro_input!(left);
    let Enum(DataEnum { variants, .. }) = &mut left.data else {
        return syn::Error::new(left.ident.span(), "macro must be used on an enum")
            .to_compile_error()
            .into();
    };

    let right: DeriveInput = parse_macro_input!(right);
    let Enum(DataEnum {
        variants: to_add, ..
    }) = right.data
    else {
        return syn::Error::new(left.ident.span(), "last argument must be an enum")
            .to_compile_error()
            .into();
    };

    // Insert variants from the right to the left
    variants.extend(to_add);

    quote! { #left }.into()
}
