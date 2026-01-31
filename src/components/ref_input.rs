use ide::{AnalysisHost, FileId};
use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;

use crate::{use_ast_node_strict};
use crate::ast::{update_node_value};

#[component]
pub fn RefInput(ptr: ReadOnlySignal<SyntaxNodePtr>) -> Element {
    let node = use_ast_node_strict!(ptr => syntax::ast::Ref);
    let selected = node.read().token().unwrap();
    let analysis = use_context::<Signal<(AnalysisHost, FileId)>>();

    let bindings_option = get_bindings_in_scope(node.read().syntax(), &analysis.read());

    if bindings_option.is_none() {
        return rsx! {
        }
    }

    let bindings = bindings_option.unwrap();

    let options = bindings
        .iter()
        .map(|label| {
            rsx! {
                option {
                    selected: label == selected.text(),
                    { label.clone() }
                }
            }
        });

    rsx! {
        select { 
            class: "ref-input simple-inout",
            onchange: move |e| {

                update_node_value(
                    node.read().syntax().clone(),
                    &e.value(),
                    |syntax| {
                        <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                            .and_then(|sf| sf.expr())
                            .map(|expr| expr.syntax().clone())
                    }
                );
            },
            {options}
        }
    }
}

pub fn get_bindings_in_scope(node: &SyntaxNode, analysis: &(AnalysisHost, FileId)) -> Option<Vec<String>>{
    let snapshot = analysis.0.snapshot();
    let scopes = snapshot.scopes(analysis.1).ok()?;
    println!("expr_id: {:?}", SyntaxNodePtr::new(node));
    let expr_id = snapshot
        .source_map(analysis.1).unwrap()
        .expr_for_node(SyntaxNodePtr::new(node))?;
    let scope_id = scopes.scope_for_expr(expr_id)?;
    Some(scopes
        .ancestors(scope_id)
        .filter_map(|scope| scope.as_definitions())
        .flatten()
        .map(|(name, _def)| name.to_string())
        .collect::<Vec<String>>())
}
