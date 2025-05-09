use darling::ToTokens;
use proc_macro::TokenStream;
use syn::__private::quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn api_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &item_fn.sig.ident;
    let fn_inputs = &item_fn.sig.inputs;
    let fn_output = &item_fn.sig.output;
    let fn_body = &item_fn.block;

    let expended = quote! {
        #[tokio::test]
        async fn #fn_name(#fn_inputs) #fn_output {
            let mut app = TestApp::new().await;
            #fn_body
            app.clean_up().await;
        }
    };
    expended.into()
}
