use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;

use crate::components::expression::components::ExpressionUI;

#[double]
use crate::components::expression::components as expression_components;

#[double]
use crate::ast::functions as ast_functions;

#[component]
pub fn LambdaUI(ptr: ReadSignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let lambda = ast_functions::use_ast_node::<syntax::ast::Lambda>(ptr);
    let params = lambda.read().param().unwrap().pat().unwrap().fields();

    let param_elements = params.map(|param| {
        let label = param.syntax().text().to_string();
        rsx! {
            li { "{label}" }
        }
    });

    let body_ptr = use_memo(move || SyntaxNodePtr::new(lambda.read().body().unwrap().syntax()));

    rsx! {
        div {
            class: "lambda-node",
            h3 { "Lambda Function" }
            div {
                class: "lambda-parameters",
                h4 { "Parameters:" }
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
    use crate::ast::mock_functions::use_ast_node_context;
    use crate::components::expression::mock_components::ExpressionUI_context;

    #[test]
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
