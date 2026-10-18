use rnix::{SyntaxNode, SyntaxElement, ast::{LetIn, Pattern}};
use rowan::ast::{AstChildren, AstNode};
use dioxus::prelude::*;
use mockall_double::double;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstPath {
    pub indices: Vec<usize>,
}

#[cfg_attr(test, automock)]
pub mod hooks {
    use super::*;

    pub fn use_source() -> Signal<String> {
        use_context::<Signal<String>>()
    }

}

#[cfg_attr(test, automock)]
pub mod functions {
    use rnix::ast::HasEntry;

    use super::*;

    #[double]
    use super::hooks;

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

    pub fn update_node_value(node: SyntaxElement, new_value: &str)
    {
        let range = node.text_range();
        let mut source = hooks::use_source();
        source.write().replace_range(usize::from(range.start())..usize::from(range.end()), new_value);
    }

    pub fn add_binding<T>(nodes: AstChildren<T>)
    where
        T: rnix::ast::AstNode + PartialEq + 'static,
    {
        let mut source = hooks::use_source();
        source.write().insert_str(nodes.last().map(|node| usize::from(node.syntax().text_range().end())).unwrap_or(0), "\nnew_attr = 0;");
    }

    pub fn get_bindings_in_scope(node: &SyntaxNode) -> Result<Vec<String>, BindingsRetrievalError> {
        let mut result = Vec::new();
        get_bindings_rec(node, &mut result)?;
        Ok(result)
    }

    pub fn get_bindings_rec(node: &SyntaxNode, acc: &mut Vec<String>) -> Result<(), BindingsRetrievalError> {
        if let Some(let_in) = LetIn::cast(node.clone()) {
            for entry in let_in.attrpath_values() {
                if let Some(attrpath) = entry.attrpath() {
                    acc.push(attrpath.syntax().text().to_string());
                }
            }
        }
        if let Some(lambda) = rnix::ast::Lambda::cast(node.clone()) {
            if let Some(rnix::ast::Param::Pattern(pat)) = lambda.param() {
                for pat in pat.pat_entries() {
                    if let Some(ident) = pat.ident() {
                        acc.push(ident.syntax().text().to_string());
                    }
                }
            }
        }
        if let Some(parent) = node.parent() {
            get_bindings_rec(&parent, acc)?;
        }
        Ok(())
    }
}


#[derive(Error, Debug)]
pub enum BindingsRetrievalError {
    #[error("Failed getting expression id")]
    ExprId,
    #[error("Failed getting scope for expression")]
    ScopeForExpr,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAstPathError;

pub struct IndexedNode {
    pub index: AstPath,
    pub node: SyntaxNode,
}

impl std::fmt::Display for IndexedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexedNode {{ index: {}, node: {:?} }}", self.index, self.node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use std::str::FromStr;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_ast_path_roundtrip() {
        let code = r#"
            let x = 1;
            in {
                y = {
                    z = 2;
                };
            }
        "#;
        let root = rnix::Root::parse(code);
        let ast = root.syntax();
        let path = AstPath::from_str("0.1.0").expect("Failed to parse AstPath");
        let node = functions::resolve_path(&ast, &path).expect("Node should exist");
        assert_snapshot!(node);
        let collected_nodes = functions::collect_path(ast.clone(), &path);
        assert_snapshot!(collected_nodes.iter().map(|n| format!("{}", n)).collect::<String>());
        let path_from_root = functions::path_from_root(&node);
        assert_eq!(path, path_from_root);
    }

    #[test]
    #[serial]
    fn test_empty_ast_path() {
        let path = AstPath::from_str("").expect("Failed to parse empty AstPath");
        assert_eq!(path.indices.len(), 0);
    }

    #[test]
    #[serial]
    fn test_invalid_ast_path() {
        let result = AstPath::from_str("0.a.1");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "failed to parse AstPath");
    }

    #[test]
    #[serial]
    fn test_update_node_value() {
        // drop into dioxus environment to allow SIgnal creation
        let mut vdom = VirtualDom::new(|| {
            let code = r#"
                {
                    a = 1;
                    b = "b";
                }
            "#;
            let root = rnix::Root::parse(code);
            let ast = root.syntax();
            let use_source_ctx = mock_hooks::use_source_context();
            let source_signal = Signal::new(code.to_string());
            use_source_ctx.expect().return_const_st(source_signal);
            let node = functions::resolve_path(&ast, &AstPath::from_str("0.1.1").unwrap()).expect("Node should exist");
            let new_value = "2";
            functions::update_node_value(node.clone().into(), new_value);
            let updated_source = source_signal.read().clone();
            assert_eq!(updated_source, r#"
                {
                    a = 1;
                    b = 2;
                }
            "#);
            rsx! { "success" }
        });
        vdom.rebuild_in_place();
        assert!(dioxus_ssr::render(&vdom).contains("success"));
    }

    #[test]
    #[serial]
    fn test_get_bindings_in_scope() {
        let code = r#"
            let x = 1; y = 2; in {z}:{ a = x + y + z; }
        "#;
        let root = rnix::Root::parse(code);
        let ast = root.syntax();
        let node = functions::resolve_path(&ast, &AstPath::from_str("0.2.1").unwrap()).expect("Node should exist");
        let bindings = functions::get_bindings_in_scope(&node).expect("Failed to get bindings");
        assert_eq!(bindings, vec![ "z".to_string(), "x".to_string(), "y".to_string()]);
    }


}
