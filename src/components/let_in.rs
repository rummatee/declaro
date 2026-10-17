use rnix::ast::{LetIn, HasEntry, Entry};
use rowan::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use focusable_macro::focusable;
use crate::ast::functions::{add_binding, update_node_value};
use crate::components::expression::components::ExpressionUI;
use std::iter::zip;

#[double]
use crate::components::expression::components as expression_components;

#[component]
pub fn LetInUI(expression: ReadSignal<LetIn>, nesting_level: u16) -> Element {
    let bindings = expression.read().entries();
    let body = expression.read().body();
    let enumerated = bindings.clone().enumerate();
    let focus = use_signal::<Option<i8>>(|| None);
    let labels = focusable!({
        iterator = enumerated,
        focus = focus,
        arms = [
        {
            matcher = Entry::AttrpathValue(attr),
            focused = {
                element_type = input,
                preparation = {
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                },
                content = {
                    class: "attribute-label",
                    value: "{label.trim()}",
                    oninput: move |evt| {
                        /* We probably want rename here again
                         * I'm planning to use a language server client to allow using arbitrary
                         * language servers.
                         * For now, just edit only locally */
                        update_node_value(attr.attrpath().unwrap().syntax().clone().into(), evt.value().as_ref());

                    },
                    onfocusout: move |_| {
                        focus.set(None);
                    }
                }
            },
            blurred = {
                element_type = label,
                preparation = {
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                    },
                    content = {
                        class: "attribute-label",
                        "{label}"
                    }
            }
        }
        ]
    });
    let elements = zip(labels,bindings.clone()).map(|(label, binding)| {
        let attr = match binding {
            Entry::AttrpathValue(attr) => attr,
            _ => return rsx! { div { "Unsupported binding type" } },
        };
        let value = attr.value().unwrap();
        let node = value.syntax();
        rsx! {
            div {
                class: "attribute-item",
                {label}
                expression_components::ExpressionUI { node: node.clone(), nesting_level: nesting_level }
            }
        }
    });
    rsx! {
        div {
            class: "let-in",
            h3 {
                class: "binding-set-header let-header",
                "let"
            }
            div {
                class: "let-in-bindings binding-set",
                { elements }
                span { 
                    class: "add-binding",
                    onclick: move |_| {
                        add_binding(bindings.clone());
                    },
                    "+"
                }
            }
            h3 {
                class: "binding-set-header in-header",
                "in"
            }
            if let Some(body) = body {
                ExpressionUI{ node: body.syntax().clone(), nesting_level: nesting_level}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use super::*;
    use crate::components::expression::mock_components::ExpressionUI_context;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_let_in_ui() {
        let expression_ui_ctx = ExpressionUI_context();
        const SOURCE: &str = r#"
        let a = 1; b = 2; in { a + b }
        "#;
        expression_ui_ctx.expect()
            .returning(|props| {
                rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse(SOURCE);
            let expression = LetIn::cast(root.tree().expr().unwrap().syntax().clone()).unwrap();
            rsx! { LetInUI { expression, nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        expression_ui_ctx.checkpoint();
        assert_snapshot!(html);
    }
}
