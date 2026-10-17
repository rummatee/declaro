use rnix::ast::{Str, StrContent, InterpolPart, AstToken};
use rowan::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use focusable_macro::focusable;

use crate::ast::functions::update_node_value;

#[double]
use crate::ast::functions as ast_functions;

#[cfg(test)]
use mockall::automock;

#[derive(Props, PartialEq, Clone)]
pub struct StringInputProps {
    string: ReadSignal<Str>,
}

#[cfg_attr(test, automock)]
pub mod components { 

    use super::*;

    #[allow(non_snake_case)]
    pub fn StringInput(props: StringInputProps) -> Element {
        let string = props.string;
        let bindings = ast_functions::get_bindings_in_scope(string.read().syntax()).unwrap_or_default();
        let mut focus = use_signal::<Option<i8>>(|| None);

        let string_parts = string.read().parts().enumerate();
        let mut prev_value = use_signal(|| "".to_string());
        let elements = focusable!({
            iterator = string_parts,
            focus = focus,
            arms = [
            {
                matcher = InterpolPart::Literal(text),
                focused = {
                    element_type = input,
                    content = {
                        class: "string-fragment focused",
                        value: "{text}",
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

                            update_node_value(
                                text.syntax().clone().into(),
                                &own_value
                            );
                        }
                    },
                },
                blurred = {
                    element_type = span,
                    content = {
                        class: "string-fragment",
                        "{text}"
                    }
                },
                onfocus = {
                    prev_value.set(text.to_string());
                }
            },
            {
                matcher = InterpolPart::Interpolation(dynamic),
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

                            update_node_value(
                                dynamic.expr().unwrap().syntax().clone().into(),
                                &e.value().as_ref()
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
    use crate::ast::mock_functions::get_bindings_in_scope_context;
    use serial_test::serial;
    use insta::assert_snapshot;
    use super::*;

    #[test]
    #[serial]
    fn test_string_input() {
        let get_bindings_in_scope_ctx = get_bindings_in_scope_context();
        get_bindings_in_scope_ctx.expect()
            .returning(|_| {
            Ok(vec!["foo".to_string(), "bar".to_string()])
            });
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse("\"Hello ${foo}\"").tree();
            let string = Str::cast(root.expr().unwrap().syntax().clone()).unwrap();
            rsx! { components::StringInput { string } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}

