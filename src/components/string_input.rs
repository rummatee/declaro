use syntax::ast::{HasStringParts};
use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;

use crate::ast::functions::update_node_value;

#[double]
use crate::ast::hooks as ast_hooks;

#[component]
pub fn StringInput(ptr: ReadSignal<SyntaxNodePtr>) -> Element {
    let node = ast_hooks::use_ast_node::<syntax::ast::String>(ptr);
    let value = node.read().string_parts().filter_map(|part| {
        match part {
        syntax::ast::StringPart::Fragment(text) => Some(text.text().to_string()),
        _ => None,
        }
    }).collect::<Vec<String>>().join("");
    rsx! {
        input {
            class: "string-input simple-input",
            value: value,
            oninput: move |e| {
                println!("New value: {}", e.value());
                update_node_value(
                    node.read().syntax().clone(),
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

#[cfg(test)]
mod tests {
    use crate::ast::mock_hooks::use_ast_node_context;
    use insta::assert_snapshot;
    use super::*;

    #[test]
    fn test_string_input() {
        let use_ast_node_ctx = use_ast_node_context();
        use_ast_node_ctx.expect()
            .returning(|_| {
                Memo::new(|| {
                    let syntax_node = syntax::parse_file("\"foo\"").syntax_node();
                    let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                    syntax::ast::String::cast(expr.syntax().clone()).unwrap()
                })
            });
        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file("foo").syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));

             rsx! { StringInput { ptr: ptr_signal } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}

