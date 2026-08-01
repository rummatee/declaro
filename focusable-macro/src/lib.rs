use proc_macro2::TokenStream;
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    Expr, Ident, Token, Type,
};
use quote::quote;

// Parses a sequence of `name = value` pairs where value is either
// a raw token tree in braces, or a regular parsed type.
struct RawField {
    name: Ident,
    value: TokenStream,
}

impl Parse for RawField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;

        let value = if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            let inner = content.parse::<TokenStream>()?;
            quote! { {#inner} }
        } else if input.peek(syn::token::Bracket) {
            // preserve the brackets so ArmsArray can consume them
            let content;
            bracketed!(content in input);
            let inner: TokenStream = content.parse()?;
            quote! { [#inner] }
        } else {
            // Consume raw tokens until the next `,` or `}` at this nesting level
            let mut tokens = vec![];
            while !input.is_empty()
                && !input.peek(Token![,])
                && !input.peek(syn::token::Brace)
            {
                let tt: proc_macro2::TokenTree = input.parse()?;
                tokens.push(tt);
            }
            tokens.into_iter().collect()
        };

        Ok(RawField { name, value })
    }
}

// Parses a `{ name = value, name = value, ... }` block
// and lets you extract fields by name
struct NamedFields(Vec<RawField>);

impl Parse for NamedFields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        let mut fields = vec![];
        while !content.is_empty() {
            fields.push(content.parse::<RawField>()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        Ok(NamedFields(fields))
    }
}

impl NamedFields {
    fn get(&self, name: &str) -> Option<&TokenStream> {
        self.0.iter().find(|f| f.name == name).map(|f| &f.value)
    }

    fn require(&self, name: &str, span: proc_macro2::Span) -> syn::Result<&TokenStream> {
        self.get(name).ok_or_else(|| syn::Error::new(span, format!("missing field `{name}`")))
    }
}

struct Braced(TokenStream);

impl Parse for Braced {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        Ok(Braced(content.parse()?))
    }
}

struct ElementConfig {
    element_type: Type,
    preparation: Option<TokenStream>,
    content: Option<TokenStream>,
}


struct FocusableArm {
    matcher: TokenStream,
    focused: ElementConfig,
    blurred: ElementConfig,
    onfocus: Option<TokenStream>,
}

struct FocusableInput {
    iterator: Expr,
    focus: Expr,
    arms: Vec<FocusableArm>,
}

impl Parse for ElementConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fields = NamedFields::parse(input)?;
        let element_type = syn::parse2(fields.require("element_type", input.span())?.clone())?;
        let preparation = fields.get("preparation").
            map(|ts| syn::parse2::<Braced>(ts.clone()).map(|b| b.0))
            .transpose()?;
        let content = fields.get("content")
            .map(|ts| syn::parse2::<Braced>(ts.clone()).map(|b| b.0))
            .transpose()?;

        Ok(ElementConfig {
            element_type,
            preparation,
            content,
        })
    }
}

impl Parse for FocusableArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fields = NamedFields::parse(input)?;
        let matcher = fields.require("matcher", input.span())?.clone();
        let focused = syn::parse2(fields.require("focused", input.span())?.clone())?;
        let blurred = syn::parse2(fields.require("blurred", input.span())?.clone())?;
        let onfocus = fields.get("onfocus")
            .map(|ts| syn::parse2::<Braced>(ts.clone()).map(|b| b.0))
            .transpose()?;
        Ok(FocusableArm {
            matcher,
            focused,
            blurred,
            onfocus,
        })
    }
}

impl Parse for FocusableInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let fields = NamedFields::parse(input)?;
        let iterator = syn::parse2(fields.require("iterator", input.span())?.clone())?;
        let focus = syn::parse2(fields.require("focus", input.span())?.clone())?;
        let arms_ts = fields.require("arms", span)?.clone();
        let arms = syn::parse2::<ArmsArray>(arms_ts)?.0;
        Ok(FocusableInput {
            iterator,
            focus,
            arms,
        })
    }
}

struct ArmsArray(Vec<FocusableArm>);

impl Parse for ArmsArray {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let mut arms = vec![];
        while !content.is_empty() {
            arms.push(content.parse::<FocusableArm>()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        Ok(ArmsArray(arms))
    }
}

#[proc_macro]
pub fn focusable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let FocusableInput {
        iterator,
        focus,
        arms,
    } = syn::parse_macro_input!(input as FocusableInput);

    let match_arms = arms.iter().map(|arm| {
        let FocusableArm { matcher, focused, blurred , onfocus} = arm;
        let focused_type = &focused.element_type;
        let focused_preparation = &focused.preparation;
        let focused_content = &focused.content;
        let blurred_type = &blurred.element_type;
        let blurred_preparation = &blurred.preparation;
        let blurred_content = &blurred.content;
        quote! {
            #matcher => {
                if focused {
                    #focused_preparation
                    Some(rsx! {
                        #focused_type {
                            onmounted: move |input: Event<MountedData>| async move {
                                let _ = input.set_focus(true).await;
                            },
                            #focused_content
                        }
                    })
                } else {
                    #blurred_preparation
                    Some(rsx! {
                        #blurred_type {
                            onclick: move |_| { 
                                let mut __focus = #focus;
                                __focus.set(Some(index));
                                #onfocus
                            },
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
