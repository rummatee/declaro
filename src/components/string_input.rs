use syntax::ast::{HasStringParts};
use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use thiserror::Error;

use crate::ast::functions::update_node_value;

#[double]
use crate::ast::hooks as ast_hooks;
#[double]
use crate::utils::hooks;


#[component]
pub fn StringInput(ptr: ReadSignal<SyntaxNodePtr>) -> Element {
    let node = ast_hooks::use_ast_node::<syntax::ast::String>(ptr);
    let analysis = hooks::use_analysis_host();
    let bindings = crate::ast::functions::get_bindings_in_scope(node.read().syntax(), &analysis.read()).unwrap_or_default();
    let mut focus = use_signal::<Option<i8>>(|| None);

    let string_parts = node.read().string_parts().enumerate();
    let mut prev_value = use_signal(|| "".to_string());
    let elements = string_parts.clone().filter_map(|indexed_part| {
        let focused = focus.read().is_some_and(|f| f == indexed_part.0 as i8);
        let string_parts = string_parts.clone();
        match indexed_part.1 {
            syntax::ast::StringPart::Fragment(text) => Some(rsx! {
                if focused {
                    input {
                        class: "string-fragment focused",
                        value: "{text.text()}",
                        onmounted: move |input| async move {
                            let _ = input.data().set_focus(true).await;
                        },
                        oninput: move |evt| {
                            let mut own_value = evt.value().clone();
                            let prev = prev_value.read().clone();
                            prev_value.set(own_value.clone());
                            // Find the first position where the strings differ
                            let diff_pos = prev.chars()
                                .zip(own_value.chars())
                                .position(|(a, b)| a != b)
                                .unwrap_or(prev.len().min(own_value.len()));
                            // Check if "${" was inserted at diff_pos
                            if own_value.len() > prev.len() &&
                                own_value.get(diff_pos-1..diff_pos+1) == Some("${") {
                                own_value.insert_str(diff_pos+1, "a}");
                                let new_focus = focus.read().unwrap() + 1;
                                focus.set(Some(new_focus));
                            }
                            let new_value_inner = string_parts.clone().map(|part| {
                                if part.0 == indexed_part.0 {
                                    own_value.clone()
                                } else {
                                    match part.1 {
                                        syntax::ast::StringPart::Fragment(t) => t.text().to_string(),
                                        syntax::ast::StringPart::Dynamic(t) => format!("${{{}}}", t.expr().unwrap().syntax().text()),
                                        _ => "".to_string(),
                                    }
                                }
                            }).collect::<Vec<_>>().join("");

                            let new_value = format!("\"{}\"", new_value_inner);

                            update_node_value(
                                node.read().syntax().clone(),
                                &new_value,
                                |syntax| {
                                    <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                                        .and_then(|sf| sf.expr())
                                        .map(|expr| expr.syntax().clone())
                                }
                            );
                        }
                    }
                } else {
                    span {
                        class: "string-fragment",
                        onclick: move |_| {
                            focus.set(Some(indexed_part.0 as i8));
                            prev_value.set(text.text().to_string());
                        },
                        "{text.text()}"
                    } 
                }
            }),
            syntax::ast::StringPart::Dynamic(dynamic) => {
                let expr_text = dynamic.expr().unwrap().syntax().text();
                if focused {

                    let options = bindings
                        .iter()
                        .map(|label| {
                            rsx! {
                                option {
                                    selected: label == &expr_text.to_string(),
                                    { label.clone() }
                                }
                            }
                        });

                    Some(rsx! {
                        select { 
                            class: "ref-input simple-inout",
                            onmounted: move |input| async move {
                                let _ = input.data().set_focus(true).await;
                            },
                            onchange: move |e| {
                                let new_value_inner = string_parts.clone().map(|part| {
                                    if part.0 == indexed_part.0 {
                                        format!("${{{}}}", e.value())
                                    } else {
                                        match part.1 {
                                            syntax::ast::StringPart::Fragment(t) => t.text().to_string(),
                                            syntax::ast::StringPart::Dynamic(t) => format!("${{{}}}", t.expr().unwrap().syntax().text()),
                                            _ => "".to_string(),
                                        }
                                    }
                                }).collect::<Vec<_>>().join("");

                                let new_value = format!("\"{}\"", new_value_inner);

                                update_node_value(
                                    node.read().syntax().clone(),
                                    &new_value,
                                    |syntax| {
                                        <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                                            .and_then(|sf| sf.expr())
                                            .map(|expr| expr.syntax().clone())
                                    }
                                );
                            },
                            {options}
                        }
                    })
                } else {
                    Some(rsx! {
                        span {
                            class: "dynamic-fragment",
                            title: "{bindings.join(\", \")}",
                            onclick: move |_| {
                                focus.set(Some(indexed_part.0 as i8));
                            },
                            {expr_text.to_string()}
                        }
                    })
                }
            }
            _ => None,
        }
    }).map(|part| part.unwrap()).collect::<Vec<_>>();

    rsx! {
        div {
            class: "string-input simple-input",
            onfocusout: move |_| {
                focus.set(None);
            },
            for element in elements {
                {element}
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

