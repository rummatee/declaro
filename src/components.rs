pub mod attributeSet;
pub mod stringInput;
use dioxus::prelude::*;
use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use crate::router::Route;

pub use attributeSet::*;
pub use stringInput::*;

use crate::ast::{collect_path, resolve_path, AstPath};

#[derive(Clone)]
struct NavContext {
    current_node: ReadOnlySignal<AstPath>,
}

#[component]
pub fn NodeUI(path: ReadOnlySignal<AstPath>) -> Element {
    use_context_provider(|| NavContext{ current_node: path.clone() });
    let ast = use_context::<Signal<SyntaxNode>>();
    let node = resolve_path(&ast.read(), &path.read()).unwrap();
    println!("NodeUI rendering node: {}", node.to_string());
    let ptr = SyntaxNodePtr::new(&node);
    let body = match_ast! {
        match node {
        syntax::ast::AttrSet(_) => rsx! {  AttributeSetUI { ptr:ptr }  },
        _ => rsx! {},
        }
    };
    rsx! {
        Nav { path: path() }
        { body }
    }
}

#[component]
pub fn Nav(path: AstPath) -> Element {
    let ast = use_context::<Signal<SyntaxNode>>();
    let nodes = collect_path(ast.read().clone(), &path);
    println!("Nav nodes: {:?}", nodes.iter().map(|n| n.node.to_string()).collect::<Vec<String>>());
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
                syntax::ast::Expr(_sf) => {
                    let mut index = index_node.index.clone();
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
