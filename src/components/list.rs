use rnix::ast::List;
use rowan::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use crate::ast::functions::update_node_value;
use crate::components::new_expression_modal;

#[double]
use crate::components::expression::components as expression_components;

#[component]
pub fn ListUI(list: ReadSignal<List>, nesting_level: u16) -> Element {
    let elements = list.read().items();
    let mut modal_open = use_signal(|| false);
    rsx! {
        div {
            class: "list-container",
            {elements.enumerate().map(|(i,element)| {
                rsx! {
                    div {
                        class: "list-item",
                        expression_components::ExpressionUI { node: element.syntax().clone(), nesting_level: nesting_level + 1, type_changeble: false}
                        span { 
                            class: "list-item-remove",
                            onclick: move |_| {
                                let new_list = list.read().items().enumerate().filter(|(j, _item)| i != *j).collect::<Vec<_>>();
                                let mut new_list_syntax = new_list.iter().fold(String::from("["), |mut acc, (_j,item)| {
                                    acc.push_str(&item.syntax().text().to_string());
                                    acc.push(' ');
                                    acc
                                });
                                new_list_syntax.push(']');
                                update_node_value(list.read().syntax().clone().into(), &new_list_syntax);
                            },
                            "x"
                        }
                    }
                }
            })}
            span {
                class: "list-item-add",
                onclick: move |_| {
                    modal_open.set(true);
                },
                "+"
            }
        }
        { if *modal_open.read() {
            rsx! {
                new_expression_modal::NewExpressionModal {
                    onclose: Some(EventHandler::new(move |_| {
                        modal_open.set(false);
                    })),
                    onchoose: move |evt: Event<String>| {
                        let expression = evt.data;
                        let mut new_list_syntax = list.read().items().fold(String::from("["), |mut acc, item| {
                            acc.push_str(&item.syntax().text().to_string());
                            acc.push(' ');
                            acc
                        });
                        new_list_syntax.push_str(&expression);
                        new_list_syntax.push(']');
                        update_node_value(list.read().syntax().clone().into(), &new_list_syntax);
                        modal_open.set(false);
                    }
                }
            }
        } else {
            rsx! {}
        }}

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
    fn test_list_ui() {
        let expression_ui_ctx = ExpressionUI_context();
        const SOURCE: &str = r#"
        [ 1 "string" true ]
        "#;
        expression_ui_ctx.expect()
            .returning(|props| {
                rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse(SOURCE);
            let list = List::cast(root.tree().expr().unwrap().syntax().clone()).unwrap();
            rsx! { ListUI { list, nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        expression_ui_ctx.checkpoint();
        assert_snapshot!(html);
    }
}
