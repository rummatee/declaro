use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced, parse::{Parse, ParseStream}, parse_macro_input, Block, Expr, Ident, Pat, Token, Type
};

struct FocusableArm {
    matcher: Pat,
    focused_type: Type,
    focused_preparation: Option<proc_macro2::TokenStream>,
    focused_content: Option<proc_macro2::TokenStream>,
    blurred_type: Type,
    blurred_preparation: Option<proc_macro2::TokenStream>,
    blurred_content: Option<proc_macro2::TokenStream>,
}

struct FocusableInput {
    iterator: Expr,
    focus: Expr,
    arms: Vec<FocusableArm>,
}

impl Parse for FocusableInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let iterator: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let focus: Expr = input.parse()?;

        let mut arms = vec![];
        let mut focused_span = None;
        let mut blurred_span = None;
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let matcher: Pat = Pat::parse_multi(input)?;
            input.parse::<Token![,]>()?;
            let mut focused_preparation = None;
            let mut focused_content = None;
            let mut blurred_preparation = None;
            let mut blurred_content = None;
            let mut focused_type = None;
            let mut blurred_type = None;
            let ident: Ident = input.parse()?;
            if ident != "focused" {
                return Err(syn::Error::new(ident.span(), "Expected 'focused'"));
            }
            input.parse::<Token![:]>()?;
            let focused;
            braced!(focused in input);
            focused_span = Some(focused.span());
            while !focused.is_empty() {
                let ident: Ident = focused.parse()?;
                match ident.to_string().as_str() {
                    "t" => {
                        focused.parse::<Token![:]>()?;
                        let t: Type = focused.parse()?;
                        focused_type = Some(t);
                    }
                    "preparation" => {
                        focused.parse::<Token![:]>()?;
                        let preparation_block;
                        braced!(preparation_block in focused);
                        focused_preparation = Some(preparation_block.parse()?);
                    }
                    "content" => {
                        focused.parse::<Token![:]>()?;
                        let content_block;
                        braced!(content_block in focused);
                        focused_content = Some(content_block.parse()?);
                    }
                    _ => return Err(syn::Error::new(ident.span(), "Expected 't', 'preparation', or 'content'")),
                }
                if !focused.is_empty() {
                    focused.parse::<Token![,]>()?;
                }

            }
            input.parse::<Token![,]>()?;
            let ident: Ident = input.parse()?;
            if ident != "blurred" {
                return Err(syn::Error::new(ident.span(), "Expected 'blurred'"));
            }
            input.parse::<Token![:]>()?;
            let blurred;
            braced!(blurred in input);
            blurred_span = Some(blurred.span());
            while !blurred.is_empty() {
                let ident: Ident = blurred.parse()?;
                match ident.to_string().as_str() {
                    "t" => {
                        blurred.parse::<Token![:]>()?;
                        let t: Type = blurred.parse()?;
                        blurred_type = Some(t);
                    }
                    "preparation" => {
                        blurred.parse::<Token![:]>()?;
                        let preparation_block;
                        braced!(preparation_block in blurred);
                        blurred_preparation = Some(preparation_block.parse()?);
                    }
                    "content" => {
                        blurred.parse::<Token![:]>()?;
                        let content_block;
                        braced!(content_block in blurred);
                        blurred_content = Some(content_block.parse()?);
                    }
                    _ => return Err(syn::Error::new(ident.span(), "Expected 't', 'preparation', or 'content'")),
                }
                if !blurred.is_empty() {
                    blurred.parse::<Token![,]>()?;
                }
            }

            let focused_span = focused_span.ok_or(syn::Error::new(input.span(), "Missing 'focused' block"))?;
            let blurred_span = blurred_span.ok_or(syn::Error::new(input.span(), "Missing 'blurred' block"))?;

            let focused_type = focused_type.ok_or(syn::Error::new(focused_span, "Missing 'focused' type"))?;
            let blurred_type = blurred_type.ok_or(syn::Error::new(blurred_span, "Missing 'blurred' type"))?;
            arms.push(FocusableArm { matcher, focused_type, focused_preparation, focused_content, blurred_type, blurred_preparation, blurred_content });
        }

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(FocusableInput { iterator, focus, arms })
    }
}

#[proc_macro]
pub fn focusable(input: TokenStream) -> TokenStream {
    let FocusableInput { iterator, focus, arms } = parse_macro_input!(input as FocusableInput);

    let match_arms = arms.iter().map(|arm| {
        let FocusableArm { matcher, focused_type, focused_preparation, focused_content, blurred_type, blurred_preparation, blurred_content } = arm;
        quote! {
            #matcher => {
                if focused {
                    #focused_preparation
                    Some(rsx! {
                        #focused_type {
                            onmounted: move |input| async move {
                                let _ = input.data().set_focus(true).await;
                            },
                            onblur: move |_| { #focus.set(None); },
                            #focused_content
                        }
                    })
                } else {
                    #blurred_preparation
                    Some(rsx! {
                        #blurred_type {
                            onclick: move |_| { #focus.set(Some(index)); },
                            #blurred_content
                        }
                    })
                }
            }
        }
    });

    quote! {
        {
            let mut focus = #focus;
            #iterator.clone().filter_map(move |indexed_part| {
                let #iterator = #iterator.clone();
                let focused = focus.read().is_some_and(|f| f == indexed_part.0 as i8);
                let index = indexed_part.0 as i8;
                match indexed_part.1 {
                    #(#match_arms)*
                    _ => None,
                }
            })
        }
    }
    .into()
}
