use syntax::ast::{HasBindings, HasStringParts};
use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;

use crate::{use_ast_node_strict};
use crate::ast::path_from_root;
use crate::components::stringInput::StringInput;

#[component]
pub fn AttributeSetUI(ptr: ReadOnlySignal<SyntaxNodePtr>) -> Element {
    let set = use_ast_node_strict!(ptr => syntax::ast::AttrSet);
    let elements = set.read().bindings()
        .filter_map(|binding| match binding {
            syntax::ast::Binding::AttrpathValue(attr) => Some(attr),
            _ => None,
                    })
        .map(|attr| {
            let label = attr.attrpath()
                .map(|ap| ap.syntax().text().to_string())
                .unwrap_or("unknown".to_string());
            let valueUI = match attr.value() {
                    Some(val) => {
                        let node = val.syntax();
                        match_ast!{
                        match node {
                            syntax::ast::String(_str_node) => {
                                let ptr = SyntaxNodePtr::new(&node);
                                rsx! { StringInput { ptr: ptr, id: format!("input-{}", label) } }
                            },
                            syntax::ast::AttrSet(_set_node) => {
                                rsx! { Link {
                                    to: crate::router::Route::NodeUI{ path: path_from_root(&node)},
                                    "AttrSet"
                                    }
                                }
                            },
                            _ => rsx! { div { "Unsupported Value Type" } },
                        }
                    }},
                    _ => return rsx! { div { "Unsupported" } }
                };
        rsx! {
            div {
                class: "attribute-item",
                label {
                    class: "attribute-label",
                    "{label}"
                }
                {valueUI}
            }
        }
    });
    rsx! {
        div {
            class: "attribute-set",
            { elements }
        }
    }
}
