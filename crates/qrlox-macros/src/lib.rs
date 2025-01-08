mod interpreter;

use interpreter::expand_native_function;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn interpreter_native(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_native_function(syn::parse_macro_input!(item as syn::ItemFn))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
