use proc_macro::TokenStream;
use quote::quote;
use std::fmt::Display;
use std::str::FromStr;
use syn::spanned::Spanned;
use syn::{AttributeArgs, ItemFn, Lit, MetaNameValue};

fn parse_lit_num<N>(name_value: &MetaNameValue) -> syn::Result<N>
    where
        N: FromStr,
        N::Err: Display,
{
    let name = name_value.path.get_ident().unwrap().to_string();
    match &name_value.lit {
        Lit::Int(lit_int) => match lit_int.base10_parse::<N>() {
            Ok(val) => Ok(val),
            Err(e) => Err(syn::Error::new(
                name_value.span(),
                format!("Failed to parse value of {} as number: {}", name, e),
            )),
        },
        _ => Err(syn::Error::new(
            name_value.span(),
            format!("Failed to parse value of {} as number.", name),
        )),
    }
}

fn test_impl(mut input: ItemFn, args: AttributeArgs) -> Result<TokenStream, syn::Error> {
    if input.sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(input.sig.fn_token, ""));
    }

    let mut config_lines = quote! {
        let mut test_config = ::chazi::TestConfig::default();
    };
    let mut header = quote! {
        #[::core::prelude::v1::test]
    };
    for arg in args {
        match arg {
            syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                let ident = name_value.path.get_ident();
                if ident.is_none() {
                    let msg = "Must have specified ident";
                    return Err(syn::Error::new_spanned(name_value, msg));
                }
                match ident.unwrap().to_string().to_lowercase().as_str() {
                    "timeout_ms" => {
                        let timeout_ms = parse_lit_num::<u64>(&name_value)?;
                        config_lines = quote! {
                            #config_lines
                            test_config.timeout = ::std::time::Duration::from_millis(#timeout_ms);
                        };
                    }
                    "exit_code" => {
                        let exit_code = parse_lit_num::<i32>(&name_value)?;
                        config_lines = quote! {
                            #config_lines
                            test_config.expected_result = ::chazi::TestResult::ExitCode(#exit_code);
                        };
                    },
                    name => {
                        let msg = format!(
                            "Unknown attribute {}",
                            name,
                        );
                        return Err(syn::Error::new_spanned(name_value, msg));
                    }
                }
            }
            syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                let ident = path.get_ident();
                if ident.is_none() {
                    let msg = "ident of arg is missing";
                    return Err(syn::Error::new_spanned(path, msg));
                }
                let name = ident.unwrap().to_string().to_lowercase();
                match name.as_str() {
                    "ignore" => {
                        config_lines = quote! {
                            #config_lines
                            test_config.ignore = true;
                        };
                        header = quote! {
                            #header
                            #[ignore]
                        };
                    }
                    "check_reach" => {
                        config_lines = quote! {
                            #config_lines
                            test_config.check_reach = true;
                        };
                    }
                    "should_panic" => {
                        config_lines = quote! {
                            #config_lines
                            test_config.expected_result = ::chazi::TestResult::Panic;
                        };
                    }
                    "parent_should_panic" => {
                        config_lines = quote! {
                            #config_lines
                            test_config.parent_should_panic = true;
                        };
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            path,
                            format!("Unknown attribute for chazi::test"),
                        ));
                    }
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    format!("Unknown attribute for chazi::test"),
                ));
            }
        }
    }

    let test_fn_name = input.sig.ident.to_string();

    let body = input.block;
    input.block = syn::parse_quote! {
        {
            fn test_impl() {
                #body
            }
            #config_lines
            ::chazi::fork_in_test(::std::module_path!(), #test_fn_name, test_impl, test_config)
        }
    };

    let result = quote! {
        #header
        #input
    };
    Ok(result.into())
}

#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    test_impl(input, args).unwrap_or_else(|e| e.to_compile_error().into())
}
