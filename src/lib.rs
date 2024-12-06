use std::env;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn template2struct(input: TokenStream) -> TokenStream {
    let args: Vec<syn::Lit> = parse_macro_input!(input with syn::punctuated::Punctuated::<syn::Lit, syn::token::Comma>::parse_terminated).into_iter().collect();

    if args.len() != 2 {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "Expected two arguments: struct name and file path",
        )
        .to_compile_error()
        .into();
    }

    let struct_name = if let syn::Lit::Str(lit) = &args[0] {
        lit.value()
    } else {
        return syn::Error::new_spanned(&args[0], "Expected a string literal for the struct name")
            .to_compile_error()
            .into();
    };

    let file_path = if let syn::Lit::Str(lit) = &args[1] {
        lit.value()
    } else {
        return syn::Error::new_spanned(&args[1], "Expected a string literal for the file path")
            .to_compile_error()
            .into();
    };

    let macro_path = file_path.replace("/src", "");

    let struct_ident = proc_macro2::Ident::new(&struct_name, proc_macro2::Span::call_site());
    let fp = format!(
        "{}/src/{file_path}",
        env::current_dir().unwrap().to_string_lossy()
    );
    let file_contents = match std::fs::read_to_string(&fp) {
        Ok(content) => content,
        Err(reason) => {
            //let path = env::current_dir();
            println!("The current directory is {fp}");

            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to read file: {} {reason}", file_path),
            )
            .to_compile_error()
            .into();
        }
    };

    let rmgt = file_contents
        .replace("<", " ")
        .replace(">", " ")
        .replace("\"", " ");

    let fields: Vec<_> = rmgt
        .split_whitespace()
        .filter(|word| word.starts_with('$'))
        .map(|word| word.trim_start_matches('$'))
        .map(|word| proc_macro2::Ident::new(word, proc_macro2::Span::call_site()))
        .collect();

    let vars: Vec<_> = rmgt
        .split_whitespace()
        .filter(|word| word.starts_with('$'))
        .collect();

    let expanded = quote! {
        pub struct #struct_ident {
            #(
                pub #fields: String,
            )*
        }

        impl #struct_ident {

            pub fn new(#(
                #fields: &str,
            )*) -> Self {
                Self {
                    #(
                        #fields: #fields.to_owned(),
                    )*
                }
            }
            pub fn render(&self) -> String {
                const TEMPLATE: &'static str = include_str!(#macro_path);
                let mut result = TEMPLATE.to_owned();
                #(
                    result = result.replace(#vars, &self.#fields);
                )*
                result
            }
        }


    };

    println!("{}", expanded);

    TokenStream::from(expanded)
}
