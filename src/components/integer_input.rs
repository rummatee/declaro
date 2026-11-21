use rnix::ast::{Integer, AstToken};
use rowan::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use crate::ast::functions::update_node_value;

#[double]
use crate::components::expression::components as expression_components;

#[component]
pub fn IntegerInput(integer: ReadSignal<Integer>) -> Element {
    let value = integer.read().value().unwrap_or_default();
    rsx! {
        input {
            class: "integer-input simple-input",
            value: "{value}",
            type: "number",
            oninput: move |evt| {
                update_node_value(integer.read().syntax().clone().into(), evt.value().as_ref());
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
    fn test_integer_input() {
        let expression_ui_ctx = ExpressionUI_context();
        const SOURCE: &str = "123";
        expression_ui_ctx.expect()
            .returning(|props| {
                rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse(SOURCE);
            let integer = Integer::cast(root.tree().expr().unwrap().syntax().children_with_tokens().next().unwrap().into_token().unwrap()).unwrap();
            rsx! { IntegerInput { integer } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        expression_ui_ctx.checkpoint();
        assert_snapshot!(html);
    }
}
