extern crate comp_graph;
extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(OutputStructMacro)]
pub fn output_struct_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let name = input.ident;

    let attrs = output_attrs(&input.data);

    let output = quote! {
        unsafe impl ::comp_graph::compute_graph::OutputStruct for #name {
            fn declare_outputs<'a>(&'a self, outputs: &mut ::comp_graph::compute_graph::OutputAttributes<'a>) {
                #attrs
            }
        }
    };
    output.into()
}

fn output_attrs(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let s = f.ident.as_ref().unwrap().to_string();
                    quote_spanned! {f.span()=>
                        outputs.add(&#s, &self.#name);
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

#[proc_macro_derive(InputStructMacro)]
pub fn input_struct_impl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let name = input.ident;

    let attrs = input_attrs(&input.data);
    let new_attrs = input_new_attrs(&input.data);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let output = quote! {
        unsafe impl #impl_generics ::comp_graph::compute_graph::InputStruct for #name #ty_generics # where_clause {
            fn declare_inputs<'a>(&'a mut self, inputs: &mut ::comp_graph::compute_graph::InputAttributes<'a>) {
                #attrs
            }
            fn new(input_maker: InputMaker) -> Self {
                #name {
                    #new_attrs
                }
            }
        }
    };
    output.into()
}

fn input_new_attrs(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! {f.span()=>
                        #name: Input::new(input_maker),
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

fn input_attrs(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let s = f.ident.as_ref().unwrap().to_string();
                    quote_spanned! {f.span()=>
                        inputs.add(&#s, &mut self.#name);
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

