use rnix::ast::HasEntry;
use rnix::ast::{AttrSet, Entry};
use rowan::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use focusable_macro::focusable;
use crate::ast::functions::add_binding;
use std::iter::zip;

#[double]
use crate::components::expression::components as expression_components;

#[component]
pub fn AttributeSetUI(set: ReadSignal<AttrSet>, nesting_level: u16) -> Element {
    let bindings = set.read().entries();
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
                        let new_label = format!("{} ", evt.value().clone());
                        let range = attr.attrpath().unwrap().syntax().text_range();
                        let mut source = use_context::<Signal<String>>();
                        source.write().replace_range(usize::from(range.start())..usize::from(range.end()), &new_label);
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
            class: "attribute-set binding-set",
            { elements }
            span { 
                class: "add-binding",
                onclick: move |_| {
                    add_binding(bindings.clone());
                },
                "+"
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
    fn test_attribute_set_ui() {
        let expression_ui_ctx = ExpressionUI_context();
        const SOURCE: &str = r#"
        {
            a = 1;
            b = 2;
        }
        "#;
        expression_ui_ctx.expect()
            .returning(|props| {
                rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse(SOURCE);
            let set = AttrSet::cast(root.tree().expr().unwrap().syntax().clone()).unwrap();
            rsx! { AttributeSetUI { set, nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        expression_ui_ctx.checkpoint();
        assert_snapshot!(html);
    }
}
