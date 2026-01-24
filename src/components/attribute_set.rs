use syntax::ast::{HasBindings};
use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;

use crate::components::ExpressionUI;
use crate::{use_ast_node_strict};
use crate::ast::path_from_root;
use crate::components::string_input::StringInput;
use crate::components::ref_input::RefInput;

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
            let value = attr.value().unwrap();
            let node = value.syntax();
            let ptr = SyntaxNodePtr::new(node);
        rsx! {
            div {
                class: "attribute-item",
                label {
                    class: "attribute-label",
                    "{label}"
                }
                ExpressionUI { ptr: ptr }
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
