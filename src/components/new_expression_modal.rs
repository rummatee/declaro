use dioxus::prelude::*;
use std::rc::Rc;
use crate::ast::functions::is_valid_expression;

pub struct ExpressionType<'a> {
    pub name: &'a str,
    pub expression: &'a str,
}

const MENU_ITEMS: &'static[ExpressionType] = &[
    ExpressionType {name: "Attribute Set", expression: "{}"},
    ExpressionType {name: "Array", expression: "[]"},
    ExpressionType {name: "String", expression: "\"\""},
    ExpressionType {name: "Number", expression: "0"},
    ExpressionType {name: "Boolean", expression: "true"},
    ExpressionType {name: "Let In", expression: "let x = 0; in {}"},
    ExpressionType {name: "Lambda", expression: "{}: {}"},
];

#[component]
pub fn NewExpressionModal(onclose: Option<EventHandler<Event<MouseData>>>, onchoose: EventHandler<Event<String>>) -> Element {
    let mut selected: Signal<Option<usize>> = use_signal(|| None);
    let mut expression_string: Signal<String> = use_signal(|| String::new());
    let can_submit = is_valid_expression(expression_string.read().as_str());
    let menu_elements: Box<dyn Iterator<Item = Element>> = Box::new(MENU_ITEMS.iter().enumerate().map(|(index, item)| {
        let is_selected = selected.read().map_or(false, |selected_index| selected_index == index);
        rsx! {
            li {
                class: if is_selected { "selected" } else { "" },
                onclick: move |_| {
                    selected.set(Some(index));
                    expression_string.set(item.expression.to_string());
                },
                "{item.name}"
            }
        }
    }));
    rsx! {
        div {
            class: "modal-overlay",
            div {
                class: "modal",
                div {
                    class: "modal-header",
                    h2 { "New Expression" }
                    button {
                        class: "close-button",
                        onclick: move |evt| {
                            if let Some(callback) = onclose {
                                callback.call(evt);
                            }
                        },
                        "×"
                    }
                }
                div {
                    class: "modal-body",
                    input {
                        class: "expression-input",
                        value: expression_string.read().as_str(),
                        oninput: move |evt| {
                            expression_string.set(evt.value().clone());
                        }
                    }
                    ul {
                        class: "expression-type-list",
                        { menu_elements }
                    }
                    button {
                        class: {
                            if can_submit {
                                "submit-button"
                            } else {
                                "submit-button disabled"
                            }
                        },
                        onclick: move |_| {
                            if can_submit {
                                let expression = expression_string.read();
                                if !expression.is_empty() {
                                    onchoose.call(Event::new(Rc::new(expression.clone()), false));
                                }
                            }
                        },
                        "Submit"
                    }
                }
            }
        }
    }
}
