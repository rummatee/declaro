use ide::{AnalysisHost, FileId};
use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;

#[double]
use util_functions as utils;

#[cfg(test)]
use mockall::automock;

use crate::hooks::use_analysis_host;


#[component]
pub fn RefInput(ptr: ReadSignal<SyntaxNodePtr>) -> Element {
    let analysis = use_analysis_host();
    println!("Rendering RefInput with ptr: {:?}", ptr.read());
    let node = crate::ast::use_ast_node_generic::<syntax::ast::Ref>(ptr);
    println!("Node: {:?}", node.read());
    let selected = node.read().token().unwrap();

    let bindings_option = utils::get_bindings_in_scope(node.read().syntax(), &analysis.read());
    println!("Bindings in scope: {:?}", bindings_option);

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

    println!("Options: {:?}", options);

    rsx! {
        select { 
            class: "ref-input simple-inout",
            onchange: move |e| {

                crate::ast::update_node_value(
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

#[cfg_attr(test, automock)]
mod util_functions {
    use super::*;
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
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use super::*;
    #[test]
    fn test_ref_input() {
        let use_ast_node_ctx = crate::ast::use_ast_node_generic_context();
        let use_analysis_host_ctx = crate::hooks::use_analysis_host_context();
        let get_bindings_in_scope_ctx = mock_util_functions::get_bindings_in_scope_context();
        use_ast_node_ctx.expect()
            .returning(|_| {
                Memo::new(|| {
                    let syntax_node = syntax::parse_file("foo").syntax_node();
                    let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                    syntax::ast::Ref::cast(expr.syntax().clone()).unwrap()
                })
            });
        use_analysis_host_ctx.expect()
            .returning(|| {
            let analysis_host = AnalysisHost::new_single_file("");
            Signal::new(analysis_host)
            });
        get_bindings_in_scope_ctx.expect()
            .returning(|_, _| Some(vec!["foo".to_string(), "bar".to_string()]));

        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file("foo").syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));

             rsx! { RefInput { ptr: ptr_signal } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);

    }
}
