use std::iter::once;
use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaGear;
use closure::closure;
use mockall_double::double;

#[double]
use crate::ast::hooks as ast_hooks;

use crate::ast::functions::{update_node_value, path_from_root};

use crate::components::attribute_set::AttributeSetUI;
use crate::components::lambda::LambdaUI;
use crate::components::let_in::LetInUI;

#[double]
use crate::components::ref_input::components as ref_input_components;

#[double]
use crate::components::string_input::components as string_input_components;

#[cfg(test)]
use mockall::automock;

fn decide_link_or_element(_node: &SyntaxNode, nesting_level: u16) -> bool {
    nesting_level > 1
}

fn link_or_element(node: &SyntaxNode, nesting_level: u16, element: Element) -> Element {
    if decide_link_or_element(node, nesting_level) {
        rsx! {
                Link {
                    class: "subpage-link",
                    to: crate::router::Route::NodeUI{ path: path_from_root(node)},
                    "Link"
                }
            }
        } else {
            element
    }
}

fn can_use_non_fallback_ui(node: &SyntaxNode) -> bool {
    match_ast! {
        match node {
        syntax::ast::AttrSet(_) => true,
        syntax::ast::Lambda(_) => true,
        syntax::ast::String(_) => true,
        syntax::ast::Ref(_) => true,
        syntax::ast::LetIn(_) => true,
        _ => false,
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ExpressionUIProps {
    ptr: ReadSignal<SyntaxNodePtr>,
    nesting_level: u16,
}

impl std::fmt::Debug for ExpressionUIProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpressionUIProps")
            .field("ptr", &self.ptr.read())
            .field("nesting_level", &self.nesting_level)
            .finish()
    }
}

#[cfg_attr(test, automock)]
pub mod components {

    use super::*;

    #[allow(non_snake_case)]
    pub fn ExpressionUI(props: ExpressionUIProps) -> Element {
        let ptr = props.ptr;
        let nesting_level = props.nesting_level;
        let ast = ast_hooks::use_syntax_node();
        let mut menu_open = use_signal(|| false);
        let node = ptr.read().to_node(&ast.read());
        let mut fallback_ui = use_signal(|| {
            !can_use_non_fallback_ui(&node)
        });
        let node_ref = node.clone();
        let next_level = nesting_level + 1;
        let body = if fallback_ui() {
            rsx! { FallbackExpressionUI { ..props }  }
        } else { match_ast! {
            match node_ref {
                syntax::ast::AttrSet(_) => {
                    link_or_element(&node, nesting_level, rsx! {  AttributeSetUI { ptr:ptr, nesting_level: next_level }  })
                },
                syntax::ast::Lambda(_) => {
                    link_or_element(&node, nesting_level, rsx! { LambdaUI { ptr:ptr, nesting_level: next_level }  })
                },
                syntax::ast::LetIn(_) => {
                    link_or_element(&node, nesting_level, rsx! { LetInUI { ptr:ptr, nesting_level: next_level }  })
                },
                syntax::ast::String(_) => rsx! { string_input_components::StringInput { ptr:ptr } },
                syntax::ast::Ref(_) => rsx! { ref_input_components::RefInput { ptr:ptr } },
                _ => rsx! { FallbackExpressionUI { ..props }  },
            }
        }};
        let extra_classes = match_ast! {
            match node_ref {
                syntax::ast::AttrSet(_) => if decide_link_or_element(&node, nesting_level) {"atom"} else {"composed"},
                syntax::ast::Lambda(_) => if decide_link_or_element(&node, nesting_level) {"atom"} else {"composed"},
                syntax::ast::LetIn(_) => if decide_link_or_element(&node, nesting_level) {"atom"} else {"composed"},
                syntax::ast::String(_) => "atom",
                syntax::ast::Ref(_) => "atom",
                _ => "atom",
            }
        };
        let menu_items = vec![
            ("Attribute Set", "{}") ,
            ("Lambda", "{}:{}"),
            ("Let In", "let a = 1; in {}"),
            ("String", "\"\""),
            ("Reference", "ref"),
        ];
        let menu_elements: Box<dyn Iterator<Item = Element>> = if fallback_ui() {
            if can_use_non_fallback_ui(&node) {
                Box::new(once(rsx! {
                    li {
                        onclick: move |_| {
                            fallback_ui.set(false);
                            menu_open.set(false);
                        },
                        "Use specific editor"
                    }
                }))
            } else {
                Box::new(std::iter::empty())
            }
        } else {
            Box::new(
                menu_items
                .into_iter()
                .map(|(label, template)| {
                    rsx! {
                        li { 
                            onclick: closure!(move mut menu_open, clone node, |_| {
                                menu_open.set(false);
                                update_node_value(
                                    node.clone().into(),
                                    template
                                );
                            }),
                            "{label}"
                        }
                    }
                })
            .chain(once(rsx! {
                li {
                    onclick: move |_| {
                        fallback_ui.set(true);
                        menu_open.set(false);
                    },
                    "Fallback Editor"
                }
            }))
            )
        };
        
        rsx! {
            div {
                class: "expression-ui ".to_owned() + extra_classes,
                if menu_elements.size_hint().0 > 0 {
                    div {
                        onclick: move |_| {
                            menu_open.set(!menu_open());
                        },
                        Icon {
                            class: "change-expression-type",
                            icon: FaGear,
                            width: 14,
                            height: 14,
                        }
                    }
                }
                if menu_open() {
                    ul {
                        class: "expression-type-menu",
                        { menu_elements }
                    }
                }
                { body }
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn FallbackExpressionUI(props: ExpressionUIProps) -> Element {
        let ptr = props.ptr;
        let node = ast_hooks::use_ast_node::<syntax::ast::Expr>(ptr);
        let value = node.read().syntax().text().to_string();
        rsx! {
            textarea {
                class: "fallback-expression-input simple-input",
                value: value,
                oninput: move |e| {
                    println!("New value: {}", e.value());
                    update_node_value(
                        node.read().syntax().clone().into(),
                        &e.value()
                    );
                }
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use super::*;
    use crate::ast::mock_hooks::{use_ast_node_context, use_syntax_node_context};
    use serial_test::serial;
    use ide::AnalysisHost;

    macro_rules! expression_ui_tests {
        ($(($name:ident, $source:expr, $ty:ty)),* $(,)?) => {
            $(
                #[test]
                #[serial]
                fn $name() {
                    let use_syntax_node_ctx = use_syntax_node_context();
                    use_syntax_node_ctx.expect()
                        .returning(|| {
                            Signal::new(syntax::parse_file($source).syntax_node())
                        });
                    let use_ast_node_ctx = use_ast_node_context();
                    use_ast_node_ctx.expect::<$ty>()
                        .returning(|_| {
                            Memo::new(|| {
                                let syntax_node = syntax::parse_file($source).syntax_node();
                                let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                                <$ty>::cast(expr.syntax().clone()).unwrap()
                            })
                        });
                    let use_analysis_host_ctx = crate::utils::mock_hooks::use_analysis_host_context();
                    use_analysis_host_ctx.expect()
                        .returning(|| {
                        let analysis_host = AnalysisHost::new_single_file($source);
                        Signal::new(analysis_host)
                        });
                    let expression_ui_ctx = super::mock_components::ExpressionUI_context();
                    expression_ui_ctx.expect()
                        .returning(|props| {
                        rsx! { div { "ExpressionUI for props: {props:?}" } }
                        });

                    /* I'm mocking RefInput and StringInput, because otherwise, I would need to mock all it's
                     * dependencies, as mockall doesn't fall back to the real implementation.
                     * In the future, I might want to mock all inputs here for consistency and
                     * because it might be needed if the components have more dependencies, however
                     * this would require implementing a property struct for all of them, which at
                     * the moment adds more noise than needed. */
                    let ref_input_ctx = crate::components::ref_input::mock_components::RefInput_context();
                    ref_input_ctx.expect()
                        .returning(|_| {
                        rsx! { div { "RefInput" } }
                        });

                    let string_input_ctx = crate::components::string_input::mock_components::StringInput_context();
                    string_input_ctx.expect()
                        .returning(|_| {
                        rsx! { div { "StringInput" } }
                        });

                    let mut vdom = VirtualDom::new(|| {
                        let syntax_node = syntax::parse_file($source).syntax_node();
                        let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                        let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(expr.syntax()));
                        rsx! { components::ExpressionUI { ptr: ptr_signal, nesting_level: 1 } }
                    });
                    vdom.rebuild_in_place();
                    let html = dioxus_ssr::render(&vdom);
                    use_ast_node_ctx.checkpoint();
                    use_syntax_node_ctx.checkpoint();
                    expression_ui_ctx.checkpoint();
                    ref_input_ctx.checkpoint();
                    string_input_ctx.checkpoint();
                    insta::assert_snapshot!(stringify!($name), html);
                }
            )*
        }
    }

    expression_ui_tests! {
        (test_expression_ui_attrset, "{ a = 1; b = 2; }", syntax::ast::AttrSet),
        (test_expression_ui_lambda, "{ var1, var2 ? \"default\" } : {}", syntax::ast::Lambda),
        (test_expression_ui_let_in, "let a = 1 in {}", syntax::ast::LetIn),
        (test_expression_ui_reference, "foo", syntax::ast::Ref),
        (test_expression_ui_string, "\"a string\"", syntax::ast::String),
    }

    #[test]
    #[serial]
    fn test_fallback_expression_ui() {
        let use_ast_node_ctx = use_ast_node_context();
        const SOURCE: &str = r#"
        { var1, var2 ? "default" } : {}
        "#;
        use_ast_node_ctx.expect()
            .returning(|_| {
                Memo::new(|| {
                    let syntax_node = syntax::parse_file(SOURCE).syntax_node();
                    let expr = syntax::ast::SourceFile::cast(syntax_node).unwrap().expr().unwrap();
                    syntax::ast::Expr::cast(expr.syntax().clone()).unwrap()
                })
            });
        let mut vdom = VirtualDom::new(|| {
            let syntax_node = syntax::parse_file(SOURCE).syntax_node();
            let ptr_signal = Signal::new(syntax::SyntaxNodePtr::new(&syntax_node));
             rsx! { components::FallbackExpressionUI { ptr: ptr_signal, nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}
