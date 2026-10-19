use std::iter::once;
use rnix::{SyntaxNode, SyntaxKind, match_ast, ast};
use rowan::ast::AstNode;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaGear;
use closure::closure;
use mockall_double::double;

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

    match node.kind() {
        SyntaxKind::NODE_ATTR_SET => true,
        SyntaxKind::NODE_LAMBDA => true,
        SyntaxKind::NODE_STRING => true,
        SyntaxKind::NODE_IDENT => true,
        SyntaxKind::NODE_LET_IN => true,
        _ => false,
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct ExpressionUIProps {
    node: ReadSignal<SyntaxNode>,
    nesting_level: u16,
}

impl std::fmt::Debug for ExpressionUIProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpressionUIProps")
            .field("node", &self.node.read())
            .field("nesting_level", &self.nesting_level)
            .finish()
    }
}

#[cfg_attr(test, automock)]
pub mod components {

    use super::*;

    #[allow(non_snake_case)]
    pub fn ExpressionUI(props: ExpressionUIProps) -> Element {
        let nesting_level = props.nesting_level;
        let mut menu_open = use_signal(|| false);
        let node = props.clone().node;
        let mut fallback_ui = use_signal(|| {
            !can_use_non_fallback_ui(&node.read())
        });
        let node_ref = node.read().clone();
        let next_level = nesting_level + 1;
        let body = if fallback_ui() {
            rsx! { FallbackExpressionUI { ..props }  }
        } else { match_ast! {
            match node_ref {
                ast::AttrSet(set) => {
                    link_or_element(&node.read(), nesting_level, rsx! {  AttributeSetUI { set, nesting_level: next_level }  })
                },
                ast::Lambda(lambda) => {
                    link_or_element(&node.read(), nesting_level, rsx! { LambdaUI { lambda, nesting_level: next_level }  })
                },
                ast::LetIn(expression) => {
                    link_or_element(&node.read(), nesting_level, rsx! { LetInUI { expression, nesting_level: next_level }  })
                },
                ast::Str(string) => rsx! { string_input_components::StringInput { string } },
                ast::Ident(ident) => rsx! { ref_input_components::RefInput { ident } },
                _ => rsx! { FallbackExpressionUI { ..props }  },
            }
        }};
        let extra_classes = match_ast! {
            match node_ref {
                ast::AttrSet(_) => if decide_link_or_element(&node.read(), nesting_level) {"atom"} else {"composed"},
                ast::Lambda(_) => if decide_link_or_element(&node.read(), nesting_level) {"atom"} else {"composed"},
                ast::LetIn(_) => if decide_link_or_element(&node.read(), nesting_level) {"atom"} else {"composed"},
                ast::Str(_) => "atom",
                ast::Ident(_) => "atom",
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
            if can_use_non_fallback_ui(&node.read()) {
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
                                    node.read().clone().into(),
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
        let node = props.node;
        let value = node.read().text().to_string();
        rsx! {
            textarea {
                class: "fallback-expression-input simple-input",
                value: value,
                oninput: move |e| {
                    println!("New value: {}", e.value());
                    update_node_value(
                        node.read().clone().into(),
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
    use serial_test::serial;

    macro_rules! expression_ui_tests {
        ($(($name:ident, $source:expr, $ty:ty)),* $(,)?) => {
            $(
                #[test]
                #[serial]
                fn $name() {
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
                        let syntax_node = rnix::Root::parse($source).tree();
                        let expr = syntax_node.expr().unwrap();
                        rsx! { components::ExpressionUI { node: expr.syntax().clone(), nesting_level: 1 } }
                    });
                    vdom.rebuild_in_place();
                    let html = dioxus_ssr::render(&vdom);
                    expression_ui_ctx.checkpoint();
                    ref_input_ctx.checkpoint();
                    string_input_ctx.checkpoint();
                    insta::assert_snapshot!(stringify!($name), html);
                }
            )*
        }
    }

    expression_ui_tests! {
        (test_expression_ui_attrset, "{ a = 1; b = 2; }", ast::AttrSet),
        (test_expression_ui_lambda, "{ var1, var2 ? \"default\" } : {}", ast::Lambda),
        (test_expression_ui_let_in, "let a = 1 in {}", ast::LetIn),
        (test_expression_ui_reference, "foo", ast::Ident),
        (test_expression_ui_string, "\"a string\"", ast::Str),
    }

    #[test]
    #[serial]
    fn test_fallback_expression_ui() {
        const SOURCE: &str = r#"
        { var1, var2 ? "default" } : {}
        "#;
        let mut vdom = VirtualDom::new(|| {
            let root = rnix::Root::parse(SOURCE).tree();
             rsx! { components::FallbackExpressionUI { node: root.expr().unwrap().syntax().clone(), nesting_level: 1 } }
        });
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert_snapshot!(html);
    }
}
