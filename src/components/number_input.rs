use rnix::ast::{Integer, Float, AstToken};
use dioxus::prelude::*;
use crate::ast::functions::update_node_value;

pub enum Number {
    Integer(Integer),
    Float(Float),
}

impl Number {
    fn syntax(&self) -> &rnix::SyntaxToken {
        match self {
            Number::Integer(i) => i.syntax(),
            Number::Float(f) => f.syntax(),
        }
    }
}

#[component]
pub fn NumberInput(number: ReadSignal<Number>) -> Element {
    let token = number.read();
    let value = token.syntax().text();
    rsx! {
        input {
            class: "integer-input simple-input",
            value: "{value}",
            type: "number",
            oninput: move |evt| {
                update_node_value(number.read().syntax().clone().into(), evt.value().as_ref());
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
    use rowan::ast::AstNode;

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
            let number = Number::Integer(integer);
            rsx! { NumberInput { number } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        expression_ui_ctx.checkpoint();
        assert_snapshot!(html);
    }
}
