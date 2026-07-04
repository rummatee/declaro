use syntax::{SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;

#[double]
use crate::ast::functions as ast_functions;
#[double]
use crate::ast::hooks as ast_hooks;
#[double]
use crate::utils::hooks;

#[cfg(test)]
use mockall::automock;

#[derive(Props, PartialEq, Clone)]
pub struct RefInputProps {
    ptr: ReadSignal<SyntaxNodePtr>,
}

#[cfg_attr(test, automock)]
pub mod components { 

    use super::*;

    pub fn RefInput(props: RefInputProps) -> Element {
        let ptr = props.ptr;
        let analysis = hooks::use_analysis_host();
        let node = ast_hooks::use_ast_node::<syntax::ast::Ref>(ptr);
        let selected = node.read().token().unwrap();

        let bindings = ast_functions::get_bindings_in_scope(node.read().syntax(), &analysis.read()).unwrap_or_default();
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
                class: "ref-input simple-input",
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
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use serial_test::serial;
    use super::*;
    use crate::ast::mock_hooks::use_ast_node_context;
    use ide::{AnalysisHost, FileId};

    #[test]
    #[serial]
    fn test_ref_input() {
        let use_ast_node_ctx = use_ast_node_context();
        let use_analysis_host_ctx = crate::utils::mock_hooks::use_analysis_host_context();
        let get_bindings_in_scope_ctx = crate::ast::mock_functions::get_bindings_in_scope_context();
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

             rsx! { components::RefInput { ptr: ptr_signal } }
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
        let get_bindings_in_scope_ctx = crate::ast::mock_functions::get_bindings_in_scope_context();
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
            .returning(|_, _| Err(crate::ast::BindingsRetrievalError::GetExprIdError));
        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file("foo").syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));
             rsx! { components::RefInput { ptr: ptr_signal } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}
