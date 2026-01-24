use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;

use crate::components::ExpressionUI;
use crate::use_ast_node_strict;

#[component]
pub fn LambdaUI(ptr: ReadOnlySignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let lambda = use_ast_node_strict!(ptr => syntax::ast::Lambda);
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
            ExpressionUI { ptr: body_ptr, nesting_level: nesting_level }
        }
    }
}
