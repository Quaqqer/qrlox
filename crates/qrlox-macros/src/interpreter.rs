use quote::{format_ident, quote};
use syn::{PatIdent, TypeReference};

enum Parameter {
    Normal(syn::Type),
    Span,
    Interpreter,
}

fn parse_param(arg: &syn::FnArg) -> Parameter {
    match arg {
        syn::FnArg::Typed(syn::PatType { ty, pat, .. }) => match pat.as_ref() {
            syn::Pat::Ident(PatIdent { ident, .. }) if ident == "span" => match ty.as_ref() {
                syn::Type::Reference(TypeReference { .. }) => Parameter::Span,
                _ => panic!("Must take span as a refernce"),
            },
            syn::Pat::Ident(PatIdent { ident, .. }) if ident == "interpreter" => {
                match ty.as_ref() {
                    syn::Type::Reference(TypeReference { .. }) => Parameter::Interpreter,
                    _ => panic!("Must take interpreter as a reference"),
                }
            }
            _ => Parameter::Normal(ty.as_ref().clone()),
        },
        syn::FnArg::Receiver(_) => panic!("`self` is not allowed for native functions"),
    }
}

pub fn expand_native_function(fn_: syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // Get name of native function
    let name = fn_.sig.ident.clone();
    let name_s = name.to_string();

    // Get visibility of rust item
    let visibility = fn_.vis.clone();

    // Get the parameters
    let params = fn_.sig.inputs.iter().map(parse_param).collect::<Vec<_>>();

    let mut stmts = Vec::<proc_macro2::TokenStream>::new();

    let mut normal_params = 0usize;

    for (i, param) in params.iter().enumerate().rev() {
        let ident = format_ident!("arg{}", i);

        let expr = match param {
            Parameter::Normal(_) => {
                let tokens = quote!(args.pop().unwrap());
                normal_params += 1;
                tokens
            }
            Parameter::Span => {
                quote!(span)
            }
            Parameter::Interpreter => {
                quote!(interpreter)
            }
        };

        stmts.push(quote!(let #ident = #expr;));
    }

    for (i, param) in params.iter().enumerate() {
        let Parameter::Normal(ty) = param else {
            continue;
        };

        let ident = format_ident!("arg{}", i);
        stmts.push(quote!(let #ident = native_cast::<#ty>(#ident, #i, span)?));
    }

    let args = params
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("arg{}", i))
        .collect::<Vec<_>>();
    let call = quote!(Into::<InterpreterResult>::into(#name(#(#args),*)).0);

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        #visibility fn #name() -> Native {
            #fn_

            Native {
                name: #name_s.to_string(),
                f: Box::new(
                    |interpreter: &mut InterpreterCtx, span: &Span, mut args: Vec<Value>| {
                        if args.len() != #normal_params {
                            return Err(err!(span, "Function expected {} arguments but got {}", #normal_params, args.len()));
                        }

                        #(#stmts;)*

                        #call
                    }
                )
            }
        }
    })
}
