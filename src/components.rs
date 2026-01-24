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

use crate::ast::{collect_path, resolve_path, AstPath, update_node_value};
use closure::closure;

#[derive(Clone)]
struct NavContext {
    current_node: ReadOnlySignal<AstPath>,
}

#[component]
pub fn NodeUI(path: ReadOnlySignal<AstPath>) -> Element {
    use_context_provider(|| NavContext{ current_node: path.clone() });
    let ast = use_context::<Signal<SyntaxNode>>();
    let ptr = use_memo(move || {
        let node = resolve_path(&ast.read(), &path.read()).unwrap();
        println!("NodeUI rendering node: {}", node.to_string());
        SyntaxNodePtr::new(&node)
    });
    rsx! {
        Nav { path: path() }
        ExpressionUI { ptr: ptr }
    }
}

#[component]
pub fn ExpressionUI(ptr: ReadOnlySignal<SyntaxNodePtr>) -> Element {
    let ast = use_context::<Signal<SyntaxNode>>();
    let mut menu_open = use_signal(|| false);
    let node = ptr.read().to_node(&ast.read());
    let node_ref = node.clone();
    let body = match_ast! {
        match node_ref {
            syntax::ast::AttrSet(_) => rsx! {  AttributeSetUI { ptr:ptr }  },
            syntax::ast::Lambda(_) => rsx! { LambdaUI { ptr:ptr }  },
            syntax::ast::String(_) => rsx! { StringInput { ptr:ptr } },
            syntax::ast::Ref(_) => rsx! { RefInput { ptr:ptr } },
            _ => rsx! {},
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
            class: "expression-ui",
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
