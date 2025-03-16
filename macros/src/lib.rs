use proc_macro::TokenStream;
use quote::quote;
use syn::parse_quote;

/// A test harness that inserts the `#[test]` attribute and provides the `Client` through `crate::test_rocket(#fn_name)`.
#[proc_macro_attribute]
pub fn rocket_test(_args: TokenStream, item: TokenStream) -> TokenStream {
    // item should be a function
    let mut input = syn::parse_macro_input!(item as syn::ItemFn);

    let fn_name = input.sig.ident.to_string();

    let client_decl = parse_quote! {
        let client = ::rocket::local::blocking::Client::tracked(crate::test_rocket(#fn_name)).unwrap();
    };
    let test_attr = parse_quote!(#[test]);

    input.attrs.push(test_attr);
    input
        .block
        .stmts
        .insert(0, client_decl);

    let output = quote! {
        #input
    };

    output.into()
}

// TODO: see this
// #[proc_macro_attribute]
// pub fn blanket_impl(args: TokenStream, item: TokenStream) -> TokenStream {
//     let args = syn::parse_macro_input!(args as syn::)
//     let mut input = syn::parse_macro_input!(item as syn::ItemMod);

//     let content = input.content.unwrap();


//     let the_trait = content.1.iter().filter_map(|i| {
//         let syn::Item::Trait(t) = i else {
//             return None
//         };
//         if t.ident.to_string().to_lowercase() ==  {
//             Some(t)
//         } else {
//             None
//         }
//     });

//     todo!()
// }
