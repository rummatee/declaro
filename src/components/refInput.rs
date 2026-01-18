use dioxus_primitives::select;
use ide::{AnalysisHost, FileId};
use ide::DefDatabase;
use syntax::ast::{HasStringParts};
use syntax::{SyntaxNode, SyntaxNodePtr, TextSize};
use syntax::ast::AstNode;
use dioxus::prelude::*;

use crate::{use_ast_node_strict};
use crate::ast::{update_node_value};

#[component]
pub fn RefInput(ptr: ReadOnlySignal<SyntaxNodePtr>, id: String) -> Element {
    let node = use_ast_node_strict!(ptr => syntax::ast::Ref);
    let selected = node.read().token().unwrap().text();
    let analysis = use_context::<Signal<(AnalysisHost, FileId)>>();
    /* Approach using completions. The problem is that using the start position, it doesn't
     * recognize as inside of the expression and no completions are returned,
     * however going any character further filters the results by that prefix
    let options = analysis.read().0.snapshot().completions(
        ide::FilePos { file_id: analysis.read().1, pos: node.read().token().unwrap().text_range().start() },
        None
    )?.into_iter().filter(
        |item| {
            println!("completion item: {}", item.label);
            matches!(item.kind, ide::CompletionItemKind::Param)
        }
    ).enumerate().map(|(i, item)| {
        let label = item.label.clone();
        rsx! {
            option { { label } }
        }
    });
    */

    let snapshot = analysis.read().0.snapshot();
    let scopes = snapshot.scopes(analysis.read().1).unwrap();
    let expr_id = snapshot
        .source_map(analysis.read().1).unwrap()
        .expr_for_node(SyntaxNodePtr::new(node.read().syntax())).unwrap();
    let scope_id = scopes.scope_for_expr(expr_id).unwrap();
    let options = scopes
        .ancestors(scope_id)
        .filter_map(|scope| scope.as_definitions())
        .flatten()
        .map(|(name, _def)| {
            let label = name.to_string();
            rsx! {
                option {
                    selected: label == node.read().token().unwrap().text(),
                    { label.clone() }
                }
            }
        });

    rsx! {
        select { 
            {options}
        }
    }
}
