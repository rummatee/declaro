use syntax::{match_ast, SyntaxNode, SyntaxNodePtr};
use syntax::ast::AstNode;
use dioxus::prelude::*;

#[macro_export]
macro_rules! use_ast_node_strict {
    (
        $ptr:expr => $ty:ty
    ) => {{
        use dioxus::prelude::*;
        use rowan::ast::AstNode;

        let ast = use_context::<Signal<SyntaxNode>>();
        use_memo(move || {
            let syntax = $ptr.to_node(&ast.read());

            <$ty as AstNode>::cast(syntax)
                .expect("AST node cast failed")
        })
    }};
}

pub fn update_node_value<N, F>(
    node: N,
    new_value: &str,
    extract_new_node: F,
)
where
    N: AstNode,
    F: Fn(&SyntaxNode) -> Option<SyntaxNode>,
{
    let mut ast = use_context::<Signal<SyntaxNode>>();
    let new_syntax = syntax::parse_file(new_value).syntax_node();
    if let Some(new_node) = extract_new_node(&new_syntax) {
        let green_node = new_node.green();
        let new_root = SyntaxNode::new_root(
            node.syntax().replace_with((*green_node).to_owned())
        );
        ast.set(new_root);
    }
}
