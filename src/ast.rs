use syntax::{SyntaxNode};
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstPath {
    pub indices: Vec<usize>,
}

pub fn path_from_root(node: &SyntaxNode) -> AstPath {
    let mut indices = Vec::new();
    let mut current = node.clone();

    while let Some(parent) = current.parent() {
        let index = parent
            .children()
            .position(|child| child == current)
            .expect("node must be child of its parent");

        indices.push(index);
        current = parent;
    }

    indices.reverse();

    AstPath { indices }
}

pub fn resolve_path(root: &SyntaxNode, path: &AstPath) -> Option<SyntaxNode> {
    let mut current = root.clone();

    for &index in &path.indices {
        current = current.children().nth(index)?;
    }

    Some(current)
}

pub struct IndexedNode {
    pub index: AstPath,
    pub node: SyntaxNode,
}

pub fn collect_path(root: SyntaxNode, path: &AstPath) -> Vec<IndexedNode> {
    let mut nodes = Vec::new();
    let mut current = root;
    let mut current_path = AstPath { indices: Vec::new() };
    let index_node = IndexedNode {
        index: AstPath { indices: vec![0] },
        node: current.clone(),
    };
    nodes.push(index_node);
    for &index in &path.indices {
        if let Some(child) = current.children().nth(index) {
            current_path.indices.push(index);
            let index_node = IndexedNode {
                index: current_path.clone(),
                node: child.clone(),
            };
            nodes.push(index_node);
            current = child;
        } else {
            break;
        }
    }
    nodes
}

impl std::fmt::Display for AstPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, idx) in self.indices.iter().enumerate() {
            if i > 0 {
                write!(f, ".")?;
            }
            write!(f, "{idx}")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAstPathError;

impl std::fmt::Display for ParseAstPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse AstPath")
    }
}

impl std::str::FromStr for AstPath {
    type Err = ParseAstPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let indices = if s.is_empty() {
            Ok(Vec::new())
        } else {
            s.split('.')
                .map(|p| p.parse::<usize>().map_err(|_| ()))
                .collect::<Result<Vec<_>, _>>().map_err(|_| {ParseAstPathError})
        };

        indices.map(|indices| AstPath { indices } )
    }
}

#[macro_export]
macro_rules! use_ast_node_strict {
    (
        $ptr:expr => $ty:ty
    ) => {{
        use dioxus::prelude::*;
        use rowan::ast::AstNode;

        let ast = use_context::<Signal<SyntaxNode>>();
        use_memo(move || {
            let syntax = $ptr.read().to_node(&ast.read());

            <$ty as AstNode>::cast(syntax)
                .expect("AST node cast failed")
        })
    }};
}

pub fn update_node_value<F>(
    node: SyntaxNode,
    new_value: &str,
    extract_new_node: F,
)
where
    F: Fn(&SyntaxNode) -> Option<SyntaxNode>,
{
    let mut ast = use_context::<Signal<SyntaxNode>>();
    let new_syntax = syntax::parse_file(new_value).syntax_node();
    if let Some(new_syntax) = extract_new_node(&new_syntax) {
        let new_root = SyntaxNode::new_root(
            replace_expr(&node, new_syntax)
        );
        ast.set(new_root);
    }
}

fn replace_expr(
    old: &SyntaxNode,
    new: SyntaxNode,
) -> rowan::GreenNode {
    let parent = old.parent().expect("expr must have parent");

    let mut children: Vec<rowan::NodeOrToken<rowan::GreenNode,rowan::GreenToken>> =
        parent.green().children().map(|c| {
            c.to_owned()
        }).collect();

    let idx = parent
        .children_with_tokens()
        .position(|c| c.as_node().is_some() && c.as_node().unwrap() == old)
        .unwrap();

    children.iter().for_each(|c| println!("Child: {:?}", c.kind()));
    children[idx] = rowan::NodeOrToken::Node(new.green().into_owned());
    println!("Replaced child at index {}, with {:?}", idx, new);


    let new_parent = rowan::GreenNode::new(rowan::SyntaxKind(parent.kind() as u16), children);

    parent.replace_with(new_parent)
}

