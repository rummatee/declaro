use syntax::SyntaxNodePtr;
use syntax::ast::AstNode;
use dioxus::prelude::*;
use focusable_macro::focusable;
use crate::ast::functions::update_node_value;
use mockall_double::double;

#[double]
use crate::components::expression::components as expression_components;

#[double]
use crate::ast::hooks as ast_hooks;

#[component]
pub fn LambdaUI(ptr: ReadSignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let lambda = ast_hooks::use_ast_node::<syntax::ast::Lambda>(ptr);
    let params = lambda.read().param().unwrap().pat().unwrap().fields();

    let focus = use_signal::<Option<i8>>(|| None);
    let enumerated_params = params.clone().enumerate();
    let param_elements = focusable!({
        iterator = enumerated_params,
        focus = focus,
        arms = [
            {
                matcher = _,
                focused = {
                    element_type = input,
                    preparation = {
                        let param_name = indexed_part.1.syntax().text().to_string();
                    },
                    content = {
                        class: "lambda-parameter",
                        value: "{param_name.trim()}",
                        oninput: move |evt| {
                            let new_param_name = evt.value().clone();
                            let new_params = enumerated_params.clone().map(|(i, param)| {
                                if i == indexed_part.0 {
                                    new_param_name.clone()
                                } else {
                                    param.syntax().text().to_string()
                                }
                            }).collect::<Vec<_>>().join(", ");
                            let new_lambda_text = format!("{{{}}} : {}", new_params, lambda.read().body().unwrap().syntax().text());
                            update_node_value(
                                lambda.read().syntax().clone(),
                                &new_lambda_text,
                                |syntax| {
                                    <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                                        .and_then(|sf| sf.expr())
                                        .map(|expr| expr.syntax().clone())
                                }
                            );
                        },
                        onfocusout: move |_| {
                            focus.set(None);
                        }
                    }
                },
                blurred = {
                    element_type = div,
                    preparation = {
                        let param_name = indexed_part.1.syntax().text().to_string();
                    },
                    content = {
                        class: "lambda-parameter",
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
