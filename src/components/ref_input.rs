use ide::{AnalysisHost, FileId};
use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use thiserror::Error;

#[double]
use util_functions as utils;
#[double]
use crate::ast::functions as ast_functions;
#[double]
use crate::utils::hooks;

#[cfg(test)]
use mockall::automock;



#[component]
pub fn RefInput(ptr: ReadSignal<SyntaxNodePtr>) -> Element {
    let analysis = hooks::use_analysis_host();
    let node = ast_functions::use_ast_node::<syntax::ast::Ref>(ptr);
    let selected = node.read().token().unwrap();

    let bindings = utils::get_bindings_in_scope(node.read().syntax(), &analysis.read())?;

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

                ast_functions::update_node_value(
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

#[derive(Error, Debug)]
pub enum BindingsRetrievalError {
    #[error("Failed getting scopes")]
    GetScopesError(#[from] ide::Cancelled),
    #[error("Failed getting expression id")]
    GetExprIdError,
    #[error("Failed getting scope for expression")]
    GetScopeForExprError,
}

#[cfg_attr(test, automock)]
mod util_functions {
    use super::*;
    pub fn get_bindings_in_scope(node: &SyntaxNode, analysis: &(AnalysisHost, FileId)) -> Result<Vec<String>, BindingsRetrievalError> {
        let snapshot = analysis.0.snapshot();
        let scopes = snapshot.scopes(analysis.1)?;
        let expr_id = snapshot
            .source_map(analysis.1).unwrap()
            .expr_for_node(SyntaxNodePtr::new(node)).ok_or(BindingsRetrievalError::GetExprIdError)?;
        let scope_id = scopes.scope_for_expr(expr_id).ok_or(BindingsRetrievalError::GetScopeForExprError)?;
        Ok(scopes
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
    use serial_test::serial;
    use super::*;
    use crate::ast::mock_functions::use_ast_node_context;

    #[test]
    #[serial]
    fn test_ref_input() {
        let use_ast_node_ctx = use_ast_node_context();
        let use_analysis_host_ctx = crate::utils::mock_hooks::use_analysis_host_context();
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
            .returning(|_, _| Ok(vec!["foo".to_string(), "bar".to_string()]));

        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file("foo").syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));

             rsx! { RefInput { ptr: ptr_signal } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);

    }

    #[test]
    #[serial]
    fn test_ref_input_get_binding_error() {
        let use_ast_node_ctx = use_ast_node_context();
        let use_analysis_host_ctx = crate::utils::mock_hooks::use_analysis_host_context();
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
            .returning(|_, _| Err(BindingsRetrievalError::GetExprIdError));
        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file("foo").syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));
             rsx! { RefInput { ptr: ptr_signal } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_eq!(html, "");
    }

    #[test]
    #[serial]
    fn test_get_bindings_in_scope() {
        let analysis_host = AnalysisHost::new_single_file("let foo = 1; in let bar = 2; in foo + bar").0;
        let syntax_node = syntax::parse_file("let foo = 1; in let bar = 2; in foo + bar").syntax_node();
        let path = crate::ast::AstPath{indices: vec![0, 1, 1, 0]}; // Path to the 'foo' reference in 'foo + bar'
        let expr = crate::ast::functions::resolve_path(&syntax_node, &path).unwrap();
        let ref_node = syntax::ast::Ref::cast(expr.clone()).unwrap();
        let bindings = util_functions::get_bindings_in_scope(ref_node.syntax(), &(analysis_host, FileId(0))).unwrap();
        assert_eq!(bindings, vec!["bar".to_string(), "foo".to_string()]);
    }
}
