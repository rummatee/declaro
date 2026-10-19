pub mod attribute_set;
pub mod string_input;
pub mod ref_input;
pub mod lambda;
pub mod let_in;
pub mod expression;
use dioxus::prelude::*;
use rnix::{SyntaxNode, match_ast, ast};
use rowan::ast::AstNode;
use crate::router::Route;

use expression::components::ExpressionUI;

use crate::ast::functions::{collect_path, resolve_path};
use crate::ast::AstPath;

#[component]
pub fn NodeUI(path: ReadSignal<AstPath>) -> Element {
    let ast = use_context::<Signal<SyntaxNode>>();
    let node = use_memo(move || {
        let node = resolve_path(&ast.read(), &path.read()).unwrap();
        println!("NodeUI rendering node: {}", node);
        node
    });
    let level: u16 = 0;
    rsx! {
        Nav { path: path() }
        ExpressionUI { node: node, nesting_level: level }
    }
}


#[component]
pub fn Nav(path: AstPath) -> Element {
    let leaf = use_context::<Signal<SyntaxNode>>();
    let nodes = collect_path(leaf.read().clone(), &path);
    let elements = nodes.iter().filter_map(|index_node|{
        let node = &index_node.node;
        match_ast! {
            match node {
                ast::AttrpathValue(attr) => {
                    let mut index = index_node.index.clone();
                    index.indices.push(1); // The second child of an AttrpathValue is the value,
                                        // which is what we want to link to.
                    let label = attr.attrpath()
                        .map(|ap| ap.syntax().text().to_string())
                        .unwrap_or("unknown".to_string());
                    Some((label,index))
                },
                ast::Root(_sf) => {
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
