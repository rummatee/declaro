use syntax::ast::{HasStringParts};
use syntax::SyntaxNodePtr;
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use focusable_macro::focusable;

use crate::ast::functions::update_node_value;

#[double]
use crate::ast::hooks as ast_hooks;
#[double]
use crate::utils::hooks;

#[cfg(test)]
use mockall::automock;

#[derive(Props, PartialEq, Clone)]
pub struct StringInputProps {
    ptr: ReadSignal<SyntaxNodePtr>,
}

#[cfg_attr(test, automock)]
pub mod components { 

    use super::*;

    #[allow(non_snake_case)]
    pub fn StringInput(props: StringInputProps) -> Element {
        let ptr = props.ptr;
        let node = ast_hooks::use_ast_node::<syntax::ast::String>(ptr);
        let analysis = hooks::use_analysis_host();
        let bindings = crate::ast::functions::get_bindings_in_scope(node.read().syntax(), &analysis.read()).unwrap_or_default();
        let mut focus = use_signal::<Option<i8>>(|| None);

        let string_parts = node.read().string_parts().enumerate();
        let mut prev_value = use_signal(|| "".to_string());
        let elements = focusable!({
            iterator = string_parts,
            focus = focus,
            arms = [
            {
                matcher = syntax::ast::StringPart::Fragment(text),
                focused = {
                    element_type = input,
                    content = {
                        class: "string-fragment focused",
                        value: "{text.text()}",
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
                                &new_value
                            );
                        }
                    },
                },
                blurred = {
                    element_type = span,
                    content = {
                        class: "string-fragment",
                        "{text.text()}"
                    }
                },
                onfocus = {
                    prev_value.set(text.text().to_string());
                }
            },
            {
                matcher = syntax::ast::StringPart::Dynamic(dynamic),
                focused = {
                    element_type = select,
                    preparation = {
                        let expr_text = dynamic.expr().unwrap().syntax().text();
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
                    },
                    content = { 
                        class: "ref-input simple-input",
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
                                &new_value
                            );
                        },
                        {options}
                    },
                },
                blurred = {
                    element_type = span,
                    preparation = {
                        let expr_text = dynamic.expr().unwrap().syntax().text();
                    },
                    content = {
                        class: "dynamic-fragment",
                        title: "{bindings.join(\", \")}",
                        {expr_text.to_string()}
                    }
                }
            }
        ]});

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
}

#[cfg(test)]
mod tests {
    use crate::ast::mock_hooks::use_ast_node_context;
    use serial_test::serial;
    use insta::assert_snapshot;
    use ide::AnalysisHost;
    use super::*;

    #[test]
    #[serial]
    fn test_string_input() {
        let use_ast_node_ctx = use_ast_node_context();
        let use_analysis_host_ctx = crate::utils::mock_hooks::use_analysis_host_context();
        use_ast_node_ctx.expect()
            .returning(|_| {
            Memo::new(|| {
                let syntax_node = syntax::parse_file("foo").syntax_node();
                let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                syntax::ast::Ref::cast(expr.syntax().clone()).unwrap()
            })
            });
        use_analysis_host_ctx.expect()
            .returning(|| {
            let analysis_host = AnalysisHost::new_single_file("");
            Signal::new(analysis_host)
            });
        use_ast_node_ctx.expect()
            .returning(|_| {
                Memo::new(|| {
                    let syntax_node = syntax::parse_file("\"Hello ${foo}\"").syntax_node();
                    let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                    syntax::ast::String::cast(expr.syntax().clone()).unwrap()
                })
            });
        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file("foo").syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));

             rsx! { components::StringInput { ptr: ptr_signal } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}

