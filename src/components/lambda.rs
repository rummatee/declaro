use rnix::{SyntaxNode, ast::{Lambda, Pattern}};
use rowan::ast::AstNode;
use dioxus::prelude::*;
use focusable_macro::focusable;
use mockall_double::double;
use crate::ast::functions::update_node_value;
use dioxus_primitives::collapsible;

#[double]
use crate::components::expression::components as expression_components;

#[component]
pub fn LambdaUI(lambda: ReadSignal<Lambda>, nesting_level: u16) -> Element {
    let params = Pattern::try_from(lambda.read().param().unwrap()).expect("Expected a Pattern node").pat_entries();

    let focus = use_signal::<Option<i8>>(|| None);
    let params_copy = params.clone();
    let enumerated_params = params_copy.enumerate();
    let param_elements = focusable!({
        iterator = enumerated_params,
        focus = focus,
        arms = [
            {
                matcher = _,
                focused = {
                    element_type = LambdaParameter,
                    preparation = {
                        let param_name_node = indexed_part.1.ident().map(|name| name.syntax().clone());
                        let param_name = param_name_node.clone().map(|node| node.text().to_string()).unwrap_or_default();
                        let default_expr = indexed_part.1.default().map(|expr| expr.syntax().clone());
                    },
                    content = {
                        oninput: move |evt: Event<FormData>| {
                            /* We probably want rename here again
                            * I'm planning to use a language server client to allow using arbitrary
                            * language servers.
                            * For now, just edit only locally */
                            update_node_value(param_name_node.clone().unwrap().into(), evt.value().as_ref());
                        },
                        onfocusout: move |_| {
                            focus.set(None);
                        },
                        name: param_name,
                        nesting_level: nesting_level,
                        default: default_expr
                    }
                },
                blurred = {
                    element_type = div,
                    preparation = {
                        let param_name = indexed_part.1.ident().map(|name| name.syntax().text().to_string()).unwrap_or_default();
                        let default_expr = indexed_part.1.default().map(|expr| expr.syntax().text().to_string());
                    },
                    content = {
                        class: "lambda-parameter",
                        title: {default_expr},
                        "{param_name.trim()}"
                    }
                }
            }
        ]
    });

    let body = lambda.read().body();

    rsx! {
        div {
            class: "lambda-node",
            div {
                class: "lambda-symbol",
                "λ"
            }
            div {
                class: "lambda-parameters",
                ul {
                    { param_elements }
                }
            }
            expression_components::ExpressionUI { node: body.unwrap().syntax().clone(), nesting_level: nesting_level }
        }
    }
}

#[component]
pub fn LambdaParameter(
    onmounted: Option<EventHandler<Event<MountedData>>>,
    onblur: Option<EventHandler<Event<FocusData>>>,
    oninput: Option<EventHandler<Event<FormData>>>,
    onfocusout: Option<EventHandler<Event<FocusData>>>,
    name: String,
    default: Option<SyntaxNode>,
    nesting_level: u16,
) -> Element {
    if let Some(default_expr) = default {
        rsx! {
            collapsible::Collapsible {
                collapsible::CollapsibleTrigger {
                    class: "lambda-parameter",
                    input {
                        onmounted: move |evt| {
                            if let Some(callback) = onmounted {
                                callback.call(evt)
                            }
                        },
                        onblur: move |evt| {
                            if let Some(callback) = onblur {
                                callback.call(evt)
                            }
                        },
                        oninput: move |evt| {
                            if let Some(callback) = oninput {
                                callback.call(evt)
                            }
                        },
                        value: "{name.trim()}",
                    }
                    dioxus_free_icons::Icon {
                        icon: dioxus_free_icons::icons::fa_solid_icons::FaChevronDown,
                        class: "lambda-parameter-default-icon"
                    }
                }
                collapsible::CollapsibleContent {
                    class: "lambda-parameter-default",
                    expression_components::ExpressionUI { 
                        node: default_expr, 
                        nesting_level: nesting_level + 1
                    }
                }
            }
        }
    } else {
        rsx! {
            input {
                class: "lambda-parameter",
                    onmounted: move |evt| {
                        if let Some(callback) = onmounted {
                            callback.call(evt)
                        }
                    },
                    onblur: move |evt| {
                        if let Some(callback) = onblur {
                            callback.call(evt)
                        }
                    },
                    oninput: move |evt| {
                        if let Some(callback) = oninput {
                            callback.call(evt)
                        }
                    },
                    onfocusout: move |evt| {
                        if let Some(callback) = onfocusout {
                            callback.call(evt)
                        }
                    },
                    value: "{name.trim()}"
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
    fn test_lambda_ui() {
        let expression_ui_ctx = ExpressionUI_context();
        const SOURCE: &str = r#"
        { var1, var2 ? "default" } : {}
        "#;
        expression_ui_ctx.expect()
            .returning(|props| {
            rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse(SOURCE).tree();
            let lambda = Lambda::cast(root.expr().unwrap().syntax().clone()).unwrap();
             rsx! { LambdaUI { lambda, nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}
