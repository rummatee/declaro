use syntax::ast::{HasStringParts};
use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;

use crate::{use_ast_node_strict};
use crate::ast::{update_node_value};

#[component]
pub fn StringInput(ptr: SyntaxNodePtr, id: String) -> Element {
    let node = use_ast_node_strict!(ptr => syntax::ast::String);
    let value = node.string_parts().filter_map(|part| {
        match part {
        syntax::ast::StringPart::Fragment(text) => Some(text.text().to_string()),
        _ => None,
        }
    }).collect::<Vec<String>>().join("");
    rsx! {
        input {
            class: "string-input",
            id: "{id}",
            value: value,
            oninput: move |e| {
                println!("New value: {}", e.value());
                update_node_value(
                    node.clone(),
                    &format!("\"{}\"", e.value()),
                    |syntax| {
                        <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                            .and_then(|sf| sf.expr())
                            .map(|expr| expr.syntax().clone())
                    }
                );
            }
        }
    }
}
