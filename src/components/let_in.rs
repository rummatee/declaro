use syntax::ast::{HasBindings};
use syntax::SyntaxNodePtr;
use syntax::ast::AstNode;
use dioxus::prelude::*;
use mockall_double::double;
use focusable_macro::focusable;
use crate::ast::functions::add_binding;
use crate::components::expression::components::ExpressionUI;
use std::iter::zip;

#[double]
use crate::components::expression::components as expression_components;

#[double]
use crate::ast::hooks as ast_hooks;
#[double]
use crate::utils::hooks;

#[component]
pub fn LetInUI(ptr: ReadSignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let expression = ast_hooks::use_ast_node::<syntax::ast::LetIn>(ptr);
    let analysis = hooks::use_analysis_host();
    let bindings = expression.read().bindings();
    let body_pointer = SyntaxNodePtr::new(expression.read().body().unwrap().syntax());
    let enumerated = bindings.clone().enumerate();
    let focus = use_signal::<Option<i8>>(|| None);
    let labels = focusable!({
        iterator = enumerated,
        focus = focus,
        arms = [
        {
            matcher = syntax::ast::Binding::AttrpathValue(attr),
            focused = {
                element_type = input,
                preparation = {
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                },
                content = {
                    class: "attribute-label",
                    value: "{label.trim()}",
                    oninput: move |evt| {
                        let snapshot = analysis.read().0.snapshot();
                        let fpos = ide::FilePos { file_id: analysis.read().1, pos: attr.attrpath().unwrap().syntax().text_range().start()};
                        let result = snapshot.rename(fpos, evt.value().as_ref());
                        println!("Rename result: {:?}", result);
                        result.unwrap().unwrap().content_edits.get(&analysis.read().1).unwrap().iter().for_each(|edit| {
                            edit.apply(&mut use_context::<Signal<String>>().write());
                        });
                    },
                    onfocusout: move |_| {
                        focus.set(None);
                    }
                }
            },
            blurred = {
                element_type = label,
                preparation = {
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                    },
                    content = {
                        class: "attribute-label",
                        "{label}"
                    }
            }
        }
        ]
    });
    let elements = zip(labels,bindings.clone()).map(|(label, binding)| {
        let attr = match binding {
            syntax::ast::Binding::AttrpathValue(attr) => attr,
            _ => return rsx! { div { "Unsupported binding type" } },
        };
        let value = attr.value().unwrap();
        let node = value.syntax();
        let ptr = SyntaxNodePtr::new(node);
        rsx! {
            div {
                class: "attribute-item",
                {label}
                expression_components::ExpressionUI { ptr: ptr, nesting_level: nesting_level }
            }
        }
    });
    rsx! {
        div {
            class: "let-in",
            h3 {
                class: "binding-set-header let-header",
                "let"
            }
            div {
                class: "let-in-bindings binding-set",
                { elements }
                span { 
                    class: "add-binding",
                    onclick: move |_| {
                        add_binding(bindings.clone());
                    },
                    "+"
                }
            }
            h3 {
                class: "binding-set-header in-header",
                "in"
            }
            ExpressionUI{ ptr: body_pointer, nesting_level: nesting_level}
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
    use ide::AnalysisHost;

    #[test]
    #[serial]
    fn test_let_in_ui() {
        let use_ast_node_ctx = use_ast_node_context();
        let expression_ui_ctx = ExpressionUI_context();
        let use_analysis_host_ctx = crate::utils::mock_hooks::use_analysis_host_context();
        const SOURCE: &str = r#"
        let a = 1; b = 2; in { a + b }
        "#;
        use_ast_node_ctx.expect()
            .returning(|_| {
                Memo::new(|| {
                    let syntax_node = syntax::parse_file(SOURCE).syntax_node();
                    let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                    syntax::ast::LetIn::cast(expr.syntax().clone()).unwrap()
                })
            });
        use_analysis_host_ctx.expect()
            .returning(|| {
            let analysis_host = AnalysisHost::new_single_file(SOURCE);
            Signal::new(analysis_host)
            });
        expression_ui_ctx.expect()
            .returning(|props| {
                rsx! { div { "ExpressionUI for props: {props:?}" } }
            });
        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file(SOURCE).syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));
            rsx! { LetInUI { ptr: ptr_signal, nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        use_ast_node_ctx.checkpoint();
        expression_ui_ctx.checkpoint();
        assert_snapshot!(html);
    }
}
