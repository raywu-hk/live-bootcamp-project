use darling::ToTokens;
use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn clean_up(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(item as ItemFn);
    let cleanup_statement = parse_quote! {
        app.clean_up().await;
    };
    item_fn.block.stmts.push(cleanup_statement);
    item_fn.to_token_stream().into()
}
