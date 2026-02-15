pub mod attribute_set;
pub mod string_input;
pub mod ref_input;
pub mod lambda;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaGear;
use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use crate::router::Route;

pub use attribute_set::*;
pub use lambda::*;
use string_input::StringInput;
use ref_input::RefInput;

use crate::ast::{collect_path, resolve_path, AstPath, update_node_value, path_from_root};
use closure::closure;

#[component]
pub fn NodeUI(path: ReadSignal<AstPath>) -> Element {
    let ast = use_context::<Signal<SyntaxNode>>();
    let ptr = use_memo(move || {
        let node = resolve_path(&ast.read(), &path.read()).unwrap();
        println!("NodeUI rendering node: {}", node.to_string());
        SyntaxNodePtr::new(&node)
    });
    let level: u16 = 0;
    rsx! {
        Nav { path: path() }
        ExpressionUI { ptr: ptr, nesting_level: level }
    }
}

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

#[component]
pub fn ExpressionUI(ptr: ReadSignal<SyntaxNodePtr>, nesting_level: u16) -> Element {
    let ast = use_context::<Signal<SyntaxNode>>();
    let mut menu_open = use_signal(|| false);
    let node = ptr.read().to_node(&ast.read());
    let node_ref = node.clone();
    let next_level = nesting_level + 1;
    let body = match_ast! {
        match node_ref {
            syntax::ast::AttrSet(_) => {
                link_or_element(&node, nesting_level, rsx! {  AttributeSetUI { ptr:ptr, nesting_level: next_level }  })
            },
            syntax::ast::Lambda(_) => {
                link_or_element(&node, nesting_level, rsx! { LambdaUI { ptr:ptr, nesting_level: next_level }  })
            },
            syntax::ast::String(_) => rsx! { StringInput { ptr:ptr } },
            syntax::ast::Ref(_) => rsx! { RefInput { ptr:ptr } },
            _ => rsx! {},
        }
    };
    let extra_classes = match_ast! {
        match node_ref {
            syntax::ast::AttrSet(_) => if decide_link_or_element(&node, nesting_level) {"atom"} else {"composed"},
            syntax::ast::Lambda(_) => if decide_link_or_element(&node, nesting_level) {"atom"} else {"composed"},
            syntax::ast::String(_) => "atom",
            syntax::ast::Ref(_) => "atom",
            _ => "",
        }
    };
    let menu_items = vec![
        ("Attribute Set", "{}") ,
        ("Lambda", "{}:{}"),
        ("String", "\"\""),
        ("Reference", "ref"),
    ];
    let menu_elements = menu_items.into_iter().map(|(label, template)| {
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
    });
    rsx! {
        div {
            class: "expression-ui ".to_owned() + extra_classes,
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
pub fn Nav(path: AstPath) -> Element {
    let ast = use_context::<Signal<SyntaxNode>>();
    let nodes = collect_path(ast.read().clone(), &path);
    println!("Nav nodes: {:?}", nodes.iter().map(|n| n.node.kind().to_string()).collect::<Vec<String>>());
    let elements = nodes.iter().filter_map(|index_node|{
        let node = &index_node.node;
        match_ast! {
            match node {
                syntax::ast::AttrpathValue(attr) => {
                    let mut index = index_node.index.clone();
                    index.indices.push(1); // The second child of an AttrpathValue is the value,
                                        // which is what we want to link to.
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                    Some((label,index))
                },
                syntax::ast::SourceFile(_sf) => {
                    let index = index_node.index.clone();
                    let label = "root".to_string();
                    Some((label,index))
                },
                _ => None,
            }
        }
    }).map(|link| {
        rsx! {
            Link {
                to: Route::NodeUI{ path: link.1},
                {link.0}
            }
        }
    });
    rsx! {
        nav { {elements} }
    }
}
