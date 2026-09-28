use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn raise_handler(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);

    if function.sig.ident != "main" {
        return syn::Error::new_spanned(
            function.sig.ident,
            "raise_handler can only be used on main",
        )
        .to_compile_error()
        .into();
    }

    if !function.sig.inputs.is_empty()
        || function.sig.asyncness.is_some()
        || function.sig.constness.is_some()
        || function.sig.unsafety.is_some()
        || function.sig.abi.is_some()
        || !function.sig.generics.params.is_empty()
    {
        return syn::Error::new_spanned(
            function.sig,
            "raise_handler requires a synchronous `fn main()` with no parameters",
        )
        .to_compile_error()
        .into();
    }

    let visibility = function.vis.clone();
    let output = function.sig.output.clone();
    function.sig.ident = syn::Ident::new("__rust_utils_main_inner", function.sig.ident.span());

    quote! {
        #function

        #visibility fn main() #output {
            ::rust_utils::raise::suppress_raised_hook();

            match ::rust_utils::raise::catch_raised(|| __rust_utils_main_inner()) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("{error}");
                    ::std::process::exit(1);
                }
            }
        }
    }
    .into()
}
