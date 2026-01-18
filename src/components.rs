pub mod attributeSet;
pub mod stringInput;
pub mod refInput;
pub mod lambda;
use dioxus::prelude::*;
use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use crate::router::Route;

pub use attributeSet::*;
pub use stringInput::*;
pub use lambda::*;

use crate::ast::{collect_path, resolve_path, AstPath};

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
    let body = use_memo(move || {
        let node = ptr.read().to_node(&ast.read());
        match_ast! {
            match node {
            syntax::ast::AttrSet(_) => rsx! {  AttributeSetUI { ptr:ptr }  },
            syntax::ast::Lambda(_) => rsx! { LambdaUI { ptr:ptr }  },
            _ => rsx! {},
            }
        }
    });
    rsx! {
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
                syntax::ast::SourceFile(_sf) => {
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
