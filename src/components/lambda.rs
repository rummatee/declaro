use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use focusable_macro::focusable;
use mockall_double::double;
use crate::ast::functions::update_node_value;
use dioxus_primitives::collapsible;

#[double]
use crate::components::expression::components as expression_components;

#[double]
use crate::ast::hooks as ast_hooks;

#[component]
pub fn LambdaUI(ptr: ReadSignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let lambda = ast_hooks::use_ast_node::<syntax::ast::Lambda>(ptr);
    let params = lambda.read().param().unwrap().pat().unwrap().fields();

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
                        let param_name = indexed_part.1.name().map(|name| name.syntax().text().to_string()).unwrap_or_default();
                        let default_expr = indexed_part.1.default_expr().map(|expr| expr.syntax().clone());
                        let default_expr_text = default_expr.clone().map(|expr| expr.text().to_string());
                        let body = lambda.read().body().unwrap();
                    },
                    content = {
                        oninput: move |evt: Event<FormData>| {
                            let new_name = evt.value().clone();
                            let new_lambda = format!("{{{}}}: {}",enumerated_params.clone().map(|(i, param)| {
                                if i == indexed_part.0 {
                                    match default_expr_text.clone() {
                                        Some(default) => format!("{} ? {}", new_name, default),
                                        None => new_name.clone(),
                                    }
                                } else {
                                    param.syntax().text().to_string()
                                }

                            }).collect::<Vec<_>>().join(", "), body.clone().syntax().text());
                            update_node_value(
                                lambda.read().syntax().clone(),
                                &new_lambda,
                                |syntax| {
                                    <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                                        .and_then(|sf| sf.expr())
                                        .map(|expr| expr.syntax().clone())
                                }
                            );
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
                        let param_name = indexed_part.1.name().map(|name| name.syntax().text().to_string()).unwrap_or_default();
                        let default_expr = indexed_part.1.default_expr().map(|expr| expr.syntax().text().to_string());
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

    let body_ptr = use_memo(move || SyntaxNodePtr::new(lambda.read().body().unwrap().syntax()));

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
            expression_components::ExpressionUI { ptr: body_ptr, nesting_level: nesting_level }
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
                            if onmounted.is_some() {
                                onmounted.unwrap().call(evt)
                            }
                        },
                        onblur: move |evt| {
                            if onblur.is_some() {
                                onblur.unwrap().call(evt)
                            }
                        },
                        oninput: move |evt| {
                            if oninput.is_some() {
                                oninput.unwrap().call(evt)
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
                        ptr: SyntaxNodePtr::new(&default_expr), 
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
                        if onmounted.is_some() {
                            onmounted.unwrap().call(evt)
                        }
                    },
                    onblur: move |evt| {
                        if onblur.is_some() {
                            onblur.unwrap().call(evt)
                        }
                    },
                    oninput: move |evt| {
                        if oninput.is_some() {
                            oninput.unwrap().call(evt)
                        }
                    },
                    onfocusout: move |evt| {
                        if onfocusout.is_some() {
                            onfocusout.unwrap().call(evt)
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
    use crate::ast::mock_hooks::use_ast_node_context;
    use crate::components::expression::mock_components::ExpressionUI_context;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_lambda_ui() {
        let use_ast_node_ctx = use_ast_node_context();
        let expression_ui_ctx = ExpressionUI_context();
        const SOURCE: &str = r#"
        { var1, var2 ? "default" } : {}
        "#;
        use_ast_node_ctx.expect()
            .returning(|_| {
                Memo::new(|| {
                    let syntax_node = syntax::parse_file(SOURCE).syntax_node();
                    let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                    syntax::ast::Lambda::cast(expr.syntax().clone()).unwrap()
                })
            });
        expression_ui_ctx.expect()
            .returning(|props| {
            rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file(SOURCE).syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));
             rsx! { LambdaUI { ptr: ptr_signal, nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}
