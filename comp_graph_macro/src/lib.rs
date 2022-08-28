extern crate proc_macro;
extern crate comp_graph;
use proc_macro::TokenStream;

#[proc_macro_derive(OutputStructMacro)]
pub fn derive_answer_fn(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}