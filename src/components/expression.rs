use std::iter::once;
use syntax::ast::{HasStringParts};
use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaGear;
use closure::closure;

use crate::{use_ast_node_strict};
use crate::ast::{update_node_value, path_from_root};

use crate::components::attribute_set::AttributeSetUI;
use crate::components::string_input::StringInput;
use crate::components::ref_input::RefInput;
use crate::components::lambda::LambdaUI;


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
        _ => false,
        }
    }
}

#[component]
pub fn ExpressionUI(ptr: ReadSignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let ast = use_context::<Signal<SyntaxNode>>();
    let mut menu_open = use_signal(|| false);
    let node = ptr.read().to_node(&ast.read());
    let mut fallback_ui = use_signal(|| {
        !can_use_non_fallback_ui(&node)
    });
    let node_ref = node.clone();
    let next_level = nesting_level + 1;
    let body = if fallback_ui() {
        rsx! { FallbackExpressionUI { ptr:ptr }  }
    } else { match_ast! {
        match node_ref {
            syntax::ast::AttrSet(_) => {
                link_or_element(&node, nesting_level, rsx! {  AttributeSetUI { ptr:ptr, nesting_level: next_level }  })
            },
            syntax::ast::Lambda(_) => {
                link_or_element(&node, nesting_level, rsx! { LambdaUI { ptr:ptr, nesting_level: next_level }  })
            },
            syntax::ast::String(_) => rsx! { StringInput { ptr:ptr } },
            syntax::ast::Ref(_) => rsx! { RefInput { ptr:ptr } },
            _ => rsx! { FallbackExpressionUI { ptr:ptr }  },
        }
    }};
    let extra_classes = match_ast! {
        match node_ref {
            syntax::ast::AttrSet(_) => if decide_link_or_element(&node, nesting_level) {"atom"} else {"composed"},
            syntax::ast::Lambda(_) => if decide_link_or_element(&node, nesting_level) {"atom"} else {"composed"},
            syntax::ast::String(_) => "atom",
            syntax::ast::Ref(_) => "atom",
            _ => "atom",
        }
    };
    let menu_items = vec![
        ("Attribute Set", "{}") ,
        ("Lambda", "{}:{}"),
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
                                node.clone(),
                                template,
                                |syntax| {
                                    <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                                        .and_then(|sf| sf.expr())
                                        .map(|expr| expr.syntax().clone())
                                }
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

#[component]
pub fn FallbackExpressionUI(ptr: ReadSignal<SyntaxNodePtr>) -> Element {
    let node = use_ast_node_strict!(ptr => syntax::ast::Expr);
    let value = node.read().syntax().text().to_string();
    rsx! {
        textarea {
            class: "fallback-expression-input simple-input",
            value: value,
            oninput: move |e| {
                println!("New value: {}", e.value());
                update_node_value(
                    node.read().syntax().clone(),
                    &e.value(),
                    |syntax| {
                        <syntax::ast::SourceFile as AstNode>::cast(syntax.clone())
                            .and_then(|sf| sf.expr())
                            .map(|expr| expr.syntax().clone())
                    }
                );
            }
        }
    }
}
